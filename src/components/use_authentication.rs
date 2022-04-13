use super::missing_context;
use crate::context::{Authentication, OAuth2Context};
use std::rc::Rc;
use yew::prelude::*;

pub trait UseAuthenticationProperties: Clone {
    fn set_authentication(&mut self, auth: Authentication);
}

#[derive(Clone, Debug, Properties)]
pub struct Props<C>
where
    C: Component,
    C::Properties: UseAuthenticationProperties,
{
    pub children: ChildrenWithProps<C>,
}

impl<C> PartialEq for Props<C>
where
    C: Component,
    C::Properties: UseAuthenticationProperties,
{
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

/// A component which injects the authentication into the properties of component.
///
/// The component's properties must implement the trait [`UseAuthenticationProperties`].
#[function_component(UseAuthentication)]
pub fn use_authentication<C>(props: &Props<C>) -> Html
where
    C: Component,
    C::Properties: UseAuthenticationProperties,
{
    let auth = use_context::<OAuth2Context>();

    html!(
        if let Some(auth) = auth {
            if let OAuth2Context::Authenticated(auth) = auth {
                { for props.children.iter().map(|mut c|{
                    let props = Rc::make_mut(&mut c.props);
                    props.set_authentication(auth.clone());
                    c
                }) }
            }
        } else {
            { missing_context() }
        }
    )
}
