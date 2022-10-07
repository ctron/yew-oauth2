//! The [`NotAuthenticated`] component

use super::missing_context;
use crate::context::OAuth2Context;
use yew::prelude::*;

/// Properties for the [`NotAuthenticated`] component
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

/// Yew component, rendering children when the agent is not authenticated.
#[function_component(NotAuthenticated)]
pub fn not_authenticated(props: &Props) -> Html {
    let auth = use_context::<OAuth2Context>();

    match auth {
        None => missing_context(),
        Some(OAuth2Context::NotInitialized) => html!(),
        Some(OAuth2Context::NotAuthenticated { .. } | OAuth2Context::Failed(..)) => {
            html!({ for props.children.iter() })
        }
        Some(OAuth2Context::Authenticated { .. }) => {
            html!()
        }
    }
}
