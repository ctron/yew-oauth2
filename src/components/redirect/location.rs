//! Redirect by setting the browser's location directly.

use super::{Redirect, Redirector, RedirectorProperties};
use gloo_utils::window;
use yew::prelude::*;

/// A redirector using the browsers location.
pub struct LocationRedirector {}

impl Redirector for LocationRedirector {
    type Properties = LocationProps;

    fn new<COMP: Component>(_: &Context<COMP>) -> Self {
        Self {}
    }

    fn logout(&self, props: &Self::Properties) {
        log::debug!("Navigate due to logout: {}", props.logout_href);
        window().location().set_href(&props.logout_href).ok();
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LocationProps {
    #[prop_or_default]
    pub children: Option<Children>,

    pub logout_href: String,
}

impl RedirectorProperties for LocationProps {
    fn children(&self) -> Option<&Children> {
        self.children.as_ref()
    }
}

pub mod oauth2 {
    //! Convenient access for the OAuth2 variant
    use super::*;
    use crate::agent::client::OAuth2Client as Client;
    pub type LocationRedirect = Redirect<Client, LocationRedirector>;
}

#[cfg(feature = "openid")]
pub mod openid {
    //! Convenient access for the Open ID Connect variant
    use super::*;
    use crate::agent::client::OpenIdClient as Client;
    pub type LocationRedirect = Redirect<Client, LocationRedirector>;
}
