# OAuth2 component for Yew

[![crates.io](https://img.shields.io/crates/v/yew-oauth2.svg)](https://crates.io/crates/yew-oauth2)
[![docs.rs](https://docs.rs/yew-oauth2/badge.svg)](https://docs.rs/yew-oauth2)
[![CI](https://github.com/ctron/yew-oauth2/workflows/CI/badge.svg)](https://github.com/ctron/yew-oauth2/actions?query=workflow%3A%22CI%22)

Add to your `Cargo.toml`:

```toml
yew-oauth2 = "0.3"
```

By default, the `router` integration is disabled, you can enable it using:

```toml
yew-oauth2 = { version = "0.3", features = ["router"] }
```

## OpenID Connect

Starting with version `0.2.0`, this crate also supports Open ID Connect. That should be just an extension on top
of OAuth2, but the reality is more complicated.

In order to use this, a different crate is required in the background. That crate also has a dependency on `ring`, which
uses a lot of C code, which is not available on WASM.

That is why this functionality is gated by the feature `openid`. When you enable this feature, for the time being, you
will also need to use the patched version of `openidconnect`, by adding the following to your `Cargo.toml`:

```toml
[dependencies]
# YES, you need to add this additionally to your application!
openidconnect = { version = "2.2", default-features = false, features = ["reqwest", "rustls-tls", "rustcrypto"] }

[patch.crates-io]
openidconnect = { git = "https://github.com/ctron/openidconnect-rs", rev = "6ca4a9ab9de35600c44a8b830693137d4769edf4" }
```

Also see: https://github.com/ramosbugs/openidconnect-rs/pull/58

## Example

A quick example, see the full example here: [yew-oauth2-example](yew-oauth2-example/)

```rust

use yew_oauth2::prelude::*;
use yew_oauth2::oauth2::*; // use `openid::*` when using OpenID connect

impl Component for MyApplication {
    fn view(&self, ctx: &Context<Self>) -> Html {
        let login = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::<Client>:::new().start_login();
        });
        let logout = ctx.link().callback_once(|_: MouseEvent| {
            OAuth2Dispatcher::<Client>::new().logout();
        });

        html!(
            <OAuth2
                config={
                    Config {
                        client_id: "my-client".into(),
                        auth_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/auth".into(),
                        token_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/token".into(),
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

### Testing

Testing the example projects locally can be done using a local Keycloak instance and `trunk`.

Start the Keycloak instance using:

```shell
podman-compose -f develop/docker-compose.yaml up
```

Then start `trunk` with the local developer instance:

```shell
cd yew-oauth2-example # or yew-oauth2-redirect-example
trunk serve
```

And navigate your browser to [http://localhost:8080](http://localhost:8080).

**NOTE:** It is important to use `http://localhost:8080` instead of e.g. `http://127.0.0.1:8080`, as Keycloak is configured by default to use `http://localhost:*` as a valid redirect URL when in dev-mode. Otherwise, you will get
an "invalid redirect" error from Keycloak.
