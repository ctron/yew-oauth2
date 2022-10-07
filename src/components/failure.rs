//! The [`Failure`] component

use super::missing_context;
use crate::context::OAuth2Context;
use yew::prelude::*;

/// Properties for the [`Failure`] component
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct FailureProps {
    #[prop_or_default]
    pub id: Option<String>,
    #[prop_or_default]
    pub style: Option<String>,
    #[prop_or_default]
    pub class: Option<String>,
    #[prop_or_default]
    pub element: Option<String>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Failure)]
pub fn failure(props: &FailureProps) -> Html {
    let auth = use_context::<OAuth2Context>();

    let element = props.element.as_deref().unwrap_or("div").to_string();

    match auth {
        None => missing_context(),
        Some(OAuth2Context::Failed(..)) => {
            html!(
                <@{element}
                    id={ props.id.clone() }
                    style={ props.style.clone() }
                    class={ &props.class }
                    >
                    { for props.children.iter() }
                </@>
            )
        }
        Some(_) => html!(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct FailureMessageProps {
    #[prop_or_default]
    pub id: Option<String>,
    #[prop_or_default]
    pub style: Option<String>,
    #[prop_or_default]
    pub class: Option<String>,
    #[prop_or_default]
    pub element: Option<String>,
}

#[function_component(FailureMessage)]
pub fn failure_message(props: &FailureMessageProps) -> Html {
    let auth = use_context::<OAuth2Context>();

    let element = props.element.as_deref().unwrap_or("span").to_string();

    match auth {
        None => missing_context(),
        Some(OAuth2Context::Failed(message)) => {
            html!(
                <@{element}
                    id={ props.id.clone() }
                    style={ props.style.clone() }
                    class={ &props.class }
                    >
                    { message }
                </@>
            )
        }
        Some(_) => html!(),
    }
}
