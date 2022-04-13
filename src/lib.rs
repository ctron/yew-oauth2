pub mod agent;
pub mod components;
pub mod config;
pub mod context;
pub mod prelude;

#[cfg(feature = "openid")]
pub mod openid {
    pub use crate::agent::client::OpenIdClient as Client;
    pub use crate::components::context::openid::*;
    pub use crate::components::redirect::location::openid::*;
    #[cfg(feature = "yew-router-nested")]
    pub use crate::components::redirect::router::openid::*;
    pub use crate::config::openid::*;
}

pub mod oauth2 {
    pub use crate::agent::client::OAuth2Client as Client;
    pub use crate::components::context::oauth2::*;
    pub use crate::components::redirect::location::oauth2::*;
    #[cfg(feature = "yew-router-nested")]
    pub use crate::components::redirect::router::oauth2::*;
    pub use crate::config::oauth2::*;
}
