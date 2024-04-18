use crate::{
    agent::{
        client::{expires, Client, LoginContext},
        InnerConfig, LogoutOptions, OAuth2Error,
    },
    config::openid,
    context::{Authentication, OAuth2Context},
};
use async_trait::async_trait;
use gloo_utils::window;
use oauth2::TokenResponse;
use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthenticationFlow, CoreClaimName, CoreClaimType, CoreClient,
        CoreClientAuthMethod, CoreGenderClaim, CoreGrantType, CoreJsonWebKey, CoreJsonWebKeyType,
        CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm,
        CoreJwsSigningAlgorithm, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
        CoreTokenResponse,
    },
    reqwest::async_http_client,
    AuthorizationCode, ClientId, CsrfToken, EmptyAdditionalClaims, IdTokenClaims, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, ProviderMetadata, RedirectUrl, RefreshToken, Scope,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, rc::Rc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenIdLoginState {
    pub pkce_verifier: String,
    pub nonce: String,
}

const DEFAULT_POST_LOGOUT_DIRECT_NAME: &str = "post_logout_redirect_uri";

/// An OpenID Connect based client implementation
#[derive(Clone, Debug)]
pub struct OpenIdClient {
    /// The client
    client: CoreClient,
    /// An override for the URL to end the session (logout)
    end_session_url: Option<Url>,
    /// A URL to direct to after the logout was performed
    after_logout_url: Option<String>,
    /// The name of the query parameter sent to the issuer, containing the post-logout redirect URL
    post_logout_redirect_name: Option<String>,
    /// Additional audiences of the ID token which are considered trustworthy
    additional_trusted_audiences: Vec<String>,
}

/// Additional metadata read from the discovery endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdditionalProviderMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_session_endpoint: Option<Url>,
}

impl openidconnect::AdditionalProviderMetadata for AdditionalProviderMetadata {}

pub type ExtendedProviderMetadata = ProviderMetadata<
    AdditionalProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

#[async_trait(? Send)]
impl Client for OpenIdClient {
    type TokenResponse = CoreTokenResponse;
    type Configuration = openid::Config;
    type LoginState = OpenIdLoginState;
    type SessionState = (
        String,
        Rc<IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>>,
    );

    async fn from_config(config: Self::Configuration) -> Result<Self, OAuth2Error> {
        let openid::Config {
            client_id,
            issuer_url,
            end_session_url,
            after_logout_url,
            post_logout_redirect_name,
            additional_trusted_audiences,
        } = config;

        let issuer = IssuerUrl::new(issuer_url)
            .map_err(|err| OAuth2Error::Configuration(format!("invalid issuer URL: {err}")))?;

        let metadata = ExtendedProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|err| {
                OAuth2Error::Configuration(format!("Failed to discover client: {err}"))
            })?;

        let end_session_url = end_session_url
            .map(|url| Url::parse(&url))
            .transpose()
            .map_err(|err| {
                OAuth2Error::Configuration(format!("Unable to parse end_session_url: {err}"))
            })?
            .or_else(|| metadata.additional_metadata().end_session_endpoint.clone());

        let client = CoreClient::from_provider_metadata(metadata, ClientId::new(client_id), None);

        Ok(Self {
            client,
            end_session_url,
            after_logout_url,
            post_logout_redirect_name,
            additional_trusted_audiences,
        })
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

        if let Some(audience) = &config.audience {
            req = req.add_extra_param("audience".to_string(), audience);
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
            OAuth2Error::LoginResult("Server did not return an ID token".to_string())
        })?;

        let claims = Rc::new(
            id_token
                .clone()
                .into_claims(
                    &self
                        .client
                        .id_token_verifier()
                        .set_other_audience_verifier_fn(|aud| {
                            self.additional_trusted_audiences.contains(aud)
                        }),
                    &Nonce::new(state.nonce),
                )
                .map_err(|err| {
                    OAuth2Error::LoginResult(format!("failed to verify ID token: {err}"))
                })?,
        );

        Ok((
            OAuth2Context::Authenticated(Authentication {
                access_token: result.access_token().secret().to_string(),
                refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
                expires: expires(result.expires_in()),
                claims: Some(claims.clone()),
            }),
            (id_token.to_string(), claims),
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
                OAuth2Error::Refresh(format!("failed to exchange refresh token: {err}"))
            })?;

        Ok((
            OAuth2Context::Authenticated(Authentication {
                access_token: result.access_token().secret().to_string(),
                refresh_token: result.refresh_token().map(|t| t.secret().to_string()),
                expires: expires(result.expires_in()),
                claims: Some(session_state.1.clone()),
            }),
            session_state,
        ))
    }

    fn logout(&self, session_state: Self::SessionState, options: LogoutOptions) {
        if let Some(url) = &self.end_session_url {
            let mut url = url.clone();

            let name = self
                .post_logout_redirect_name
                .as_deref()
                .unwrap_or(DEFAULT_POST_LOGOUT_DIRECT_NAME);

            url.query_pairs_mut()
                .append_pair("id_token_hint", &session_state.0);

            if let Some(after) = options
                .target
                .map(|url| url.to_string())
                .or_else(|| self.after_logout_url())
            {
                url.query_pairs_mut().append_pair(name, &after);
            }

            log::info!("Navigating to: {url}");

            window().location().replace(url.as_str()).ok();
        } else {
            log::warn!("Found no session end URL");
        }
    }
}

impl OpenIdClient {
    fn after_logout_url(&self) -> Option<String> {
        if let Some(after) = &self.after_logout_url {
            if Url::parse(after).is_ok() {
                // test if this is an absolute URL
                return Some(after.to_string());
            }

            window()
                .location()
                .href()
                .ok()
                .and_then(|url| {
                    Url::parse(&url)
                        .and_then(|current| current.join(after))
                        .ok()
                })
                .map(|u| u.to_string())
        } else {
            window().location().href().ok()
        }
    }
}
