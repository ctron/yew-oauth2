use chrono::{Duration, TimeZone, Utc};
use gloo_timers::callback::Interval;
use yew::{context::ContextHandle, prelude::*};
use yew_oauth2::prelude::*;

pub enum Msg {
    Context(OAuth2Context),
    Update,
}

pub struct Expiration {
    auth: Option<OAuth2Context>,
    _handle: Option<ContextHandle<OAuth2Context>>,
    _interval: Interval,
}

impl Component for Expiration {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (auth, handle) = match ctx
            .link()
            .context::<OAuth2Context>(ctx.link().callback(Msg::Context))
        {
            Some((auth, handle)) => (Some(auth), Some(handle)),
            None => (None, None),
        };

        let cb = ctx.link().callback(|()| Msg::Update);
        let interval = Interval::new(1_000, move || cb.emit(()));

        Self {
            auth,
            _handle: handle,
            _interval: interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Context(auth) => self.auth = Some(auth),
            Self::Message::Update => {
                // just trigger re-render
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(OAuth2Context::Authenticated {
            expires: Some(expires),
            ..
        }) = self.auth
        {
            let expires = Utc.timestamp(expires as i64, 0);

            let rem = expires - Utc::now();
            let rem = Duration::seconds(rem.num_seconds());
            let rem = rem
                .to_std()
                .map(|s| format!("{}", humantime::Duration::from(s)))
                .unwrap_or_else(|_| "??".into());

            html!(<div> { "Expires: "} { expires } { format!(" (remaining: {})", rem) } </div>)
        } else {
            html!()
        }
    }
}
