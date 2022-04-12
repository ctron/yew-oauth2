use super::{Redirect, Redirector};
use gloo_utils::window;
use yew::prelude::*;

pub struct LocationRedirector {}

impl Redirector for LocationRedirector {
    type Properties = LocationProps;

    fn logout(props: &Self::Properties) {
        window().location().set_href(&props.logout_href).ok();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct LocationProps {
    pub logout_href: String,
}

pub mod oauth2 {
    use super::*;
    use crate::agent::OAuth2Client;
    pub type LocationRedirect = Redirect<OAuth2Client, LocationRedirector>;
}

pub mod openid {
    use super::*;
    use crate::agent::OpenIdClient;
    pub type LocationRedirect = Redirect<OpenIdClient, LocationRedirector>;
}
