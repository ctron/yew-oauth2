use super::{Redirect, Redirector};
use gloo_utils::window;
use std::marker::PhantomData;
use yew::prelude::*;
use yew_router_nested::agent::RouteRequest;
use yew_router_nested::prelude::Route;
use yew_router_nested::{agent::RouteAgentDispatcher, Switch};

pub struct RouterRedirector<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    _marker: PhantomData<R>,
}

impl<R> Redirector for RouterRedirector<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    type Properties = RouterProps<R>;

    fn logout(props: &Self::Properties) {
        let route = Route::from(props.logout.clone());
        RouteAgentDispatcher::new().send(RouteRequest::ChangeRoute(route));
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<R>
where
    R: Switch + PartialEq + Clone + 'static,
{
    pub logout: R,
}

pub type RouterRedirect<R> = Redirect<RouterRedirector<R>>;
