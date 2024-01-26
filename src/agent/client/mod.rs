//! Client implementations

mod oauth2;
#[cfg(feature = "openid")]
mod openid;

pub use self::oauth2::*;
#[cfg(feature = "openid")]
pub use openid::*;

use crate::{
    agent::{InnerConfig, LogoutOptions, OAuth2Error},
    context::OAuth2Context,
};
use async_trait::async_trait;
use js_sys::Date;
use num_traits::ToPrimitive;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginContext<S>
where
    S: Serialize,
{
    pub url: Url,
    pub csrf_token: String,
    pub state: S,
}

#[async_trait(?Send)]
pub trait Client: 'static + Sized + Clone + Debug {
    type TokenResponse;
    type Configuration: Clone + Debug + PartialEq;
    type LoginState: Debug + Serialize + DeserializeOwned;
    type SessionState: Clone + Debug;

    async fn from_config(config: Self::Configuration) -> Result<Self, OAuth2Error>;

    fn set_redirect_uri(self, url: Url) -> Self;

    fn make_login_context(
        &self,
        config: &InnerConfig,
        redirect_url: Url,
    ) -> Result<LoginContext<Self::LoginState>, OAuth2Error>;

    async fn exchange_code(
        &self,
        code: String,
        login_state: Self::LoginState,
    ) -> Result<(OAuth2Context, Self::SessionState), OAuth2Error>;

    async fn exchange_refresh_token(
        &self,
        refresh_token: String,
        session_state: Self::SessionState,
    ) -> Result<(OAuth2Context, Self::SessionState), OAuth2Error>;

    /// Trigger the logout of the session
    ///
    /// Clients may choose to contact some back-channel or redirect to a logout URL.
    fn logout(&self, _session_state: Self::SessionState, _options: LogoutOptions) {}
}

/// Convert a duration to a timestamp, in seconds.
fn expires(expires_in: Option<Duration>) -> Option<u64> {
    if let Some(expires_in) = expires_in {
        let expires = ((Date::now() / 1000f64) + expires_in.as_secs_f64())
            .to_u64()
            .unwrap_or(u64::MAX);
        Some(expires)
    } else {
        None
    }
}
