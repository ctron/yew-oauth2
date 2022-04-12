use serde::{Deserialize, Serialize};

pub mod openid {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Config {
        pub client_id: String,
        pub issuer_url: String,
    }
}

pub mod oauth2 {
    use super::*;

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
