//! Components for redirecting the user

pub mod location;
#[cfg(feature = "yew-router-nested")]
pub mod router;

use super::missing_context;
use crate::agent::Client;
use crate::context::{OAuth2Context, Reason};
use std::marker::PhantomData;
use yew::{context::ContextHandle, prelude::*};

pub trait Redirector: 'static {
    type Properties: RedirectorProperties;

    fn logout(props: &Self::Properties);
}

pub trait RedirectorProperties: yew::Properties {
    fn children(&self) -> Option<&Children>;
}

#[derive(Debug, Clone)]
pub enum Msg {
    Context(OAuth2Context),
}

/// A component which redirect the user in case the context is not authenticated.
pub struct Redirect<C, R>
where
    C: Client,
    R: Redirector,
{
    auth: Option<OAuth2Context>,
    _handler: Option<ContextHandle<OAuth2Context>>,
    _marker: PhantomData<(C, R)>,
}

impl<C, R> Component for Redirect<C, R>
where
    C: Client,
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
                let changed = self.auth.as_ref() != Some(&auth);
                self.apply_state(ctx, auth);
                changed
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.auth {
            None => missing_context(),
            Some(OAuth2Context::Authenticated(..)) => match ctx.props().children() {
                Some(children) => html!({for children.iter()}),
                None => html!(),
            },
            _ => html!(),
        }
    }
}

impl<C, R> Redirect<C, R>
where
    C: Client,
    R: Redirector,
{
    fn apply_state(&mut self, ctx: &Context<Self>, auth: OAuth2Context) {
        if self.auth.as_ref() == Some(&auth) {
            return;
        }

        log::debug!("Current state: {:?}, new state: {:?}", self.auth, auth);

        match &auth {
            OAuth2Context::NotInitialized
            | OAuth2Context::Failed(..)
            | OAuth2Context::Authenticated { .. } => {
                // nothing that we should handle
            }
            OAuth2Context::NotAuthenticated { reason } => match reason {
                Reason::NewSession => {
                    // new session, then start the login
                    super::start_login::<C>();
                }
                Reason::Expired | Reason::Logout => {
                    match self.auth {
                        None | Some(OAuth2Context::NotInitialized) => {
                            super::start_login::<C>();
                        }
                        _ => {
                            // expired or logged out explicitly, then redirect to logout page
                            R::logout(ctx.props());
                        }
                    }
                }
            },
        }

        self.auth = Some(auth);
    }
}
