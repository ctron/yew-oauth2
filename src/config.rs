use serde::{Deserialize, Serialize};
use yew::html::IntoPropValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OAuth2Configuration {
    Provided(Config),
    // Discovered{ base_url },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub auth_url: String,
    pub token_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

impl Config {
    pub fn new<C, A, T, S, I>(client_id: C, auth_url: A, token_url: T, scopes: S) -> Self
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
            scopes: scopes.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl IntoPropValue<OAuth2Configuration> for Config {
    fn into_prop_value(self) -> OAuth2Configuration {
        OAuth2Configuration::Provided(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycloakConfig {
    pub client_id: String,
    pub url: String,
    pub realm: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scopes: String,
}

impl IntoPropValue<OAuth2Configuration> for KeycloakConfig {
    fn into_prop_value(self) -> OAuth2Configuration {
        OAuth2Configuration::Provided(Config {
            client_id: self.client_id,
            scopes: split_scopes(self.scopes),
            auth_url: format!("{}/protocol/openid-connect/auth", self.realm),
            token_url: format!("{}/protocol/openid-connect/token", self.realm),
        })
    }
}

fn split_scopes(scopes: String) -> Vec<String> {
    scopes
        .split(' ')
        .map(|s| s.trim().to_string())
        .filter(String::is_empty)
        .collect()
}
