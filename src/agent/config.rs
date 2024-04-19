use super::{LoginOptions, LogoutOptions};
use crate::agent::Client;
use std::time::Duration;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct AgentConfiguration<C: Client> {
    pub config: C::Configuration,
    pub scopes: Vec<String>,
    pub grace_period: Duration,
    pub audience: Option<String>,
    pub max_expiration: Option<Duration>,

    pub default_login_options: Option<LoginOptions>,
    pub default_logout_options: Option<LogoutOptions>,
}

impl<C: Client> PartialEq for AgentConfiguration<C> {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.scopes == other.scopes
            && self.grace_period == other.grace_period
            && self.audience == other.audience
    }
}

impl<C: Client> Eq for AgentConfiguration<C> {}
