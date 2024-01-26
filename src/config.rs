//! Configuration

use serde::{Deserialize, Serialize};

/// Configuration for OpenID Connect
pub mod openid {
    use super::*;

    /// OpenID Connect client configuration
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        /// The client ID
        pub client_id: String,
        /// The OpenID connect issuer URL.
        pub issuer_url: String,
        /// An override for the end session URL.
        pub end_session_url: Option<String>,
        /// The URL to navigate to after the logout has been completed.
        pub after_logout_url: Option<String>,
        /// The name of the query parameter for the post logout redirect.
        ///
        /// The defaults to `post_logout_redirect_uri` for OpenID RP initiated logout.
        /// However, e.g. older Keycloak instances require this to be `redirect_uri`.
        pub post_logout_redirect_name: Option<String>,
        /// Additional audiences of the ID token which are considered trustworthy.
        ///
        /// Those audiences are allowed in addition to the client ID.
        pub additional_trusted_audiences: Vec<String>,
    }

    impl Config {}
}

/// Configuration for OAuth2
pub mod oauth2 {
    use super::*;

    /// Plain OAuth2 client configuration
    ///
    /// ## Non-exhaustive
    ///
    /// This struct is `#[non_exhaustive]`, so it is not possible to directly create a struct. You can do this using
    /// the [`Config::new`] function.
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        /// The client ID
        pub client_id: String,
        /// The authentication URL
        pub auth_url: String,
        /// The token exchange URL
        pub token_url: String,
    }

    impl Config {
        pub fn new(
            client_id: impl Into<String>,
            auth_url: impl Into<String>,
            token_url: impl Into<String>,
        ) -> Self {
            Self {
                client_id: client_id.into(),
                auth_url: auth_url.into(),
                token_url: token_url.into(),
            }
        }
    }
}
