//! The [`UseAuthentication`] component

use super::missing_context;
use crate::{
    context::{Authentication, OAuth2Context},
    hook::use_auth_state,
};
use std::rc::Rc;
use yew::prelude::*;

/// A trait which component's properties must implement in order to receive the
/// context.
pub trait UseAuthenticationProperties: Clone {
    fn set_authentication(&mut self, auth: Authentication);
}

/// Properties for the [`UseAuthentication`] component
#[derive(Clone, Debug, Properties)]
pub struct UseAuthenticationComponentProperties<C>
where
    C: BaseComponent,
    C::Properties: UseAuthenticationProperties,
{
    pub children: ChildrenWithProps<C>,
}

impl<C> PartialEq for UseAuthenticationComponentProperties<C>
where
    C: BaseComponent,
    C::Properties: UseAuthenticationProperties,
{
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

/// A component which injects the authentication into the properties of component.
///
/// The component's properties must implement the trait [`UseAuthenticationProperties`].
///
/// ## Example
///
/// ```rust
/// use yew_oauth2::prelude::*;
/// use yew::prelude::*;
///
/// #[derive(Clone, Debug, PartialEq, Properties)]
/// pub struct Props {
///    #[prop_or_default]
///    pub auth: Option<Authentication>,
/// }
///
/// impl UseAuthenticationProperties for Props {
///    fn set_authentication(&mut self, auth: Authentication) {
///        self.auth = Some(auth);
///    }
/// }
///
/// #[function_component(ViewUseAuth)]
/// pub fn view_use_auth(props: &Props) -> Html {
///     html!(
///         <>
///             <h2> { "Use authentication example"} </h2>
///             <code><pre>{ format!("Auth: {:?}", props.auth) }</pre></code>
///         </>
///     )
/// }
/// ```
#[function_component(UseAuthentication)]
pub fn use_authentication<C>(props: &UseAuthenticationComponentProperties<C>) -> Html
where
    C: BaseComponent,
    C::Properties: UseAuthenticationProperties,
{
    let auth = use_auth_state();

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
