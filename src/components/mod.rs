pub mod authenticated;
pub mod context;
pub mod failure;
pub mod noauth;
pub mod redirect;
pub mod use_authentication;

// only put use common components

pub use authenticated::*;
pub use failure::*;
pub use noauth::*;
pub use use_authentication::*;

use crate::agent::{Client, OAuth2Dispatcher, OAuth2Operations};
use yew::prelude::*;

fn missing_context() -> Html {
    html!(<div> { "Unable to find OAuth2 context! This element needs to be wrapped into an `OAuth2` component somewhere in the hierarchy"} </div>)
}

fn start_login<C: Client>() {
    log::debug!("Triggering login");
    OAuth2Dispatcher::<C>::new().start_login();
}
