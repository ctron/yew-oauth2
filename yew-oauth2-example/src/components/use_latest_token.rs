use yew::prelude::*;
use yew_oauth2::prelude::*;

#[function_component(UseLatestToken)]
pub fn use_latest_token() -> Html {
    let latest_token = use_latest_access_token().unwrap();

    let node_ref = use_node_ref();
    let onclick = use_callback(
        |_, (node_ref, latest_token)| {
            if let Some(node) = node_ref.get() {
                node.set_text_content(Some(&format!("{:?}", latest_token.access_token())));
            }
        },
        (node_ref.clone(), latest_token),
    );

    html!(
        <>
            <h2> { "Use latest example"} </h2>
            <p> {"A hook which gets a handle to the latest access token, but not re-render the component based on its change. You can actively get the token from it. Click on the button to retrieve the most recent valid token."} </p>

            <button type="button" {onclick}> {"Get token"} </button>

            <div>
                <strong>{"Token:"}</strong> <span ref={node_ref}></span>
            </div>
        </>
    )
}
