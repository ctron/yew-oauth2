mod location;
#[cfg(feature = "yew-router-nested")]
mod router;

pub use location::*;
#[cfg(feature = "yew-router-nested")]
pub use router::*;

use super::missing_context;
use crate::context::{OAuth2Context, Reason};
use std::marker::PhantomData;
use yew::{context::ContextHandle, prelude::*};

pub trait Redirector: 'static {
    type Properties: yew::Properties;

    fn logout(props: &Self::Properties);
}

#[derive(Debug, Clone)]
pub enum Msg {
    Context(OAuth2Context),
}

pub struct Redirect<R>
where
    R: Redirector,
{
    auth: Option<OAuth2Context>,
    _handler: Option<ContextHandle<OAuth2Context>>,
    _marker: PhantomData<R>,
}

impl<R> Component for Redirect<R>
where
    R: Redirector,
{
    type Message = Msg;
    type Properties = R::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Msg::Context);
        let (auth, handler) = match ctx.link().context::<OAuth2Context>(cb) {
            Some((auth, handler)) => (Some(auth), Some(handler)),
            None => (None, None),
        };

        log::debug!("Initial state: {auth:?}");

        let mut result = Self {
            auth: None,
            _handler: handler,
            _marker: Default::default(),
        };

        if let Some(auth) = auth {
            result.apply_state(ctx, auth);
        }

        result
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("update: {msg:?}");

        match msg {
            Self::Message::Context(auth) => {
                self.apply_state(ctx, auth);
            }
        }
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match self.auth {
            None => missing_context(),
            _ => html!(),
        }
    }
}

impl<R> Redirect<R>
where
    R: Redirector,
{
    fn apply_state(&mut self, ctx: &Context<Self>, auth: OAuth2Context) {
        if self.auth.as_ref() == Some(&auth) {
            return;
        }

        match &auth {
            OAuth2Context::NotInitialized
            | OAuth2Context::Failed(..)
            | OAuth2Context::Authenticated { .. } => {
                // nothing that we should handle
            }
            OAuth2Context::NotAuthenticated { reason } => match reason {
                Reason::NewSession => {
                    // new session, then start the login
                    super::start_login();
                }
                Reason::Expired | Reason::Logout => {
                    // expired or logged out explicitly, then redirect to logout page
                    R::logout(ctx.props());
                }
            },
        }

        self.auth = Some(auth);
    }
}
