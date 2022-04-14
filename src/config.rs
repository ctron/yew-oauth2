use serde::{Deserialize, Serialize};

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
    }
}

pub mod oauth2 {
    use super::*;

    /// Plain OAuth2 client configuration
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        pub client_id: String,
        pub auth_url: String,
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
