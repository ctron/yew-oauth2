use super::ViewAuthInfo;
use yew::prelude::*;
use yew_oauth2::prelude::*;

#[function_component(ViewAuthInfoFunctional)]
pub fn view_info() -> Html {
    let auth = use_context::<OAuth2Context>();

    html!(
        if let Some(auth) = auth {
            <h2> { "Function component example"} </h2>
            <ViewAuthInfo {auth} />
        } else {
            { "OAuth2 context not found." }
        }
    )
}
