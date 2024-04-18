//! Configuration

use serde::{Deserialize, Serialize};

/// Configuration for OpenID Connect
pub mod openid {
    use super::*;

    /// OpenID Connect client configuration
    ///
    /// ## Non-exhaustive
    ///
    /// This struct is `#[non_exhaustive]`, so it is not possible to directly create a struct, creating a new struct
    /// is done using the [`Config::new`] function. Additional properties are set using the `with_*` functions.
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
        /// However, e.g. older Keycloak instances, require this to be `redirect_uri`.
        pub post_logout_redirect_name: Option<String>,
        /// Additional audiences of the ID token which are considered trustworthy.
        ///
        /// Those audiences are allowed in addition to the client ID.
        pub additional_trusted_audiences: Vec<String>,
    }

    impl Config {
        /// Create a new configuration
        pub fn new(client_id: impl Into<String>, issuer_url: impl Into<String>) -> Self {
            Self {
                client_id: client_id.into(),
                issuer_url: issuer_url.into(),

                end_session_url: None,
                after_logout_url: None,
                post_logout_redirect_name: None,
                additional_trusted_audiences: vec![],
            }
        }

        /// Set an override for the URL for ending the session.
        pub fn with_end_session_url(mut self, end_session_url: impl Into<String>) -> Self {
            self.end_session_url = Some(end_session_url.into());
            self
        }

        /// Set the URL the issuer should redirect to after the logout
        pub fn with_after_logout_url(mut self, after_logout_url: impl Into<String>) -> Self {
            self.after_logout_url = Some(after_logout_url.into());
            self
        }

        /// Set the name of the post logout redirect query parameter
        pub fn with_post_logout_redirect_name(
            mut self,
            post_logout_redirect_name: impl Into<String>,
        ) -> Self {
            self.post_logout_redirect_name = Some(post_logout_redirect_name.into());
            self
        }

        /// Set the additionally trusted audiences
        pub fn with_additional_trusted_audiences(
            mut self,
            additional_trusted_audiences: impl IntoIterator<Item = impl Into<String>>,
        ) -> Self {
            self.additional_trusted_audiences = additional_trusted_audiences
                .into_iter()
                .map(|s| s.into())
                .collect();
            self
        }

        /// Extend the additionally trusted audiences.
        pub fn extend_additional_trusted_audiences(
            mut self,
            additional_trusted_audiences: impl IntoIterator<Item = impl Into<String>>,
        ) -> Self {
            self.additional_trusted_audiences
                .extend(additional_trusted_audiences.into_iter().map(|s| s.into()));
            self
        }

        /// Add an additionally trusted audience.
        pub fn add_additional_trusted_audience(
            mut self,
            additional_trusted_audience: impl Into<String>,
        ) -> Self {
            self.additional_trusted_audiences
                .push(additional_trusted_audience.into());
            self
        }
    }
}

/// Configuration for OAuth2
pub mod oauth2 {
    use super::*;

    /// Plain OAuth2 client configuration
    ///
    /// ## Non-exhaustive
    ///
    /// This struct is `#[non_exhaustive]`, so it is not possible to directly create a struct, creating a new struct
    /// is done using the [`crate::openid::Config::new`] function.
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
        /// Create a new configuration
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
