use super::{AgentConfiguration, Client, LoginOptions, LogoutOptions};
use std::fmt::{Display, Formatter};

/// Operation error
#[derive(Clone, Debug)]
pub enum Error {
    /// The agent cannot be reached.
    NoAgent,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAgent => write!(f, "no agent"),
        }
    }
}

impl std::error::Error for Error {}

/// Operations for the OAuth2 agent
pub trait OAuth2Operations<C: Client> {
    /// Configure the agent with a configuration.
    ///
    /// This is normally done by the [`crate::components::context::OAuth2`] context component.
    fn configure(&self, config: AgentConfiguration<C>) -> Result<(), Error>;

    /// Start a login flow with default options.
    fn start_login(&self) -> Result<(), Error>;

    /// Start a login flow.
    fn start_login_opts(&self, options: LoginOptions) -> Result<(), Error>;

    /// Trigger the logout with default options.
    fn logout(&self) -> Result<(), Error>;

    /// Trigger the logout.
    fn logout_opts(&self, options: LogoutOptions) -> Result<(), Error>;
}
