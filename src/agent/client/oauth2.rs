use crate::{
    agent::{
        client::{expires, Client, LoginContext},
        InnerConfig, OAuth2Error,
    },
    config::oauth2,
    context::{Authentication, OAuth2Context},
};
use ::oauth2::{
    basic::{BasicClient, BasicTokenResponse},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginState {
    pub pkce_verifier: String,
}

/// An OAuth2 based client implementation
#[derive(Clone, Debug)]
pub struct OAuth2Client {
    client: BasicClient,
}

impl OAuth2Client {
    fn make_authenticated(result: BasicTokenResponse) -> OAuth2Context {
        OAuth2Context::Authenticated(Authentication {
            access_token: result.access_token().secret().to_string(),
            refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
            expires: expires(result.expires_in()),
            #[cfg(feature = "openid")]
            claims: None,
        })
    }
}

#[async_trait(?Send)]
impl Client for OAuth2Client {
    type TokenResponse = BasicTokenResponse;
    type Configuration = oauth2::Config;
    type LoginState = LoginState;
    type SessionState = ();

    async fn from_config(config: Self::Configuration) -> Result<Self, OAuth2Error> {
        let oauth2::Config {
            client_id,
            auth_url,
            token_url,
        } = config;

        let client = BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new(auth_url)
                .map_err(|err| OAuth2Error::Configuration(format!("invalid auth URL: {err}")))?,
            Some(
                TokenUrl::new(token_url).map_err(|err| {
                    OAuth2Error::Configuration(format!("invalid token URL: {err}"))
                })?,
            ),
        );

        Ok(Self { client })
    }

    fn set_redirect_uri(mut self, url: Url) -> Self {
        self.client = self.client.set_redirect_uri(RedirectUrl::from_url(url));
        self
    }

    fn make_login_context(
        &self,
        config: &InnerConfig,
        redirect_url: Url,
    ) -> Result<LoginContext<Self::LoginState>, OAuth2Error> {
        let client = self
            .client
            .clone()
            .set_redirect_uri(RedirectUrl::from_url(redirect_url));

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut req = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(
                config
                    .scopes
                    .iter()
                    .map(|s| Scope::new(s.to_string()))
                    .collect::<Vec<_>>(),
            )
            .set_pkce_challenge(pkce_challenge);

        if let Some(audience) = &config.audience {
            req = req.add_extra_param("audience".to_string(), audience.clone())
        }

        let (url, state) = req.url();

        Ok(LoginContext {
            url,
            csrf_token: state.secret().clone(),
            state: LoginState {
                pkce_verifier: pkce_verifier.secret().clone(),
            },
        })
    }

    async fn exchange_code(
        &self,
        code: String,
        LoginState { pkce_verifier }: LoginState,
    ) -> Result<(OAuth2Context, Self::SessionState), OAuth2Error> {
        let pkce_verifier = PkceCodeVerifier::new(pkce_verifier);

        let result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|err| OAuth2Error::LoginResult(format!("failed to exchange code: {err}")))?;

        log::debug!("Exchange code result: {:?}", result);

        Ok((Self::make_authenticated(result), ()))
    }

    async fn exchange_refresh_token(
        &self,
        refresh_token: String,
        session_state: Self::SessionState,
    ) -> Result<(OAuth2Context, Self::SessionState), OAuth2Error> {
        let result = self
            .client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|err| {
                OAuth2Error::Refresh(format!("failed to exchange refresh token: {err}"))
            })?;

        Ok((Self::make_authenticated(result), session_state))
    }
}
