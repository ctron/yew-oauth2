use super::{Redirect, Redirector};
use std::marker::PhantomData;
use yew::prelude::*;
use yew_router_nested::agent::RouteRequest;
use yew_router_nested::prelude::Route;
use yew_router_nested::{agent::RouteAgentDispatcher, RouteState, Switch};

pub struct RouterRedirector<R, STATE = ()>
where
    R: Switch + PartialEq + Clone + 'static,
    STATE: 'static + RouteState,
{
    _marker: PhantomData<(R, STATE)>,
}

impl<R, STATE> Redirector for RouterRedirector<R, STATE>
where
    R: Switch + PartialEq + Clone + 'static,
    STATE: 'static + RouteState,
{
    type Properties = RouterProps<R>;

    fn logout(props: &Self::Properties) {
        let route = Route::<STATE>::from(props.logout.clone());
        RouteAgentDispatcher::<STATE>::new().send(RouteRequest::ChangeRoute(route));
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    pub logout: R,
}

pub mod oauth2 {
    use super::*;
    use crate::agent::client::OAuth2Client;
    pub type RouterRedirect<R> = Redirect<OAuth2Client, RouterRedirector<R>>;
}

#[cfg(feature = "openid")]
pub mod openid {
    use super::*;
    use crate::agent::client::OpenIdClient;
    pub type RouterRedirect<R> = Redirect<OpenIdClient, RouterRedirector<R>>;
}
