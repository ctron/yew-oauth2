use crate::context::OAuth2Context;
use core::fmt::{Display, Formatter};

/// An error with the OAuth2 agent
#[derive(Debug)]
pub enum OAuth2Error {
    /// Not initialized
    NotInitialized,
    /// Configuration error
    Configuration(String),
    /// Failed to start login
    StartLogin(String),
    /// Failed to handle login result
    LoginResult(String),
    /// Silent login found no active session (not a real error)
    LoginRequired,
    /// Failed to handle token refresh
    Refresh(String),
    /// Failing storing information
    Storage(String),
    /// Internal error
    Internal(String),
}

impl Display for OAuth2Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => f.write_str("not initialized"),
            Self::Configuration(err) => write!(f, "configuration error: {err}"),
            Self::StartLogin(err) => write!(f, "start login error: {err}"),
            Self::LoginResult(err) => write!(f, "login result: {err}"),
            Self::LoginRequired => f.write_str("login required"),
            Self::Refresh(err) => write!(f, "refresh error: {err}"),
            Self::Storage(err) => write!(f, "storage error: {err}"),
            Self::Internal(err) => write!(f, "internal error: {err}"),
        }
    }
}

impl std::error::Error for OAuth2Error {}

impl From<OAuth2Error> for OAuth2Context {
    fn from(err: OAuth2Error) -> Self {
        match err {
            OAuth2Error::LoginRequired => OAuth2Context::NotAuthenticated {
                reason: crate::context::Reason::LoginRequired,
            },
            other => OAuth2Context::Failed(other.to_string()),
        }
    }
}

impl OAuth2Error {
    pub(crate) fn storage_key_empty(key: impl Display) -> Self {
        Self::Storage(format!("Missing value for key: {key}"))
    }
}
