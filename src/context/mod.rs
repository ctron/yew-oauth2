//! The Authentication Context

mod utils;
pub use utils::*;

use yew::prelude::*;

#[cfg(feature = "openid")]
pub type Claims = openidconnect::IdTokenClaims<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
>;

/// The authentication information
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(not(feature = "openid"), derive(Eq))]
pub struct Authentication {
    /// The access token
    pub access_token: String,
    /// An optional refresh token
    pub refresh_token: Option<String>,
    /// OpenID claims
    #[cfg(feature = "openid")]
    pub claims: Option<std::rc::Rc<Claims>>,
    /// Expiration timestamp in seconds
    pub expires: Option<u64>,
}

/// The authentication context
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "openid"), derive(Eq))]
pub enum OAuth2Context {
    /// The agent is not initialized yet.
    NotInitialized,
    /// Not authenticated.
    NotAuthenticated {
        /// Reason why it is not authenticated.
        reason: Reason,
    },
    /// Session is authenticated.
    Authenticated(Authentication),
    /// Something failed.
    Failed(String),
}

impl OAuth2Context {
    /// Get the optional authentication.
    ///
    /// Allows easy access to the authentication information. Will return [`None`] if the
    /// context is not authenticated.
    pub fn authentication(&self) -> Option<&Authentication> {
        match self {
            Self::Authenticated(auth) => Some(auth),
            _ => None,
        }
    }

    /// Get the access token, if the context is [`OAuth2Context::Authenticated`]
    pub fn access_token(&self) -> Option<&str> {
        self.authentication().map(|auth| auth.access_token.as_str())
    }

    /// Get the claims, if the context is [`OAuth2Context::Authenticated`]
    #[cfg(feature = "openid")]
    pub fn claims(&self) -> Option<&Claims> {
        self.authentication()
            .and_then(|auth| auth.claims.as_ref().map(|claims| claims.as_ref()))
    }
}

/// Get the authentication state.
#[hook]
pub fn use_auth_state() -> Option<OAuth2Context> {
    use_context()
}

/// The reason why the context is un-authenticated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Because the user didn't log in so far.
    NewSession,
    /// Because there was a session, but now it expired.
    Expired,
    /// Because the user chose to log out.
    Logout,
}
