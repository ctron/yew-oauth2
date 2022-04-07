mod support;

pub use support::*;

use crate::{
    config::OAuth2Configuration,
    context::{OAuth2Context, Reason},
};
use gloo_storage::{SessionStorage, Storage};
use gloo_timers::callback::Timeout;
use gloo_utils::{history, window};
use js_sys::Date;
use num_traits::cast::ToPrimitive;
use oauth2::{
    basic::{BasicClient, BasicTokenResponse},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentConfiguration {
    pub config: OAuth2Configuration,
    pub grace_period: Duration,
}

#[derive(Debug)]
pub enum Msg {
    Configure(Box<Result<(BasicClient, InnerConfig), OAuth2Error>>),
    Change(OAuth2Context),
    Refresh,
}

#[derive(Debug)]
pub enum In {
    /// Initialize and configure the agent.
    Init(AgentConfiguration),
    // Reconfigure the agent.
    Configure(AgentConfiguration),
    Login,
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
    scopes: Vec<Scope>,
    grace_period: Duration,
}

pub struct OAuth2Agent {
    link: AgentLink<Self>,
    client: Option<BasicClient>,
    config: Option<InnerConfig>,

    clients: HashSet<HandlerId>,
    state: OAuth2Context,

    timeout: Option<Timeout>,
}

impl Agent for OAuth2Agent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = In;
    type Output = Out;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            client: None,
            config: None,
            clients: Default::default(),
            state: OAuth2Context::NotInitialized,
            timeout: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        log::debug!("Update: {:?}", msg);

        match msg {
            Self::Message::Configure(outcome) => match *outcome {
                Ok((client, config)) => {
                    self.client = Some(client);
                    self.config = Some(config);

                    if matches!(self.state, OAuth2Context::NotInitialized) {
                        let detected = self.detect_state();
                        log::debug!("Detected state: {detected:?}");
                        match detected {
                            Ok(true) => {}
                            Ok(false) => {
                                self.update_state(OAuth2Context::NotAuthenticated {
                                    reason: Reason::NewSession,
                                });
                            }
                            Err(err) => {
                                self.update_state(err.into());
                            }
                        }
                    }
                }
                Err(err) => {
                    if matches!(self.state, OAuth2Context::NotInitialized) {
                        self.update_state(err.into());
                    }
                }
            },
            Self::Message::Change(state) => {
                self.update_state(state);
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
            Self::Input::Login => {
                // start the login
                if let Err(err) = self.start_login() {
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
                self.update_state(OAuth2Context::NotAuthenticated {
                    reason: Reason::Logout,
                });
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
const STORAGE_KEY_PKCE_VERIFIER: &str = "ctron/oauth2/pkceVerifier";
const STORAGE_KEY_REDIRECT_URL: &str = "ctron/oauth2/redirectUrl";

impl OAuth2Agent {
    fn update_state(&mut self, state: OAuth2Context) {
        log::debug!("update state: {state:?}");

        if let OAuth2Context::Authenticated {
            expires: Some(expires),
            ..
        } = &state
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
    }

    async fn make_client(
        config: AgentConfiguration,
    ) -> Result<(BasicClient, InnerConfig), OAuth2Error> {
        match config.config {
            OAuth2Configuration::Provided(client_config) => {
                let client = BasicClient::new(
                    ClientId::new(client_config.client_id),
                    None,
                    AuthUrl::new(client_config.auth_url).map_err(|err| {
                        OAuth2Error::Configuration(format!("invalid auth URL: {err}"))
                    })?,
                    Some(TokenUrl::new(client_config.token_url).map_err(|err| {
                        OAuth2Error::Configuration(format!("invalid token URL: {err}"))
                    })?),
                );

                let config = InnerConfig {
                    scopes: client_config.scopes.into_iter().map(Scope::new).collect(),
                    grace_period: config.grace_period,
                };

                Ok((client, config))
            }
        }
    }

    fn make_auth_url(&self) -> Result<(Url, CsrfToken, PkceCodeVerifier, String), OAuth2Error> {
        let client = self.client.as_ref().ok_or(OAuth2Error::NotInitialized)?;

        let redirect_url = Self::current_url().map_err(OAuth2Error::StartLogin)?;
        let client = client
            .clone()
            .set_redirect_uri(RedirectUrl::from_url(redirect_url.clone()));

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (url, state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(
                self.config
                    .as_ref()
                    .map(|c| {
                        c.scopes
                            .iter()
                            .map(|s| oauth2::Scope::new(s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            )
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((url, state, pkce_verifier, redirect_url.into()))
    }

    fn current_url() -> Result<Url, String> {
        let href = window().location().href().map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| "unable to get current location".to_string())
        })?;
        Url::parse(&href).map_err(|err| err.to_string())
    }

    fn start_login(&self) -> Result<(), OAuth2Error> {
        let (url, state, pkce_verifier, redirect_url) = self.make_auth_url()?;

        SessionStorage::set(STORAGE_KEY_CSRF_TOKEN, state.secret())
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_PKCE_VERIFIER, pkce_verifier.secret())
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        SessionStorage::set(STORAGE_KEY_REDIRECT_URL, redirect_url)
            .map_err(|err| OAuth2Error::StartLogin(err.to_string()))?;

        window().location().set_href(url.as_str()).map_err(|err| {
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

            let pkce_verifier = Self::get_from_store(STORAGE_KEY_PKCE_VERIFIER)?;
            log::debug!("PKCE verifier: {pkce_verifier}");
            let pkce_verifier = PkceCodeVerifier::new(pkce_verifier);

            let redirect_url = Self::get_from_store(STORAGE_KEY_REDIRECT_URL)?;
            log::debug!("Redirect URL: {redirect_url}");
            let redirect_url = RedirectUrl::new(redirect_url).map_err(|err| {
                OAuth2Error::LoginResult(format!("Failed to parse redirect URL: {err}"))
            })?;

            let client = client.clone().set_redirect_uri(redirect_url);

            spawn_local(async move {
                match Self::exchange_code(client, code, pkce_verifier).await {
                    Ok(state) => link.send_message(Msg::Change(state)),
                    Err(err) => link.send_message(Msg::Change(err.into())),
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

    async fn exchange_code(
        client: BasicClient,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<OAuth2Context, OAuth2Error> {
        let result = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|err| OAuth2Error::LoginResult(format!("failed to exchange code: {err}")))?;

        log::debug!("Exchange code result: {:?}", result);
        Ok(Self::make_authenticated(result))
    }

    fn make_authenticated(result: BasicTokenResponse) -> OAuth2Context {
        let expires = if let Some(expires_in) = result.expires_in() {
            let expires = ((Date::now() / 1000f64) + expires_in.as_secs_f64())
                .to_u64()
                .unwrap_or(u64::MAX);
            Some(expires)
        } else {
            None
        };

        OAuth2Context::Authenticated {
            access_token: result.access_token().secret().to_string(),
            refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
            expires,
        }
    }

    /// Extract the state from the query.
    fn find_query_state() -> Option<State> {
        if let Ok(url) = Self::current_url() {
            let query: HashMap<_, _> = url.query_pairs().collect();

            Some(State {
                code: query.get("code").map(ToString::to_string),
                state: query.get("state").map(ToString::to_string),
                session_state: query.get("session_state").map(ToString::to_string),
                error: query.get("error").map(ToString::to_string),
            })
        } else {
            None
        }
    }

    fn refresh(&mut self) {
        let client = if let Some(client) = &self.client {
            client.clone()
        } else {
            // we need to refresh but lost our client
            self.update_state(OAuth2Context::NotAuthenticated {
                reason: Reason::Expired,
            });
            return;
        };

        if let OAuth2Context::Authenticated {
            refresh_token: Some(refresh_token),
            ..
        } = &self.state
        {
            log::debug!("Triggering refresh");

            let refresh_token = refresh_token.clone();
            let link = self.link.clone();
            spawn_local(async move {
                match Self::exchange_refresh_token(client, refresh_token).await {
                    Ok(result) => link.send_message(Msg::Change(result)),
                    Err(err) => link.send_message(Msg::Change(err.into())),
                }
            })
        }
    }

    async fn exchange_refresh_token(
        client: BasicClient,
        refresh_token: String,
    ) -> Result<OAuth2Context, OAuth2Error> {
        let refresh_token = RefreshToken::new(refresh_token);
        let result = client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await
            .map_err(|err| {
                OAuth2Error::LoginResult(format!("failed to exchange refresh token: {err}"))
            })?;

        log::debug!("Refresh token result: {result:?}");

        Ok(Self::make_authenticated(result))
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
    #[allow(unused)]
    session_state: Option<String>,
    error: Option<String>,
}
