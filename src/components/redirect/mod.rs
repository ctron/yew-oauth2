//! Components for redirecting the user

pub mod location;
#[cfg(feature = "yew-nested-router")]
pub mod router;

use super::missing_context;
use crate::agent::{Client, OAuth2Operations};
use crate::components::context::Agent;
use crate::context::{OAuth2Context, Reason};
use yew::{context::ContextHandle, prelude::*};

pub trait Redirector: 'static {
    type Properties: RedirectorProperties;

    fn new<COMP: Component>(ctx: &Context<COMP>) -> Self;

    fn logout(&self, props: &Self::Properties);
}

pub trait RedirectorProperties: yew::Properties {
    fn children(&self) -> &Html;
}

#[derive(Debug, Clone)]
pub enum Msg<C: Client> {
    Context(OAuth2Context),
    Agent(Agent<C>),
}

/// A component which redirect the user in case the context is not authenticated.
pub struct Redirect<C, R>
where
    C: Client,
    R: Redirector,
{
    auth: Option<OAuth2Context>,
    agent: Option<Agent<C>>,

    _auth_handler: Option<ContextHandle<OAuth2Context>>,
    _agent_handler: Option<ContextHandle<Agent<C>>>,

    redirector: R,
}

impl<C, R> Component for Redirect<C, R>
where
    C: Client,
    R: Redirector,
{
    type Message = Msg<C>;
    type Properties = R::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let (auth, auth_handler) = match ctx
            .link()
            .context::<OAuth2Context>(ctx.link().callback(Msg::Context))
        {
            Some((auth, handler)) => (Some(auth), Some(handler)),
            None => (None, None),
        };
        let (agent, agent_handler) = match ctx
            .link()
            .context::<Agent<C>>(ctx.link().callback(Msg::Agent))
        {
            Some((agent, handler)) => (Some(agent), Some(handler)),
            None => (None, None),
        };

        log::debug!("Initial state: {auth:?}");

        let mut result = Self {
            auth: None,
            agent,
            _auth_handler: auth_handler,
            _agent_handler: agent_handler,
            redirector: R::new(ctx),
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
            Self::Message::Agent(agent) => {
                self.agent = Some(agent);
                // we never re-render based on an agent change
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.auth {
            None => missing_context(),
            Some(OAuth2Context::Authenticated(..)) => ctx.props().children().clone(),
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
                    if let Some(agent) = &mut self.agent {
                        let _ = agent.start_login();
                    }
                }
                Reason::Expired | Reason::Logout => {
                    match self.auth {
                        None | Some(OAuth2Context::NotInitialized) => {
                            if let Some(agent) = &mut self.agent {
                                let _ = agent.start_login();
                            }
                        }
                        _ => {
                            // expired or logged out explicitly, then redirect to the logout page
                            self.logout(ctx.props());
                        }
                    }
                }
            },
        }

        self.auth = Some(auth);
    }

    fn logout(&self, props: &R::Properties) {
        self.redirector.logout(props);
    }
}
