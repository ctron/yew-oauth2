use super::missing_context;
use crate::context::OAuth2Context;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

/// A Yew component, rendering when the agent is authenticated.
#[function_component(Authenticated)]
pub fn authenticated(props: &Props) -> Html {
    let auth = use_context::<OAuth2Context>();

    html!(
        if let Some(auth) = auth {
            if let OAuth2Context::Authenticated{..} = auth {
                { for props.children.iter() }
            }
        } else {
            { missing_context() }
        }
    )
}
