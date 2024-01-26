use super::OAuth2Error;
use gloo_storage::errors::StorageError;
use gloo_storage::{SessionStorage, Storage};
use std::fmt::Display;

pub(crate) const STORAGE_KEY_CSRF_TOKEN: &str = "ctron/oauth2/csrfToken";
pub(crate) const STORAGE_KEY_LOGIN_STATE: &str = "ctron/oauth2/loginState";
pub(crate) const STORAGE_KEY_REDIRECT_URL: &str = "ctron/oauth2/redirectUrl";
pub(crate) const STORAGE_KEY_POST_LOGIN_URL: &str = "ctron/oauth2/postLoginUrl";

#[derive(Debug)]
pub(crate) struct State {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub(crate) fn get_from_store<K: AsRef<str> + Display>(key: K) -> Result<String, OAuth2Error> {
    get_from_store_optional(&key)?.ok_or_else(|| OAuth2Error::storage_key_empty(key))
}

pub(crate) fn get_from_store_optional<K: AsRef<str> + Display>(
    key: K,
) -> Result<Option<String>, OAuth2Error> {
    match SessionStorage::get::<String>(key.as_ref()) {
        Err(StorageError::KeyNotFound(_)) => Ok(None),
        Err(err) => Err(OAuth2Error::Storage(err.to_string())),
        Ok(value) if value.is_empty() => Err(OAuth2Error::storage_key_empty(key)),
        Ok(value) => Ok(Some(value)),
    }
}

/// Login state, stored in the session
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginState {
    pub redirect_url: Option<String>,
    pub post_login_url: Option<String>,
}

impl LoginState {
    /// Read the state from the session
    pub fn from_storage() -> Result<Self, OAuth2Error> {
        Ok(Self {
            redirect_url: get_from_store_optional(STORAGE_KEY_REDIRECT_URL)?,
            post_login_url: get_from_store_optional(STORAGE_KEY_POST_LOGIN_URL)?,
        })
    }
}
