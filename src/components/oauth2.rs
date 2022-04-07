use crate::{
    agent::{AgentConfiguration, OAuth2Bridge, OAuth2Operations, Out},
    config::OAuth2Configuration,
    context::OAuth2Context,
};
use std::time::Duration;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub config: OAuth2Configuration,

    #[prop_or(Duration::from_secs(30))]
    pub grace_period: Duration,

    #[prop_or_default]
    pub children: Children,
}

pub struct OAuth2 {
    context: OAuth2Context,
    agent: OAuth2Bridge,
    config: AgentConfiguration,
}

pub enum Msg {
    Context(OAuth2Context),
}

impl Component for OAuth2 {
    type Message = Msg;
    type Properties = Props;

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

impl OAuth2 {
    fn make_config(props: &Props) -> AgentConfiguration {
        AgentConfiguration {
            config: props.config.clone(),
            grace_period: props.grace_period,
        }
    }
}
