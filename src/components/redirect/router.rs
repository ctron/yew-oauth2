//! Redirect by pushing a new [`yew_router_nested::route::Route`].

use super::{Redirect, Redirector, RedirectorProperties};
use yew::prelude::*;
use yew_nested_router::prelude::*;

/// A redirector using Yew's Router, and the Browser's History API.
pub struct RouterRedirector<R>
where
    R: Target + 'static,
{
    router: Option<RouterContext<R>>,
    _handle: Option<ContextHandle<RouterContext<R>>>,
}

impl<R> Redirector for RouterRedirector<R>
where
    R: Target + 'static,
{
    type Properties = RouterProps<R>;

    fn new<COMP: Component>(ctx: &Context<COMP>) -> Self {
        // while the "route" can change, the "router" itself does not.
        let cb = Callback::from(|_| {});
        let (router, handle) = match ctx.link().context::<RouterContext<R>>(cb) {
            Some((router, handle)) => (Some(router), Some(handle)),
            None => (None, None),
        };

        Self {
            router,
            _handle: handle,
        }
    }

    fn logout(&self, props: &Self::Properties) {
        let route = props.logout.clone();
        log::debug!("ChangeRoute due to logout: {:?}", route);

        if let Some(router) = &self.router {
            router.go(route);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RouterProps<R>
where
    R: Target + 'static,
{
    #[prop_or_default]
    pub children: Option<Children>,
    pub logout: R,
}

impl<R> RedirectorProperties for RouterProps<R>
where
    R: Target + 'static,
{
    fn children(&self) -> Option<&Children> {
        self.children.as_ref()
    }
}

pub mod oauth2 {
    //! Convenient access for the OAuth2 variant
    use super::*;
    use crate::agent::client::OAuth2Client;
    pub type RouterRedirect<R> = Redirect<OAuth2Client, RouterRedirector<R>>;
}

#[cfg(feature = "openid")]
pub mod openid {
    //! Convenient access for the Open ID Connect variant
    use super::*;
    use crate::agent::client::OpenIdClient;
    pub type RouterRedirect<R> = Redirect<OpenIdClient, RouterRedirector<R>>;
}
