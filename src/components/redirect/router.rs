use super::{Redirect, Redirector, RedirectorProperties};
use std::marker::PhantomData;
use yew::prelude::*;
use yew_router_nested::{
    agent::{RouteAgentDispatcher, RouteRequest},
    prelude::Route,
    RouteState, Switch,
};

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
        log::debug!("ChangeRoute due to logout: {:?}", route);
        RouteAgentDispatcher::<STATE>::new().send(RouteRequest::ChangeRoute(route));
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    #[prop_or_default]
    pub children: Option<Children>,
    pub logout: R,
}

impl<R> RedirectorProperties for RouterProps<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    fn children(&self) -> Option<&Children> {
        self.children.as_ref()
    }
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
