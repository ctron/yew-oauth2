//! The agent, working in the background to manage the session and refresh tokens.
pub mod client;

mod config;
mod error;
mod ops;
mod state;

pub use client::*;
pub use error::*;
pub use ops::*;
pub use state::LoginState;

pub(crate) use config::*;

use crate::context::{Authentication, OAuth2Context, Reason};
use gloo_storage::{SessionStorage, Storage};
use gloo_timers::callback::Timeout;
use gloo_utils::{history, window};
use js_sys::Date;
use log::error;
use num_traits::cast::ToPrimitive;
use reqwest::Url;
use state::*;
use std::{cmp::min, collections::HashMap, fmt::Debug, time::Duration};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

/// Options for the login process
///
/// ## Non-exhaustive struct
///
/// The struct is "non-exhaustive", which means that it is possible to add fields without breaking the API.
///
/// In order to create an instance, follow the following pattern:
///
/// ```rust
/// # use reqwest::Url;
/// # use yew_oauth2::prelude::LoginOptions;
/// # let url = Url::parse("https://example.com").unwrap();
/// let opts = LoginOptions::default().with_redirect_url(url);
/// ```
///
/// ## Redirect & Post login redirect
///
/// By default, the login process will ask the issuer to redirect back the page that was active when starting the login
/// process. In some cases, the issuer might require a more strict set of redirect URLs, and so can only redirect back
/// to a single page. This can be enabled set setting a specific URL as `redirect_url`.
///
/// Once the user comes back from the login flow, which might actually be without any user interaction if the session
/// was still valid, users might find themselves on the redirect page. Therefore, it is advisable to forward/redirect
/// back to the original page, the one where the user left off.
///
/// While this crate does provide some assistance, the actual implementation on how to redirect is left to the user
/// of this crate. If, while starting the login process, the currently active URL differs from the `redirect_url`,
/// the agent will store the "current" URL and pass it to the provided "post login redirect callback" once the
/// login process has completed.
///
/// It could be argued, that the crate should just perform the redirect automatically, if no call back was provided.
/// However, there can be different ways to redirect, and there is no common one. One might think just setting a new
/// location in the browser should work, but that would actually cause a page reload, and would then start the login
/// process again, since the tokens are only held in memory for security reasons. Also using the browser's History API
/// won't work, as it does not notify listeners when pushing a new state.
///
/// Therefore, it is necessary to set a "post login redirect callback", which will be triggered to handle the redirect,
/// in order to allow the user of the crate to implement the needed logic. Having the `yew-nested-router`
/// feature enabled, it is possible to just call [`LoginOptions::with_nested_router_redirect`] and let the
/// router take care of this.
///
/// **NOTE:** As a summary, setting only the `redirect_url` will not be sufficient. The "post login redirect callback" must
/// also be implemented or the `yew-nested-router`feature used. Otherwise, the user would simply end up on the page defined by
/// `redirect_url`, which in most cases is not what one would expect.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct LoginOptions {
    /// Additional query parameters sent to the issuer.
    pub query: HashMap<String, String>,

    /// Defines the redirect URL. See ["Redirect & Post login redirect"](#redirect--post-login-redirect) for more information.
    ///
    /// If this field is empty, the current URL is used as a redirect URL.
    pub redirect_url: Option<Url>,

    /// Defines callback used for post-login redirect.
    ///
    /// In cases where the issuer is asked to redirect to a different page than the one being active when starting
    /// the login flow, this callback will be called with the current (when starting) URL once the login handshake
    /// is complete.
    ///
    /// If `None`, disables post-login redirect.
    pub post_login_redirect_callback: Option<Callback<String>>,
}

impl LoginOptions {
    pub fn new() -> Self {
        LoginOptions::default()
    }

    /// Set the query parameters for the login request
    pub fn with_query(mut self, query: impl IntoIterator<Item = (String, String)>) -> Self {
        self.query = HashMap::from_iter(query);
        self
    }

    /// Extend the current query parameters for the login request
    pub fn extend_query(mut self, query: impl IntoIterator<Item = (String, String)>) -> Self {
        self.query.extend(query);
        self
    }

    /// Add a query parameter for the login request
    pub fn add_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Set the redirect URL
    pub fn with_redirect_url(mut self, redirect_url: impl Into<Url>) -> Self {
        self.redirect_url = Some(redirect_url.into());
        self
    }

    /// Set a callback for post-login redirect
    pub fn with_redirect_callback(mut self, redirect_callback: Callback<String>) -> Self {
        self.post_login_redirect_callback = Some(redirect_callback);
        self
    }

    /// Use `yew-nested-router` History API for post-login redirect callback
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

/// Options for the logout process
///
///**NOTE**: This is a non-exhaustive struct. See [`LoginOptions`] for an example on how to work with this.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogoutOptions {
    /// An optional target to navigate to after the user was logged out.
    ///
    /// This would override any settings from the client configuration.
    pub target: Option<Url>,
}

impl LogoutOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(mut self, target: impl Into<Url>) -> Self {
        self.target = Some(target.into());
        self
    }
}

#[doc(hidden)]
pub enum Msg<C>
where
    C: Client,
{
    Configure(AgentConfiguration<C>),
    StartLogin(Option<LoginOptions>),
    Logout(Option<LogoutOptions>),
    Refresh,
}

/// The agent handling the OAuth2/OIDC state
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

#[doc(hidden)]
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

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct InnerConfig {
    scopes: Vec<String>,
    grace_period: Duration,
    max_expiration: Option<Duration>,
    audience: Option<String>,
    default_login_options: Option<LoginOptions>,
    default_logout_options: Option<LogoutOptions>,
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

            let mut expires = *expires;
            if let Some(max) = self.config.as_ref().and_then(|cfg| cfg.max_expiration) {
                // cap time the token expires by "max"
                expires = min(expires, max.as_secs());
            }

            // get now as seconds
            let now = Date::now() / 1000f64;
            // get delta from now to expiration minus the grace period
            let diff = expires as f64 - now - grace.as_secs_f64();

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
        let AgentConfiguration {
            config,
            scopes,
            grace_period,
            audience,
            default_login_options,
            default_logout_options,
            max_expiration,
        } = config;

        let client = C::from_config(config).await?;

        let inner = InnerConfig {
            scopes,
            grace_period,
            audience,
            default_login_options,
            default_logout_options,
            max_expiration,
        };

        Ok((client, inner))
    }

    /// When initializing, try to detect the state from the URL and session state.
    ///
    /// Returns `false` if there is no authentication state found and the result is final.
    /// Otherwise, it returns `true` and spawns a request for e.g. a code exchange.
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
                    let stored_state = get_from_store(STORAGE_KEY_CSRF_TOKEN)?;

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

            let redirect_url = get_from_store(STORAGE_KEY_REDIRECT_URL)?;
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
            .default_login_options
            .as_ref()
            .and_then(|opts| opts.post_login_redirect_callback.clone())
        else {
            return Ok(());
        };
        let Some(url) = get_from_store_optional(STORAGE_KEY_POST_LOGIN_URL)? else {
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

            if let Err(err) = &result {
                log::warn!("Failed to refresh token: {err}");
            }

            self.update_state_from_result(result);
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

    fn start_login(&mut self, options: Option<LoginOptions>) -> Result<(), OAuth2Error> {
        let client = self.client.as_ref().ok_or(OAuth2Error::NotInitialized)?;
        let config = self.config.as_ref().ok_or(OAuth2Error::NotInitialized)?;

        let options =
            options.unwrap_or_else(|| config.default_login_options.clone().unwrap_or_default());

        let current_url = Self::current_url().map_err(OAuth2Error::StartLogin)?;

        // take the parameter value first, then the agent configured value, then fall back to the default
        let redirect_url = options
            .redirect_url
            .or_else(|| {
                config
                    .default_login_options
                    .as_ref()
                    .and_then(|opts| opts.redirect_url.clone())
            })
            .unwrap_or_else(|| current_url.clone());

        if redirect_url != current_url {
            SessionStorage::set(STORAGE_KEY_POST_LOGIN_URL, current_url)
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

    fn logout_opts(&mut self, options: Option<LogoutOptions>) {
        if let Some(client) = &self.client {
            if let Some(session_state) = self.session_state.clone() {
                // let the client know that log out, clients may navigate to a different
                // page
                log::debug!("Notify client of logout");
                let options = options
                    .or_else(|| {
                        self.config
                            .as_ref()
                            .and_then(|config| config.default_logout_options.clone())
                    })
                    .unwrap_or_default();
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

    fn start_login(&self) -> Result<(), Error> {
        self.tx
            .try_send(Msg::StartLogin(None))
            .map_err(|_| Error::NoAgent)
    }

    fn start_login_opts(&self, options: LoginOptions) -> Result<(), Error> {
        self.tx
            .try_send(Msg::StartLogin(Some(options)))
            .map_err(|_| Error::NoAgent)
    }

    fn logout(&self) -> Result<(), Error> {
        self.tx
            .try_send(Msg::Logout(None))
            .map_err(|_| Error::NoAgent)
    }

    fn logout_opts(&self, options: LogoutOptions) -> Result<(), Error> {
        self.tx
            .try_send(Msg::Logout(Some(options)))
            .map_err(|_| Error::NoAgent)
    }
}
