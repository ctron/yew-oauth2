use super::ViewAuthInfo;
use yew::prelude::*;
use yew_oauth2::prelude::*;

#[function_component(ViewAuthInfoFunctional)]
pub fn view_auth() -> Html {
    let auth = use_context::<OAuth2Context>();

    html!(
        if let Some(auth) = auth {
            <h1> { "Function component example"} </h1>
            <ViewAuthInfo {auth} />
        } else {
            { "OAuth2 context not found." }
        }
    )
}
