use super::ViewAuthInfo;
use yew::prelude::*;
use yew_oauth2::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    #[prop_or(dummy())]
    pub auth: Authentication,
}

impl UseAuthenticationProperties for Props {
    fn set_authentication(&mut self, auth: Authentication) {
        self.auth = auth;
    }
}

fn dummy() -> Authentication {
    Authentication {
        access_token: "".to_string(),
        refresh_token: None,
        id_token: None,
        #[cfg(feature = "openid")]
        claims: None,
        expires: None,
    }
}

#[function_component(ViewUseAuth)]
pub fn view_use_auth(props: &Props) -> Html {
    html!(
        <>
            <h2> { "Use authentication example"} </h2>
            <ViewAuthInfo auth={props.auth.clone()} />
        </>
    )
}
