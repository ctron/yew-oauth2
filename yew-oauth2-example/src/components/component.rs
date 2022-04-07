use super::ViewAuthInfo;
use yew::context::ContextHandle;
use yew::prelude::*;
use yew_oauth2::prelude::*;

pub enum Msg {
    Update(OAuth2Context),
}

pub struct ViewAuthInfoComponent {
    auth: Option<OAuth2Context>,
    _handle: Option<ContextHandle<OAuth2Context>>,
}

impl Component for ViewAuthInfoComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (auth, handle) = match ctx
            .link()
            .context::<OAuth2Context>(ctx.link().callback(Msg::Update))
        {
            Some((auth, handle)) => (Some(auth), Some(handle)),
            None => (None, None),
        };

        Self {
            auth,
            _handle: handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Update(auth) => self.auth = Some(auth),
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!(
            if let Some(auth) = self.auth.clone() {
                <h1> { "Component example"} </h1>
                <ViewAuthInfo {auth} />
            } else {
                { "OAuth2 context not found." }
            }
        )
    }
}
