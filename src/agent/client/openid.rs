use crate::{
    agent::{
        client::{expires, Client, LoginContext},
        InnerConfig, OAuth2Error,
    },
    config::openid,
    context::OAuth2Context,
};
use async_trait::async_trait;
use oauth2::TokenResponse;
use openidconnect::{
    core::{
        CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreProviderMetadata,
        CoreTokenResponse,
    },
    reqwest::async_http_client,
    AuthorizationCode, ClientId, CsrfToken, EmptyAdditionalClaims, IdTokenClaims, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenIdLoginState {
    pub pkce_verifier: String,
    pub nonce: String,
}

#[derive(Clone, Debug)]
pub struct OpenIdClient {
    client: openidconnect::core::CoreClient,
}

#[async_trait(? Send)]
impl Client for OpenIdClient {
    type TokenResponse = CoreTokenResponse;
    type Configuration = openid::Config;
    type LoginState = OpenIdLoginState;
    type SessionState = IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>;

    async fn from_config(config: Self::Configuration) -> Result<Self, OAuth2Error> {
        let issuer = IssuerUrl::new(config.issuer_url)
            .map_err(|err| OAuth2Error::Configuration(format!("invalid issuer URL: {err}")))?;

        let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|err| {
                OAuth2Error::Configuration(format!("Failed to discover client: {err}"))
            })?;

        let client =
            CoreClient::from_provider_metadata(metadata, ClientId::new(config.client_id), None);

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

        let mut req = client.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for scope in &config.scopes {
            req = req.add_scope(Scope::new(scope.clone()));
        }

        let (url, state, nonce) = req.set_pkce_challenge(pkce_challenge).url();

        Ok(LoginContext {
            url,
            csrf_token: state.secret().clone(),
            state: OpenIdLoginState {
                pkce_verifier: pkce_verifier.secret().clone(),
                nonce: nonce.secret().clone(),
            },
        })
    }

    async fn exchange_code(
        &self,
        code: String,
        state: Self::LoginState,
    ) -> Result<(OAuth2Context, Self::SessionState), OAuth2Error> {
        let pkce_verifier = PkceCodeVerifier::new(state.pkce_verifier);

        let result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|err| OAuth2Error::LoginResult(format!("failed to exchange code: {err}")))?;

        log::debug!("Exchange code result: {:?}", result);

        let id_token = result.extra_fields().id_token().ok_or_else(|| {
            OAuth2Error::LoginResult(format!("Server did not return an ID token"))
        })?;

        let claims = id_token
            .clone()
            .into_claims(&self.client.id_token_verifier(), &Nonce::new(state.nonce))
            .map_err(|err| OAuth2Error::LoginResult(format!("failed to verify ID token: {err}")))?;

        Ok((
            OAuth2Context::Authenticated {
                access_token: result.access_token().secret().to_string(),
                refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
                expires: expires(result.expires_in()),
                claims: Some(claims.clone()),
            },
            claims,
        ))
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
                OAuth2Error::LoginResult(format!("failed to exchange refresh token: {err}"))
            })?;

        Ok((
            OAuth2Context::Authenticated {
                access_token: result.access_token().secret().to_string(),
                refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
                expires: expires(result.expires_in()),
                claims: Some(session_state.clone()),
            },
            session_state,
        ))
    }
}
