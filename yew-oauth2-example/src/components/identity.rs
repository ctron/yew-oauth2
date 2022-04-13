use yew::prelude::*;
use yew_oauth2::prelude::*;

#[function_component(ViewIdentity)]
pub fn view_identity() -> Html {
    let auth = use_context::<OAuth2Context>();

    html!(
        <>
            <h2> { "Claims"} </h2>
            if let Some(OAuth2Context::Authenticated { claims: Some(claims) , ..}) = auth {
                <code><pre>
                    { serde_json::to_string_pretty(claims.as_ref()).unwrap_or_default() }
                </pre></code>
            } else {
                { "No claims." }
            }
        </>
    )
}
