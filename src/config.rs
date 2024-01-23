//! Configuration

use serde::{Deserialize, Serialize};

/// Configuration for OpenID Connect
pub mod openid {
    use super::*;

    /// OpenID Connect client configuration
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        /// The client ID
        pub client_id: String,
        /// The OpenID connect issuer URL.
        pub issuer_url: String,
        #[serde(default)]
        /// Additional, non-required configuration, with a default.
        pub additional: Additional,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Additional {
        /// An override for the end session URL.
        pub end_session_url: Option<String>,
        /// The URL to navigate to after the logout has been completed.
        pub after_logout_url: Option<String>,
        /// The name of the query parameter for the post logout redirect.
        ///
        /// The defaults to `post_logout_redirect_uri` for OpenID RP initiated logout.
        /// However, e.g. older Keycloak instances require this to be `redirect_uri`.
        pub post_logout_redirect_name: Option<String>,
        pub valid_audiences: Option<Vec<String>>,
    }
}

/// Configuration for OAuth2
pub mod oauth2 {
    use super::*;

    /// Plain OAuth2 client configuration
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
        pub fn new<C, A, T, S, I>(client_id: C, auth_url: A, token_url: T) -> Self
        where
            C: Into<String>,
            A: Into<String>,
            T: Into<String>,
            S: IntoIterator<Item = I>,
            I: Into<String>,
        {
            Self {
                client_id: client_id.into(),
                auth_url: auth_url.into(),
                token_url: token_url.into(),
            }
        }
    }
}
