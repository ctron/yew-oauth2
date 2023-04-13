# OAuth2 component for Yew

[![crates.io](https://img.shields.io/crates/v/yew-oauth2.svg)](https://crates.io/crates/yew-oauth2)
[![docs.rs](https://docs.rs/yew-oauth2/badge.svg)](https://docs.rs/yew-oauth2)
[![CI](https://github.com/ctron/yew-oauth2/workflows/CI/badge.svg)](https://github.com/ctron/yew-oauth2/actions?query=workflow%3A%22CI%22)

Add to your `Cargo.toml`:

```toml
yew-oauth2 = "0.6"
```

By default, the `router` integration is disabled, you can enable it using:

```toml
yew-oauth2 = { version = "0.6", features = ["router"] }
```

## OpenID Connect

OpenID Connect requires an additional dependency and can be enabled using the feature `openid`.

Starting with version `0.6.0-alpha.1`, it is possible to use `openidconnect-rs` version 3, which is the first version
supporting WebAssembly targets without patching. However, for the moment, only an alpha version of `openidconnect-rs` 3
is released.

## Examples

A quick example how to use it (see below for more complete examples):

```rust
use yew::prelude::*;
use yew_oauth2::prelude::*;
use yew_oauth2::oauth2::*; // use `openid::*` when using OpenID connect

#[function_component(MyApplication)]
fn my_app() -> Html {
  let config = Config {
    client_id: "my-client".into(),
    auth_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/auth".into(),
    token_url: "https://my-sso/auth/realms/my-realm/protocol/openid-connect/token".into(),
  };

  html!(
    <OAuth2 config={config}>
      <MyApplicationMain/>
    </OAuth2>
  )
}

#[function_component(MyApplicationMain)]
fn my_app_main() -> Html {
  let agent = use_auth_agent().expect("Must be nested inside an OAuth2 component");

  let login = {
    let agent = agent.clone();
    Callback::from(move |_| {
      let _ = agent.start_login();
    })
  };
  let logout = Callback::from(move |_| {
    let _ = agent.logout();
  });

  html!(
    <>
      <Failure><FailureMessage/></Failure>
      <Authenticated>
        <button onclick={logout}>{ "Logout" }</button>
      </Authenticated>
      <NotAuthenticated>
        <button onclick={login}>{ "Login" }</button>
      </NotAuthenticated>
    </>
  )
}
```

This repository also has some complete examples:

<dl>
<dt>

[yew-oauth2-example](yew-oauth2-example/) </dt>
<dd>
A complete example, hiding everything behind a "login" page, and revealing the content once the user logged in.

Use with either OpenID Connect or OAuth2.
</dd>

<dt>

[yew-oauth2-redirect-example](yew-oauth2-redirect-example/) </dt>
<dd>
A complete example, showing the full menu structure, but redirecting the user automatically to the login server
when required.

Use with either OpenID Connect or OAuth2.
</dd>

</dl>

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

**NOTE:** It is important to use `http://localhost:8080` instead of e.g. `http://127.0.0.1:8080`, as Keycloak is
configured by default to use `http://localhost:*` as a valid redirect URL when in dev-mode. Otherwise, you will get
an "invalid redirect" error from Keycloak.
