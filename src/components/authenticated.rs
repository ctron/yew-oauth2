//! The [`Authenticated`] component

use super::missing_context;
use crate::context::OAuth2Context;
use yew::prelude::*;

/// Properties for the [`Authenticated`] component
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct AuthenticatedProperties {
    /// The children to show then the context is authenticated.
    pub children: Children,
}

/// A Yew component, rendering when the agent is authenticated.
#[function_component(Authenticated)]
pub fn authenticated(props: &AuthenticatedProperties) -> Html {
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
