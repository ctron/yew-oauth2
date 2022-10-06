use crate::components::*;
use yew::prelude::*;
use yew_oauth2::prelude::*;
use yew_router::prelude::*;

#[cfg(not(feature = "openid"))]
use yew_oauth2::oauth2::*;
#[cfg(feature = "openid")]
use yew_oauth2::openid::*;

#[cfg(not(feature = "openid"))]
use yew_oauth2::oauth2::Client;
#[cfg(feature = "openid")]
use yew_oauth2::openid::Client;

#[derive(Switch, Debug, Clone, PartialEq, Eq)]
pub enum AppRoute {
    #[to = "/component"]
    Component,
    #[to = "/function"]
    Function,
    #[to = "/use"]
    UseAuthentication,
    #[cfg(feature = "openid")]
    #[to = "/identity"]
    Identity,
    #[to = "/"]
    Index,
}

#[derive(Clone, Default, Debug, PartialEq, Properties)]
pub struct Props {}

pub struct Application {}

impl Component for Application {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let login = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::<Client>::new().start_login();
        });
        let logout = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::<Client>::new().logout();
        });

        #[cfg(not(feature = "openid"))]
        let config = Config {
            client_id: "frontend".into(),
            auth_url: "http://localhost:8081/realms/master/protocol/openid-connect/auth".into(),
            token_url: "http://localhost:8081/realms/master/protocol/openid-connect/token".into(),
        };

        #[cfg(feature = "openid")]
        let config = Config {
            client_id: "frontend".into(),
            issuer_url: "http://localhost:8081/realms/master".into(),
            additional: Default::default(),
        };

        let mode = if cfg!(feature = "openid") {
            "OpenID Connect"
        } else {
            "pure OAuth2"
        };

        #[cfg(feature = "openid")]
        let openid_routes = html! (
            <li><RouterAnchor<AppRoute> route={AppRoute::Identity}> { "Identity" } </RouterAnchor<AppRoute>></li>
        );
        #[cfg(not(feature = "openid"))]
        let openid_routes = html!();

        html!(
            <>
            <h1> { "Login example (" } {mode} { ")"} </h1>

            <OAuth2
                {config}
                scopes={vec!["openid".to_string()]}
                >
                <Failure>
                    <ul>
                        <li><FailureMessage/></li>
                    </ul>
                </Failure>
                <Authenticated>
                    <p>
                        <button onclick={logout}>{ "Logout" }</button>
                    </p>
                    <ul>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Index}> { "Index" } </RouterAnchor<AppRoute>></li>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Component}> { "Component" } </RouterAnchor<AppRoute>></li>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Function}> { "Function" } </RouterAnchor<AppRoute>></li>
                        <li><RouterAnchor<AppRoute> route={AppRoute::UseAuthentication}> { "Use" } </RouterAnchor<AppRoute>></li>
                        { openid_routes }
                    </ul>
                    <Expiration/>
                    <Router<AppRoute>
                        render = { Router::render(|switch: AppRoute| {
                            match switch {
                                AppRoute::Index => html!(<p> { "You are logged in"} </p>),
                                AppRoute::Component => html!(<ViewAuthInfoComponent />),
                                AppRoute::Function => html!(<ViewAuthInfoFunctional />),
                                AppRoute::UseAuthentication => html!(
                                    <UseAuthentication<ViewUseAuth>>
                                        <ViewUseAuth/>
                                    </UseAuthentication<ViewUseAuth>>
                                ),
                                #[cfg(feature = "openid")]
                                AppRoute::Identity => html!(<ViewIdentity />),
                            }
                        })}
                    />
                </Authenticated>
                <NotAuthenticated>
                    <Router<AppRoute>
                        render = { Router::render(move |switch: AppRoute| {
                            match switch {
                                AppRoute::Index => html!(
                                    <>
                                        <p>
                                            { "You need to log in" }
                                        </p>
                                        <p>
                                            <button onclick={login.clone()}>{ "Login" }</button>
                                        </p>
                                    </>
                                ),
                                _ => html!(<LocationRedirect logout_href="/" />),
                            }
                        })}
                    />
                </NotAuthenticated>
            </OAuth2>
            </>
        )
    }
}
