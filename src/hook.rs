//! Hooks for Yew

use crate::{context::LatestAccessToken, prelude::OAuth2Context};
use yew::prelude::*;

#[cfg(feature = "openid")]
pub mod openid {
    pub use crate::agent::client::OpenIdClient as Client;

    #[yew::hook]
    pub fn use_auth_agent() -> Option<crate::components::context::Agent<Client>> {
        crate::components::context::use_auth_agent::<Client>()
    }
}

pub mod oauth2 {
    pub use crate::agent::client::OAuth2Client as Client;

    #[yew::hook]
    pub fn use_auth_agent() -> Option<crate::components::context::Agent<Client>> {
        crate::components::context::use_auth_agent::<Client>()
    }
}

/// Get the authentication state.
#[hook]
pub fn use_auth_state() -> Option<OAuth2Context> {
    use_context()
}

/// Get a handle to retrieve the latest access token
#[hook]
pub fn use_latest_access_token() -> Option<LatestAccessToken> {
    use_context()
}
