use crate::context::OAuth2Context;
use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum OAuth2Error {
    NotInitialized,
    Configuration(String),
    StartLogin(String),
    LoginResult(String),
    Storage(String),
    Internal(String),
}

impl Display for OAuth2Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => f.write_str("not initialized"),
            Self::Configuration(err) => write!(f, "configuration error: {err}"),
            Self::StartLogin(err) => write!(f, "start login error: {err}"),
            Self::LoginResult(err) => write!(f, "login result: {err}"),
            Self::Storage(err) => write!(f, "storage error: {err}"),
            Self::Internal(err) => write!(f, "internal error: {err}"),
        }
    }
}

impl std::error::Error for OAuth2Error {}

impl From<OAuth2Error> for OAuth2Context {
    fn from(err: OAuth2Error) -> Self {
        OAuth2Context::Failed(err.to_string())
    }
}

impl OAuth2Error {
    pub(crate) fn storage_key_empty(key: impl Display) -> Self {
        Self::Storage(format!("Missing value for key: {key}"))
    }
}
