//! The Authentication Context

mod utils;

use std::cell::RefCell;
use std::rc::Rc;
pub use utils::*;

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
    pub claims: Option<Rc<Claims>>,
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

/// A handle to access the latest access token.
#[derive(Clone)]
pub struct LatestAccessToken {
    pub(crate) access_token: Rc<RefCell<Option<String>>>,
}

impl PartialEq for LatestAccessToken {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.access_token, &other.access_token)
    }
}

impl LatestAccessToken {
    /// The latest access token, if there is any.
    pub fn access_token(&self) -> Option<String> {
        match self.access_token.as_ref().try_borrow() {
            Ok(token) => (*token).clone(),
            Err(_) => None,
        }
    }

    pub(crate) fn set_access_token(&self, access_token: Option<impl Into<String>>) {
        *self.access_token.borrow_mut() = access_token.map(|s| s.into());
    }
}
