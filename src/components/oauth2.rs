use crate::agent::{OAuth2Bridge, OAuth2Configuration, OAuth2Operations, Out};
use crate::prelude::OAuth2Context;
use oauth2::url::{ParseError, Url};
use std::time::Duration;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub client_id: String,
    pub auth_url: String,
    #[prop_or_default]
    pub token_url: Option<String>,

    #[prop_or("openid".to_string())]
    pub scopes: String,

    #[prop_or_default]
    pub children: Children,

    #[prop_or(Duration::from_secs(30))]
    pub grace_period: Duration,
}

pub struct OAuth2 {
    context: OAuth2Context,
    agent: OAuth2Bridge,
}

pub enum Msg {
    Context(OAuth2Context),
}

impl Component for OAuth2 {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let config = Self::make_config(ctx.props()).expect("Create client");
        let mut agent = OAuth2Bridge::new(ctx.link().batch_callback(|out| match out {
            Out::ContextUpdate(context) => vec![Msg::Context(context)],
            _ => vec![],
        }));
        agent.init(config);

        Self {
            context: OAuth2Context::NotInitialized,
            agent,
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
        let config = Self::make_config(ctx.props()).expect("Create client");
        self.agent.init(config);
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
    fn make_config(props: &Props) -> Result<OAuth2Configuration, ParseError> {
        Ok(OAuth2Configuration {
            client_id: props.client_id.clone(),
            client_secret: None,
            auth_url: Url::parse(&props.auth_url)?,
            token_url: props.token_url.as_deref().map(Url::parse).transpose()?,
            scopes: props
                .scopes
                .split(' ')
                .filter(|s| s.is_empty())
                .map(ToString::to_string)
                .collect(),
            grace_period: props.grace_period,
        })
    }
}
