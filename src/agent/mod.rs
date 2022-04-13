pub mod client;
mod support;

pub use client::Client;
pub use support::*;

use crate::context::{Authentication, OAuth2Context, Reason};
use gloo_storage::{SessionStorage, Storage};
use gloo_timers::callback::Timeout;
use gloo_utils::{history, window};
use js_sys::Date;
use num_traits::cast::ToPrimitive;
use reqwest::Url;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
    time::Duration,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

#[derive(Debug)]
pub enum OAuth2Error {
    NotInitialized,
    Configuration(String),
    StartLogin(String),
    LoginResult(String),
    Storage(String),
}

impl Display for OAuth2Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => f.write_str("not initialized"),
            Self::Configuration(err) => write!(f, "configuration error: {err}"),
            Self::StartLogin(err) => write!(f, "start login error: {err}"),
            Self::LoginResult(err) => write!(f, "login result: {err}"),
            Self::Storage(err) => write!(f, "storage error: {err}"),
        }
    }
}

impl std::error::Error for OAuth2Error {}

impl From<OAuth2Error> for OAuth2Context {
    fn from(err: OAuth2Error) -> Self {
        OAuth2Context::Failed(err.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct AgentConfiguration<C: Client> {
    pub config: C::Configuration,
    pub scopes: Vec<String>,
    pub grace_period: Duration,
}

impl<C: Client> PartialEq for AgentConfiguration<C> {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.scopes == other.scopes
            && self.grace_period == other.grace_period
    }
}

impl<C: Client> Eq for AgentConfiguration<C> {}

#[derive(Debug)]
pub enum Msg<C: Client> {
    Configure(Box<Result<(C, InnerConfig), OAuth2Error>>),
    Change((OAuth2Context, Option<C::SessionState>)),
    Refresh,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoginOptions {
    pub query: HashMap<String, String>,
}

#[derive(Debug)]
pub enum In<C: Client> {
    /// Initialize and configure the agent.
    Init(AgentConfiguration<C>),
    // Reconfigure the agent.
    Configure(AgentConfiguration<C>),
    Login(LoginOptions),
    RequestState,
    Logout,
}

#[derive(Debug)]
pub enum Out {
    ContextUpdate(OAuth2Context),
    Error(OAuth2Error),
}

#[derive(Clone, Debug)]
pub struct InnerConfig {
    scopes: Vec<String>,
    grace_period: Duration,
}

pub struct OAuth2Agent<C: Client> {
    link: AgentLink<Self>,
    client: Option<C>,
    config: Option<InnerConfig>,

    clients: HashSet<HandlerId>,
    state: OAuth2Context,
    session_state: Option<C::SessionState>,

    timeout: Option<Timeout>,
}

impl<C: Client> Agent for OAuth2Agent<C> {
    type Reach = Context<Self>;
    type Message = Msg<C>;
    type Input = In<C>;
    type Output = Out;

    fn create(link: AgentLink<Self>) -> Self {
        log::debug!("Creating new agent");

        Self {
            link,
            client: None,
            config: None,
            clients: Default::default(),
            state: OAuth2Context::NotInitialized,
            session_state: None,
            timeout: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        log::debug!("Update: {:?}", msg);

        match msg {
            Self::Message::Configure(outcome) => match *outcome {
                Ok((client, config)) => {
                    log::debug!("Client created");

                    self.client = Some(client);
                    self.config = Some(config);

                    if matches!(self.state, OAuth2Context::NotInitialized) {
                        let detected = self.detect_state();
                        log::debug!("Detected state: {detected:?}");
                        match detected {
                            Ok(true) => {}
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
            },
            Self::Message::Change((state, session_state)) => {
                self.update_state(state, session_state);
            }
            Self::Message::Refresh => {
                self.refresh();
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.clients.insert(id);
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        log::debug!("Input: {:?}", msg);

        match msg {
            Self::Input::Init(config) | Self::Input::Configure(config) => {
                let link = self.link.clone();
                spawn_local(async move {
                    link.send_message(Msg::Configure(Box::new(Self::make_client(config).await)));
                });
            }
            Self::Input::Login(options) => {
                // start the login
                if let Err(err) = self.start_login(options) {
                    log::debug!("Failed to start login: {err}");
                    if id.is_respondable() {
                        self.link.respond(id, Out::Error(err));
                    }
                }
            }
            Self::Input::RequestState => {
                self.link
                    .respond(id, Out::ContextUpdate(self.state.clone()));
            }
            Self::Input::Logout => {
                self.update_state(
                    OAuth2Context::NotAuthenticated {
                        reason: Reason::Logout,
                    },
                    None,
                );
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.clients.remove(&id);
        }
    }
}

const STORAGE_KEY_CSRF_TOKEN: &str = "ctron/oauth2/csrfToken";
const STORAGE_KEY_LOGIN_STATE: &str = "ctron/oauth2/loginState";
const STORAGE_KEY_REDIRECT_URL: &str = "ctron/oauth2/redirectUrl";

impl<C: Client> OAuth2Agent<C> {
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

            let callback = self.link.callback(|_| Msg::Refresh);
            if diff > 0f64 {
                // while the API says millis is u32, internally it is i32
                let millis = (diff * 1000f64).to_i32().unwrap_or(i32::MAX);
                log::debug!("Starting timeout for: {}ms", millis);
                self.timeout = Some(Timeout::new(millis as u32, move || {
                    callback.emit(());
                }));
            } else {
                callback.emit(());
            }
        } else {
            self.timeout = None;
        }

        for handler in &self.clients {
            self.link
                .respond(*handler, Out::ContextUpdate(state.clone()));
        }

        self.state = state;
        self.session_state = session_state;
    }

    async fn make_client(config: AgentConfiguration<C>) -> Result<(C, InnerConfig), OAuth2Error> {
        let client = C::from_config(config.config).await?;

        let inner = InnerConfig {
            scopes: config.scopes,
            grace_period: config.grace_period,
        };

        Ok((client, inner))
    }

    fn current_url() -> Result<Url, String> {
        let href = window().location().href().map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| "unable to get current location".to_string())
        })?;
        Url::parse(&href).map_err(|err| err.to_string())
    }

    fn start_login(&self, options: LoginOptions) -> Result<(), OAuth2Error> {
        let client = self.client.as_ref().ok_or(OAuth2Error::NotInitialized)?;
        let config = self.config.as_ref().ok_or(OAuth2Error::NotInitialized)?;
        let redirect_url = Self::current_url().map_err(OAuth2Error::StartLogin)?;

        let login_context = client.make_login_context(config, redirect_url.clone())?;

        SessionStorage::set(STORAGE_KEY_CSRF_TOKEN, login_context.csrf_token)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_LOGIN_STATE, login_context.state)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_REDIRECT_URL, redirect_url)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        let mut login_url = login_context.url;

        login_url.query_pairs_mut().extend_pairs(options.query);

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

    /// When initializing, try to detect the state from the URL and session state.
    ///
    /// Returns `false` if there is no authentication state found and the result is final.
    /// Otherwise it returns `true` and spawns a request for e.g. a code exchange.
    fn detect_state(&self) -> Result<bool, OAuth2Error> {
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

            let link = self.link.clone();

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

            spawn_local(async move {
                match client.exchange_code(code, state).await {
                    Ok((state, session_state)) => {
                        link.send_message(Msg::Change((state, Some(session_state))))
                    }
                    Err(err) => link.send_message(Msg::Change((err.into(), None))),
                }
            });
            Ok(true)
        } else {
            log::debug!("Neither an error nor a code. Continue without applying state.");
            Ok(false)
        }
    }

    fn get_from_store<K: AsRef<str>>(key: K) -> Result<String, OAuth2Error> {
        let value: String = SessionStorage::get(key.as_ref())
            .map_err(|err| OAuth2Error::Storage(err.to_string()))?;

        if value.is_empty() {
            Err(OAuth2Error::Storage(format!(
                "Missing value for key: {}",
                key.as_ref()
            )))
        } else {
            Ok(value)
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

    fn refresh(&mut self) {
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

            let refresh_token = refresh_token.clone();
            let link = self.link.clone();
            spawn_local(async move {
                match client
                    .exchange_refresh_token(refresh_token, session_state)
                    .await
                {
                    Ok((context, session_state)) => {
                        link.send_message(Msg::Change((context, Some(session_state))))
                    }
                    Err(err) => link.send_message(Msg::Change((err.into(), None))),
                }
            })
        }
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
}

#[derive(Debug)]
struct State {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}
