pub mod auth;
pub mod context;
pub mod failure;
pub mod noauth;
pub mod redirect;

// only put use common components

pub use auth::*;
pub use failure::*;
pub use noauth::*;

use crate::agent::{Client, OAuth2Dispatcher, OAuth2Operations};
use yew::prelude::*;

fn missing_context() -> Html {
    html!(<div> { "Unable to find OAuth2 context! This element needs to be wrapped into an `OAuth2` component somewhere in the hierarchy"} </div>)
}

fn start_login<C: Client>() {
    log::debug!("Triggering login");
    OAuth2Dispatcher::<C>::new().start_login();
}
