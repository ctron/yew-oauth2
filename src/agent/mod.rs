//! The Yew agent, working in the background to manage the session and refresh tokens.
pub mod client;

mod config;
mod error;
mod ops;
pub mod state;

pub use client::*;
pub use config::*;
pub use error::*;
pub use ops::*;

use crate::context::{Authentication, OAuth2Context, Reason};
use gloo_storage::{errors::StorageError, SessionStorage, Storage};
use gloo_timers::callback::Timeout;
use gloo_utils::{history, window};
use js_sys::Date;
use log::error;
use num_traits::cast::ToPrimitive;
use reqwest::Url;
use state::*;
use std::fmt::Display;
use std::{collections::HashMap, fmt::Debug, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

#[derive(Debug, Clone, Default)]
pub struct LoginOptions {
    pub query: HashMap<String, String>,

    /// Defines the redirect URL.
    ///
    /// If this field is empty, the current URL is used as a redirect URL.
    pub redirect_url: Option<Url>,

    /// Defines callback used for post-login redirect.
    ///
    /// If None, disables post-login redirect
    pub(crate) post_login_redirect_callback: Option<Callback<String>>,
}

impl LoginOptions {
    pub fn new() -> Self {
        LoginOptions::default()
    }

    pub fn with_query(mut self, query: impl IntoIterator<Item = (String, String)>) -> Self {
        self.query = HashMap::from_iter(query);
        self
    }

    pub fn with_extended_query(
        mut self,
        query: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        self.query.extend(query);
        self
    }

    /// Define the redirect URL
    pub fn with_redirect_url(mut self, redirect_url: Url) -> Self {
        self.redirect_url = Some(redirect_url);
        self
    }

    /// Define callback for post-login redirect
    pub fn with_redirect_callback(mut self, redirect_callback: Callback<String>) -> Self {
        self.post_login_redirect_callback = Some(redirect_callback);
        self
    }

    /// Use `yew-nested-route` history api for post-login redirect callback
    #[cfg(feature = "yew-nested-router")]
    pub fn with_nested_router_redirect(mut self) -> Self {
        let callback = Callback::from(|url: String| {
            if yew_nested_router::History::push_state(JsValue::null(), &url).is_err() {
                error!("Unable to redirect");
            }
        });

        self.post_login_redirect_callback = Some(callback);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogoutOptions {
    /// An optional target to navigate to after the user was logged out.
    ///
    /// This would override any settings from the client configuration.
    pub target: Option<Url>,
}

pub enum Msg<C>
where
    C: Client,
{
    Configure(AgentConfiguration<C>),
    StartLogin(LoginOptions),
    Logout(LogoutOptions),
    Refresh,
}

#[derive(Clone, Debug)]
pub struct Agent<C>
where
    C: Client,
{
    tx: Sender<Msg<C>>,
}

impl<C> Agent<C>
where
    C: Client,
{
    pub fn new<F>(state_callback: F) -> Self
    where
        F: Fn(OAuth2Context) + 'static,
    {
        let (tx, rx) = channel(128);

        let inner = InnerAgent::new(tx.clone(), state_callback);
        inner.spawn(rx);

        Self { tx }
    }
}

pub struct InnerAgent<C>
where
    C: Client,
{
    tx: Sender<Msg<C>>,
    state_callback: Callback<OAuth2Context>,
    config: Option<InnerConfig>,
    client: Option<C>,
    state: OAuth2Context,
    session_state: Option<C::SessionState>,
    timeout: Option<Timeout>,
}

#[derive(Clone, Debug)]
pub struct InnerConfig {
    scopes: Vec<String>,
    grace_period: Duration,
    audience: Option<String>,
    options: Option<LoginOptions>,
}

impl<C> InnerAgent<C>
where
    C: Client,
{
    pub fn new<F>(tx: Sender<Msg<C>>, state_callback: F) -> Self
    where
        F: Fn(OAuth2Context) + 'static,
    {
        Self {
            tx,
            state_callback: Callback::from(state_callback),
            client: None,
            config: None,
            state: OAuth2Context::NotInitialized,
            session_state: None,
            timeout: None,
        }
    }

    fn spawn(self, rx: Receiver<Msg<C>>) {
        spawn_local(async move {
            self.run(rx).await;
        })
    }

    async fn run(mut self, mut rx: Receiver<Msg<C>>) {
        loop {
            match rx.recv().await {
                Some(msg) => self.process(msg).await,
                None => {
                    log::debug!("Agent channel closed");
                    break;
                }
            }
        }
    }

    async fn process(&mut self, msg: Msg<C>) {
        match msg {
            Msg::Configure(config) => self.configure(config).await,
            Msg::StartLogin(login) => {
                if let Err(err) = self.start_login(login) {
                    // FIXME: need to report this somehow
                    log::info!("Failed to start login: {err}");
                }
            }
            Msg::Logout(logout) => self.logout_opts(logout),
            Msg::Refresh => self.refresh().await,
        }
    }

    fn update_state(&mut self, state: OAuth2Context, session_state: Option<C::SessionState>) {
        log::debug!("update state: {state:?}");

        if let OAuth2Context::Authenticated(Authentication {
            expires: Some(expires),
            ..
        }) = &state
        {
            let grace = self
                .config
                .as_ref()
                .map(|c| c.grace_period)
                .unwrap_or_default();
            let now = Date::now() / 1000f64;
            let diff = *expires as f64 - now - grace.as_secs_f64();

            let tx = self.tx.clone();
            if diff > 0f64 {
                // while the API says millis is u32, internally it is i32
                let millis = (diff * 1000f64).to_i32().unwrap_or(i32::MAX);
                log::debug!("Starting timeout for: {}ms", millis);
                self.timeout = Some(Timeout::new(millis as u32, move || {
                    let _ = tx.try_send(Msg::Refresh);
                }));
            } else {
                // token already expired
                let _ = tx.try_send(Msg::Refresh);
            }
        } else {
            self.timeout = None;
        }

        self.notify_state(state.clone());

        self.state = state;
        self.session_state = session_state;
    }

    fn notify_state(&self, state: OAuth2Context) {
        self.state_callback.emit(state);
    }

    /// Called once the configuration process has finished, applying the outcome.
    async fn configured(&mut self, outcome: Result<(C, InnerConfig), OAuth2Error>) {
        match outcome {
            Ok((client, config)) => {
                log::debug!("Client created");

                self.client = Some(client);
                self.config = Some(config);

                if matches!(self.state, OAuth2Context::NotInitialized) {
                    let detected = self.detect_state().await;
                    log::debug!("Detected state: {detected:?}");
                    match detected {
                        Ok(true) => {
                            if let Err(e) = self.post_login_redirect() {
                                error!("Post-login redirect failed: {e}");
                            }
                        }
                        Ok(false) => {
                            self.update_state(
                                OAuth2Context::NotAuthenticated {
                                    reason: Reason::NewSession,
                                },
                                None,
                            );
                        }
                        Err(err) => {
                            self.update_state(err.into(), None);
                        }
                    }
                }
            }
            Err(err) => {
                log::debug!("Failed to configure client: {err}");
                if matches!(self.state, OAuth2Context::NotInitialized) {
                    self.update_state(err.into(), None);
                }
            }
        }
    }

    async fn make_client(config: AgentConfiguration<C>) -> Result<(C, InnerConfig), OAuth2Error> {
        let client = C::from_config(config.config).await?;

        let inner = InnerConfig {
            scopes: config.scopes,
            grace_period: config.grace_period,
            audience: config.audience,
            options: config.options,
        };

        Ok((client, inner))
    }

    /// When initializing, try to detect the state from the URL and session state.
    ///
    /// Returns `false` if there is no authentication state found and the result is final.
    /// Otherwise it returns `true` and spawns a request for e.g. a code exchange.
    async fn detect_state(&mut self) -> Result<bool, OAuth2Error> {
        let client = self.client.as_ref().ok_or(OAuth2Error::NotInitialized)?;

        let state = if let Some(state) = Self::find_query_state() {
            state
        } else {
            // unable to get location and query
            return Ok(false);
        };

        log::debug!("Found state: {:?}", state);

        if let Some(error) = state.error {
            log::info!("Login error from server: {error}");

            // cleanup URL
            Self::cleanup_url();

            // error from the OAuth2 server
            return Err(OAuth2Error::LoginResult(error));
        }

        if let Some(code) = state.code {
            // cleanup URL
            Self::cleanup_url();

            match state.state {
                None => {
                    return Err(OAuth2Error::LoginResult(
                        "Missing state from server".to_string(),
                    ))
                }
                Some(state) => {
                    let stored_state = Self::get_from_store(STORAGE_KEY_CSRF_TOKEN)?;

                    if state != stored_state {
                        return Err(OAuth2Error::LoginResult("State mismatch".to_string()));
                    }
                }
            }

            let state: C::LoginState =
                SessionStorage::get(STORAGE_KEY_LOGIN_STATE).map_err(|err| {
                    OAuth2Error::Storage(format!("Failed to load login state: {err}"))
                })?;

            log::debug!("Login state: {state:?}");

            let redirect_url = Self::get_from_store(STORAGE_KEY_REDIRECT_URL)?;
            log::debug!("Redirect URL: {redirect_url}");
            let redirect_url = Url::parse(&redirect_url).map_err(|err| {
                OAuth2Error::LoginResult(format!("Failed to parse redirect URL: {err}"))
            })?;

            let client = client.clone().set_redirect_uri(redirect_url);

            let result = client.exchange_code(code, state).await;
            self.update_state_from_result(result);

            Ok(true)
        } else {
            log::debug!("Neither an error nor a code. Continue without applying state.");
            Ok(false)
        }
    }

    fn post_login_redirect(&self) -> Result<(), OAuth2Error> {
        let config = self.config.as_ref().ok_or(OAuth2Error::NotInitialized)?;
        let Some(redirect_callback) = config
            .options
            .as_ref()
            .and_then(|opts| opts.post_login_redirect_callback.clone())
        else {
            return Ok(());
        };
        let Some(url) = Self::get_from_store_optional(STORAGE_KEY_POST_LOGIN_URL)? else {
            return Ok(());
        };
        SessionStorage::delete(STORAGE_KEY_POST_LOGIN_URL);
        redirect_callback.emit(url);

        Ok(())
    }

    fn update_state_from_result(
        &mut self,
        result: Result<(OAuth2Context, C::SessionState), OAuth2Error>,
    ) {
        match result {
            Ok((state, session_state)) => {
                self.update_state(state, Some(session_state));
            }
            Err(err) => {
                self.update_state(err.into(), None);
            }
        }
    }

    async fn refresh(&mut self) {
        let (client, session_state) =
            if let (Some(client), Some(session_state)) = (&self.client, &self.session_state) {
                (client.clone(), session_state.clone())
            } else {
                // we need to refresh but lost our client
                self.update_state(
                    OAuth2Context::NotAuthenticated {
                        reason: Reason::Expired,
                    },
                    None,
                );
                return;
            };

        if let OAuth2Context::Authenticated(Authentication {
            refresh_token: Some(refresh_token),
            ..
        }) = &self.state
        {
            log::debug!("Triggering refresh");

            let result = client
                .exchange_refresh_token(refresh_token.clone(), session_state)
                .await;
            self.update_state_from_result(result);
        }
    }

    fn get_from_store<K: AsRef<str> + Display>(key: K) -> Result<String, OAuth2Error> {
        Self::get_from_store_optional(&key)?.ok_or_else(|| OAuth2Error::storage_key_empty(key))
    }

    fn get_from_store_optional<K: AsRef<str> + Display>(
        key: K,
    ) -> Result<Option<String>, OAuth2Error> {
        match SessionStorage::get::<String>(key.as_ref()) {
            Err(StorageError::KeyNotFound(_)) => Ok(None),
            Err(err) => Err(OAuth2Error::Storage(err.to_string())),
            Ok(value) if value.is_empty() => Err(OAuth2Error::storage_key_empty(key)),
            Ok(value) => Ok(Some(value)),
        }
    }

    /// Extract the state from the query.
    fn find_query_state() -> Option<State> {
        if let Ok(url) = Self::current_url() {
            let query: HashMap<_, _> = url.query_pairs().collect();

            Some(State {
                code: query.get("code").map(ToString::to_string),
                state: query.get("state").map(ToString::to_string),
                error: query.get("error").map(ToString::to_string),
            })
        } else {
            None
        }
    }

    fn current_url() -> Result<Url, String> {
        let href = window().location().href().map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| "unable to get current location".to_string())
        })?;
        Url::parse(&href).map_err(|err| err.to_string())
    }

    fn cleanup_url() {
        if let Ok(mut url) = Self::current_url() {
            url.set_query(None);
            let state = history().state().unwrap_or(JsValue::NULL);
            history()
                .replace_state_with_url(&state, "", Some(url.as_str()))
                .ok();
        }
    }

    async fn configure(&mut self, config: AgentConfiguration<C>) {
        self.configured(Self::make_client(config).await).await;
    }

    fn start_login(&mut self, options: LoginOptions) -> Result<(), OAuth2Error> {
        let client = self.client.as_ref().ok_or(OAuth2Error::NotInitialized)?;
        let config = self.config.as_ref().ok_or(OAuth2Error::NotInitialized)?;

        let post_login_url = Self::current_url().map_err(OAuth2Error::StartLogin)?;

        // take the parameter value first, then the agent configured value, then fall back to the default
        let redirect_url = match options.redirect_url.or_else(|| {
            config
                .options
                .as_ref()
                .and_then(|opts| opts.redirect_url.clone())
        }) {
            Some(redirect_url) => redirect_url,
            None => Self::current_url().map_err(OAuth2Error::StartLogin)?,
        };

        if redirect_url != post_login_url {
            SessionStorage::set(STORAGE_KEY_POST_LOGIN_URL, post_login_url)
                .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;
        }

        let login_context = client.make_login_context(config, redirect_url.clone())?;

        SessionStorage::set(STORAGE_KEY_CSRF_TOKEN, login_context.csrf_token)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_LOGIN_STATE, login_context.state)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_REDIRECT_URL, redirect_url)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        let mut login_url = login_context.url;

        login_url.query_pairs_mut().extend_pairs(options.query);
        if let Some(options) = &config.options {
            login_url
                .query_pairs_mut()
                .extend_pairs(options.query.clone());
        }

        // the next call will most likely navigate away from this page

        window()
            .location()
            .set_href(login_url.as_str())
            .map_err(|err| {
                OAuth2Error::StartLogin(
                    err.as_string()
                        .unwrap_or_else(|| "Unable to navigate to login page".to_string()),
                )
            })?;

        Ok(())
    }

    fn logout_opts(&mut self, options: LogoutOptions) {
        if let Some(client) = &self.client {
            if let Some(session_state) = self.session_state.clone() {
                // let the client know that log out, clients may navigate to a different
                // page
                log::debug!("Notify client of logout");
                client.logout(session_state, options);
            }
        }

        // There is a bug in yew, which panics during re-rendering, which might be triggered
        // by the next step. Doing the update later, might not trigger the issue as it might
        // cause the application to navigate to a different page.
        self.update_state(
            OAuth2Context::NotAuthenticated {
                reason: Reason::Logout,
            },
            None,
        );
    }
}

impl<C> OAuth2Operations<C> for Agent<C>
where
    C: Client,
{
    fn configure(&self, config: AgentConfiguration<C>) -> Result<(), Error> {
        self.tx
            .try_send(Msg::Configure(config))
            .map_err(|_| Error::NoAgent)
    }

    fn start_login_opts(&self, options: LoginOptions) -> Result<(), Error> {
        self.tx
            .try_send(Msg::StartLogin(options))
            .map_err(|_| Error::NoAgent)
    }

    fn logout_opts(&self, options: LogoutOptions) -> Result<(), Error> {
        self.tx
            .try_send(Msg::Logout(options))
            .map_err(|_| Error::NoAgent)
    }
}
