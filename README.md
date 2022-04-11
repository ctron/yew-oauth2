# OAuth2 component for Yew

[![crates.io](https://img.shields.io/crates/v/yew-oauth2.svg)](https://crates.io/crates/yew-oauth2)
[![docs.rs](https://docs.rs/yew-oauth2/badge.svg)](https://docs.rs/yew-oauth2)
[![CI](https://github.com/ctron/yew-oauth2/workflows/CI/badge.svg)](https://github.com/ctron/yew-oauth2/actions?query=workflow%3A%22CI%22)

Add to your `Cargo.toml`:

```toml
yew-oauth2 = "0.1"
```

By default, the `router` integration is disabled, you can enable it using:

```toml
yew-oauth2 = { version = "0.1", features = ["router"] }
```

## Example

A quick example, see the full example here: [yew-oauth2-example](yew-oauth2-example/)

```rust
impl Component for MyApplication {
    fn view(&self, ctx: &Context<Self>) -> Html {
        let login = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::new().start_login();
        });
        let logout = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::new().logout();
        });

        html!(
            <OAuth2
                config={
                    Config {
                        client_id: "my-client".into(),
                        auth_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/auth".into(),
                        token_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/token".into(),
                        scopes: vec![],
                    }
                }
                >
                <Failure><FailureMessage/></Failure>
                <Authenticated>
                    <p> <button onclick={logout}>{ "Logout" }</button> </p>
                    <ul>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Index}> { "Index" } </RouterAnchor<AppRoute>></li>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Component}> { "Component" } </RouterAnchor<AppRoute>></li>
                        <li><RouterAnchor<AppRoute> route={AppRoute::Function}> { "Function" } </RouterAnchor<AppRoute>></li>
                    </ul>
                    <Router<AppRoute>
                        render = { Router::render(|switch: AppRoute| { match switch {
                                AppRoute::Index => html!(<p> { "You are logged in"} </p>),
                                AppRoute::Component => html!(<ViewAuthInfoComponent />),
                                AppRoute::Function => html!(<ViewAuthInfoFunctional />),
                        }})}
                    />
                </Authenticated>
                <NotAuthenticated>
                    <Router<AppRoute>
                        render = { Router::render(move |switch: AppRoute| { match switch {
                                AppRoute::Index => html!(
                                    <p> { "You need to log in" } <button onclick={login.clone()}>{ "Login" }</button> </p>
                                ),
                                _ => html!(<LocationRedirect logout_href="/" />),
                        }})}
                    />
                </NotAuthenticated>
            </OAuth2>
        )
    }
}
```