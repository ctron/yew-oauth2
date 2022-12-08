//! The main, wrapping [`OAuth2`] component

mod agent;

pub use agent::*;

use crate::{
    agent::{AgentConfiguration, Client, OAuth2Operations},
    context::OAuth2Context,
};
use agent::Agent as AgentContext;
use std::time::Duration;
use yew::prelude::*;

/// Properties for the context component.
#[derive(Clone, Debug, Properties)]
pub struct Props<C: Client> {
    /// The client configuration
    pub config: C::Configuration,

    /// Scopes to request for the session
    #[prop_or_default]
    pub scopes: Vec<String>,

    /// The grace period for the session timeout
    ///
    /// The amount of time before the token expiration when the agent will refresh it.
    #[prop_or(Duration::from_secs(30))]
    pub grace_period: Duration,

    /// Children which will have access to the [`OAuth2Context`].
    #[prop_or_default]
    pub children: Children,
}

impl<C: Client> PartialEq for Props<C> {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.scopes == other.scopes
            && self.grace_period == other.grace_period
            && self.children == other.children
    }
}

/// Yew component providing the OAuth2 context and configuring the agent.
///
/// All items making using of the OAuth2 or OpenID Connect context must be below this element.
pub struct OAuth2<C: Client> {
    context: OAuth2Context,
    agent: AgentContext<C>,
    config: AgentConfiguration<C>,
}

#[doc(hidden)]
pub enum Msg {
    Context(OAuth2Context),
}

impl<C: Client> Component for OAuth2<C> {
    type Message = Msg;
    type Properties = Props<C>;

    fn create(ctx: &Context<Self>) -> Self {
        let config = Self::make_config(ctx.props());
        let callback = ctx.link().callback(Msg::Context);

        let agent = crate::agent::Agent::new(move |s| callback.emit(s));
        let _ = agent.configure(config.clone());

        Self {
            context: OAuth2Context::NotInitialized,
            agent: AgentContext::new(agent),
            config,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Context(context) => {
                if self.context != context {
                    self.context = context;
                    return true;
                }
            }
        }
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> bool {
        let config = Self::make_config(ctx.props());
        if self.config != config {
            // only reconfigure agent when necessary
            let _ = self.agent.configure(config.clone());
            self.config = config;
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!(
            <>
                <ContextProvider<OAuth2Context> context={self.context.clone()} >
                    <ContextProvider<AgentContext<C>> context={self.agent.clone()}>
                        { for ctx.props().children.iter() }
                    </ContextProvider<AgentContext<C>>>
                </ContextProvider<OAuth2Context>>
            </>
        )
    }
}

impl<C: Client> OAuth2<C> {
    fn make_config(props: &Props<C>) -> AgentConfiguration<C> {
        AgentConfiguration {
            config: props.config.clone(),
            scopes: props.scopes.clone(),
            grace_period: props.grace_period,
        }
    }
}

#[cfg(feature = "openid")]
pub mod openid {
    //! Convenient access to OpenID Connect context
    pub type OAuth2 = super::OAuth2<crate::agent::client::OpenIdClient>;
}

pub mod oauth2 {
    //! Convenient access to OAuth2 context
    pub type OAuth2 = super::OAuth2<crate::agent::client::OAuth2Client>;
}
