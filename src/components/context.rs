use crate::{
    agent::{AgentConfiguration, Client, OAuth2Bridge, OAuth2Operations, Out},
    context::OAuth2Context,
};
use std::time::Duration;
use yew::prelude::*;

#[derive(Clone, Debug, Properties)]
pub struct Props<C: Client> {
    pub config: C::Configuration,

    #[prop_or_default]
    pub scopes: Vec<String>,

    #[prop_or(Duration::from_secs(30))]
    pub grace_period: Duration,

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
    agent: OAuth2Bridge<C>,
    config: AgentConfiguration<C>,
}

pub enum Msg {
    Context(OAuth2Context),
}

impl<C: Client> Component for OAuth2<C> {
    type Message = Msg;
    type Properties = Props<C>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut agent = OAuth2Bridge::new(ctx.link().batch_callback(|out| match out {
            Out::ContextUpdate(context) => vec![Msg::Context(context)],
            _ => vec![],
        }));

        let config = Self::make_config(ctx.props());
        agent.init(config.clone());

        Self {
            context: OAuth2Context::NotInitialized,
            agent,
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

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let config = Self::make_config(ctx.props());
        if self.config != config {
            // only reconfigure agent when necessary
            self.agent.configure(config.clone());
            self.config = config;
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!(
            <>
                <ContextProvider<OAuth2Context> context={self.context.clone()} >
                    { for ctx.props().children.iter() }
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
    pub type OAuth2 = super::OAuth2<crate::agent::client::OpenIdClient>;
}

pub mod oauth2 {
    pub type OAuth2 = super::OAuth2<crate::agent::client::OAuth2Client>;
}
