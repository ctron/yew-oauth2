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
    #[to = "/authenticated{*}"]
    Authenticated(AuthenticatedRoute),
    #[to = "/"]
    Index,
}

#[derive(Switch, Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedRoute {
    #[to = "/component"]
    Component,
    #[to = "/function"]
    Function,
    #[to = "/use"]
    UseAuthentication,
    #[to = "/"]
    Index,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Properties)]
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
            client_id: "example".into(),
            auth_url: "http://localhost:8081/realms/master/protocol/openid-connect/auth".into(),
            token_url: "http://localhost:8081/realms/master/protocol/openid-connect/token".into(),
        };

        #[cfg(feature = "openid")]
        let config = Config {
            client_id: "example".into(),
            issuer_url: "http://localhost:8081/realms/master".into(),
            additional: Additional {
                /*
                Set the after logout URL to a public URL. Otherwise, the SSO server will redirect
                back to the current page, which is detected as a new session, and will try to login
                again, if the page requires this.
                */
                after_logout_url: Some("/".into()),
                ..Default::default()
            },
        };

        let mode = if cfg!(feature = "openid") {
            "OpenID Connect"
        } else {
            "pure OAuth2"
        };

        html!(
            <>
            <h1> { "Redirect example (" } {mode} { ")"} </h1>

            <OAuth2
                {config}
                scopes={vec!["openid".to_string()]}
                >

                <Failure>
                    <ul>
                        <li><FailureMessage/></li>
                    </ul>
                </Failure>

                { /* We show the full menu structure here */ html!() }
                <ul>
                    <li><RouterAnchor<AppRoute> route={AppRoute::Index}> { "Public" } </RouterAnchor<AppRoute>></li>
                    <li><RouterAnchor<AppRoute> route={AppRoute::Authenticated(AuthenticatedRoute::Index)}> { "Authenticated" } </RouterAnchor<AppRoute>>
                        <ul>
                            <li><RouterAnchor<AppRoute> route={AppRoute::Authenticated(AuthenticatedRoute::Component)}> { "Component" } </RouterAnchor<AppRoute>></li>
                            <li><RouterAnchor<AppRoute> route={AppRoute::Authenticated(AuthenticatedRoute::Function)}> { "Function" } </RouterAnchor<AppRoute>></li>
                            <li><RouterAnchor<AppRoute> route={AppRoute::Authenticated(AuthenticatedRoute::UseAuthentication)}> { "Use" } </RouterAnchor<AppRoute>></li>
                        </ul>
                    </li>
                </ul>

                <aside>
                    <h2>{"Internal state"}</h2>
                    <div style="max-height: 300px; overflow: auto;">
                    <Debug/>
                    </div>
                </aside>

                <aside>
                    /* Always show the appropriate button */
                    <Authenticated>
                        <p>
                            <button onclick={logout}>{ "Logout" }</button>
                        </p>
                        <Expiration/>
                    </Authenticated>
                    <NotAuthenticated>
                        <p>
                            <button onclick={login.clone()}>{ "Login" }</button>
                        </p>
                    </NotAuthenticated>
                </aside>

                <Router<AppRoute>
                    render = { Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Index => html!(<p> { "Welcome"} </p>),
                            /*
                            When the user requests an authenticated page, we hide this behind
                            the `RouterRedirect` component. It will trigger a login when
                            required, but not show any children when not logged in, but forward
                            to the logout route instead.

                            **NOTE:** For OpenID Connect it is important to also set the
                            `after_logout_url` in the client configuration to some public route.
                            */
                            AppRoute::Authenticated(authenticated) => html!(
                                <RouterRedirect<AppRoute> logout={ AppRoute::Index }>
                                {
                                    match authenticated {
                                            AuthenticatedRoute::Index => html!(<p> { "You are logged in"} </p>),
                                            AuthenticatedRoute::Component => html!(<ViewAuthInfoComponent />),
                                            AuthenticatedRoute::Function => html!(<ViewAuthInfoFunctional />),
                                            AuthenticatedRoute::UseAuthentication => html!(
                                                <UseAuthentication<ViewUseAuth>>
                                                    <ViewUseAuth/>
                                                </UseAuthentication<ViewUseAuth>>
                                            ),
                                        }
                                }
                                </RouterRedirect<AppRoute>>
                            )
                        }
                    })}
                />

            </OAuth2>
            </>
        )
    }
}
