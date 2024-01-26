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
    Authenticated(AuthenticatedRoute),
    #[target(index)]
    Index,
}

#[derive(Target, Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedRoute {
    Component,
    Function,
    UseAuthentication,
    #[target(index)]
    Index,
}

#[function_component(Content)]
pub fn content() -> Html {
    let agent = use_auth_agent().expect("Requires OAuth2Context component in parent hierarchy");

    let login = use_callback(agent.clone(), |_, agent| {
        if let Err(err) = agent.start_login() {
            log::warn!("Failed to start login: {err}");
        }
    });
    let logout = use_callback(agent, |_, agent| {
        if let Err(err) = agent.logout() {
            log::warn!("Failed to logout: {err}");
        }
    });

    html!(
        <Router<AppRoute>>
            <Failure>
                <ul>
                    <li><FailureMessage/></li>
                </ul>
            </Failure>

            /* We show the full menu structure here */
            <ul>
                <li><Link<AppRoute> target={AppRoute::Index}> { "Public" } </Link<AppRoute>></li>
                <li><Link<AppRoute> target={AppRoute::Authenticated(AuthenticatedRoute::Index)}> { "Authenticated" } </Link<AppRoute>>
                    <ul>
                        <li><Link<AppRoute> target={AppRoute::Authenticated(AuthenticatedRoute::Component)}> { "Component" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> target={AppRoute::Authenticated(AuthenticatedRoute::Function)}> { "Function" } </Link<AppRoute>></li>
                        <li><Link<AppRoute> target={AppRoute::Authenticated(AuthenticatedRoute::UseAuthentication)}> { "Use" } </Link<AppRoute>></li>
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

            <Switch<AppRoute> render={|switch| match switch {
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
            }}/>
        </Router<AppRoute>>
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
    let config = Config::new("example", "http://localhost:8081/realms/master")
        /*
        Set the after logout URL to a public URL. Otherwise, the SSO server will redirect
        back to the current page, which is detected as a new session, and will try to log in
        again, if the page requires this.
        */
        .with_after_logout_url("/");

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
            <Content />
        </OAuth2>
        </>
    )
}
