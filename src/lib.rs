#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Yew components to implement OAuth2 and OpenID Connect logins.
//!
//! ## OAuth2 or Open ID Connect
//!
//! This crate supports both plain OAuth2 and Open ID Connect (OIDC). OIDC layers a few features
//! on top of OAuth2 (like logout URLs, discovery, …).
//!
//! In order to use OIDC, you will need to enable the feature `openidconnect`.
//!
//! ## Example
//!
//! **NOTE:** Also see the readme for more examples.
//!
//! Can be used like:
//!
//! ```rust
//! use yew::prelude::*;
//! use yew_oauth2::prelude::*;
//! use yew_oauth2::oauth2::*; // use `openid::*` when using OpenID connect
//!
//! pub struct MyApplication;
//!
//! impl Component for MyApplication {
//!   type Message = ();
//!   type Properties = ();
//!
//!   fn create(ctx: &Context<Self>) -> Self {
//!     Self
//!   }
//!
//!   fn view(&self, ctx: &Context<Self>) -> Html {
//!     let login = ctx.link().callback_once(|_: MouseEvent| {
//!       OAuth2Dispatcher::<Client>::new().start_login();
//!     });
//!     let logout = ctx.link().callback_once(|_: MouseEvent| {
//!       OAuth2Dispatcher::<Client>::new().logout();
//!     });
//!
//!     let config = Config {
//!       client_id: "my-client".into(),
//!       auth_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/auth".into(),
//!       token_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/token".into(),
//!     };
//!
//!     html!(
//!       <OAuth2 config>
//!         <Failure><FailureMessage/></Failure>
//!         <Authenticated>
//!           <button onclick={logout}>{ "Logout" }</button>
//!         </Authenticated>
//!         <NotAuthenticated>
//!           <button onclick={login.clone()}>{ "Login" }</button>
//!         </NotAuthenticated>
//!       </OAuth2>
//!     )
//!   }
//! }
//! ```

pub mod agent;
pub mod components;
pub mod config;
pub mod context;
pub mod prelude;

#[cfg(feature = "openid")]
pub mod openid {
    //! Common used Open ID Connect features
    pub use crate::agent::client::OpenIdClient as Client;
    pub use crate::components::context::openid::*;
    pub use crate::components::redirect::location::openid::*;
    #[cfg(feature = "yew-router-nested")]
    pub use crate::components::redirect::router::openid::*;
    pub use crate::config::openid::*;
}

pub mod oauth2 {
    //! Common used OAuth2 features
    pub use crate::agent::client::OAuth2Client as Client;
    pub use crate::components::context::oauth2::*;
    pub use crate::components::redirect::location::oauth2::*;
    #[cfg(feature = "yew-router-nested")]
    pub use crate::components::redirect::router::oauth2::*;
    pub use crate::config::oauth2::*;
}
