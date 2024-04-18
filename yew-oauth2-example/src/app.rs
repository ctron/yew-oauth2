use crate::components::*;
use yew::prelude::*;
use yew_nested_router::{components::*, prelude::*};
use yew_oauth2::prelude::*;

#[cfg(not(feature = "openid"))]
use yew_oauth2::oauth2::*;
#[cfg(feature = "openid")]
use yew_oauth2::openid::*;

#[derive(Target, Debug, Clone, PartialEq, Eq)]
pub enum AppRoute {
    Component,
    Function,
    UseLatestToken,
    UseAuthentication,
    #[cfg(feature = "openid")]
    Identity,
    #[target(index)]
    Index,
}

#[function_component(Content)]
pub fn content() -> Html {
    let agent = use_auth_agent().expect("Requires OAuth2Context component in parent hierarchy");

    let login = {
        let agent = agent.clone();
        Callback::from(move |_: MouseEvent| {
            if let Err(err) = agent.start_login() {
                log::warn!("Failed to start login: {err}");
            }
        })
    };
    let logout = Callback::from(move |_: MouseEvent| {
        if let Err(err) = agent.logout() {
            log::warn!("Failed to logout: {err}");
        }
    });

    #[cfg(feature = "openid")]
    let openid_routes = html! (
        <li><Link<AppRoute> to={AppRoute::Identity}> { "Identity" } </Link<AppRoute>></li>
    );
    #[cfg(not(feature = "openid"))]
    let openid_routes = html!();

    html!(
        <>
            <Router<AppRoute>>
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
                        <li><Link<AppRoute> to={AppRoute::Index}> { "Index" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> to={AppRoute::Component}> { "Component" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> to={AppRoute::Function}> { "Function" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> to={AppRoute::UseAuthentication}> { "Use" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> to={AppRoute::UseLatestToken}> { "Latest Token" } </Link<AppRoute>></li>
                        { openid_routes }
                    </ul>
                    <Expiration/>
                    <Switch<AppRoute> render={|switch| match switch {
                        AppRoute::Index => html!(<p> { "You are logged in"} </p>),
                        AppRoute::Component => html!(<ViewAuthInfoComponent />),
                        AppRoute::Function => html!(<ViewAuthInfoFunctional />),
                        AppRoute::UseLatestToken => html!(<UseLatestToken/>),
                        AppRoute::UseAuthentication => html!(
                            <UseAuthentication<ViewUseAuth>>
                                <ViewUseAuth/>
                            </UseAuthentication<ViewUseAuth>>
                        ),
                        #[cfg(feature = "openid")]
                        AppRoute::Identity => html!(<ViewIdentity />),
                    }}/>
                </Authenticated>
                <NotAuthenticated>
                    <Switch<AppRoute> render={move |switch| match switch {
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
                    }} />
                </NotAuthenticated>
            </Router<AppRoute>>
        </>
    )
}

#[function_component(Application)]
pub fn app() -> Html {
    #[cfg(not(feature = "openid"))]
    let config = Config::new(
        "example",
        "http://localhost:8081/realms/master/protocol/openid-connect/auth",
        "http://localhost:8081/realms/master/protocol/openid-connect/token",
    );

    #[cfg(feature = "openid")]
    let config = Config::new("example", "http://localhost:8081/realms/master");

    let mode = if cfg!(feature = "openid") {
        "OpenID Connect"
    } else {
        "pure OAuth2"
    };

    html!(
        <>
            <h1> { "Login example (" } {mode} { ")"} </h1>

            <OAuth2
                {config}
                scopes={vec!["openid".to_string()]}
                >
                <Content/>
            </OAuth2>
        </>
    )
}
