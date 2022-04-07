use yew::prelude::*;
use yew_oauth2::context::OAuth2Context;

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub auth: OAuth2Context,
}

#[function_component(ViewAuthInfo)]
pub fn view(props: &Props) -> Html {
    html!(
        <dl>
            <dt> { "Context" } </dt>
            <dd>
                <code><pre>
                    { format!("{:#?}", props.auth) }
                </pre></code>
            </dd>
        </dl>
    )
}
