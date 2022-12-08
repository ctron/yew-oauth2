//! Components used when rendering HTML

pub mod authenticated;
pub mod context;
pub mod failure;
pub mod noauth;
pub mod redirect;
pub mod use_authentication;

// only put pub use for common components

pub use authenticated::*;
pub use failure::*;
pub use noauth::*;
pub use use_authentication::*;

use yew::prelude::*;

fn missing_context() -> Html {
    html!(<div> { "Unable to find OAuth2 context! This element needs to be wrapped into an `OAuth2` component somewhere in the hierarchy"} </div>)
}
