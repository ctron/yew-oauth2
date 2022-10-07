use super::*;
use crate::{agent::Client, context::OAuth2Context};
use std::ops::{Deref, DerefMut};
use yew::{html::Scope, Callback, Component};
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

/// A [`Dispatcher`] for the OAuth2 agent, implements [`OAuth2Operations`].
pub struct OAuth2Dispatcher<C: Client>(Dispatcher<OAuth2Agent<C>>);

impl<C: Client> OAuth2Dispatcher<C> {
    pub fn new() -> Self {
        Self(OAuth2Agent::dispatcher())
    }
}

impl<C: Client> Default for OAuth2Dispatcher<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Client> Deref for OAuth2Dispatcher<C> {
    type Target = Dispatcher<OAuth2Agent<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Client> DerefMut for OAuth2Dispatcher<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A [`Bridge`] for the OAuth2 agent, implements [`OAuth2Operations`].
pub struct OAuth2Bridge<C: Client>(Box<dyn Bridge<OAuth2Agent<C>>>);

impl<C: Client> OAuth2Bridge<C> {
    pub fn new(callback: Callback<Out>) -> OAuth2Bridge<C> {
        Self(OAuth2Agent::bridge(callback))
    }

    pub fn from<COMP, F>(link: &Scope<COMP>, f: F) -> Self
    where
        COMP: Component,
        F: Fn(OAuth2Context) -> COMP::Message + 'static,
    {
        let callback = link.batch_callback(move |msg| match msg {
            Out::ContextUpdate(data) => vec![f(data)],
            _ => vec![],
        });
        Self::new(callback)
    }
}

impl<C: Client> Deref for OAuth2Bridge<C> {
    type Target = Box<dyn Bridge<OAuth2Agent<C>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<C: Client> DerefMut for OAuth2Bridge<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Operations for the OAuth2 agent
pub trait OAuth2Operations<C: Client> {
    /// Initialize the agent with a configuration.
    ///
    /// This is normally done by the [`crate::components::context::OAuth2`] context component.
    fn init(&mut self, config: AgentConfiguration<C>);
    /// Reconfigure the agent with a configuration.
    ///
    /// This is normally done by the [`crate::components::context::OAuth2`] context component.
    fn configure(&mut self, config: AgentConfiguration<C>);

    /// Start a login flow with default options.
    fn start_login(&mut self) {
        self.start_login_opts(Default::default());
    }
    /// Start a login flow.
    fn start_login_opts(&mut self, options: LoginOptions);

    /// Request the current state from the agent.
    ///
    /// The response will be sent over the link created by the bridge.
    fn request_state(&mut self);

    /// Trigger the logout with default options.
    fn logout(&mut self) {
        self.logout_opts(Default::default());
    }
    /// Trigger the logout.
    fn logout_opts(&mut self, options: LogoutOptions);
}

impl<C: Client> OAuth2Operations<C> for dyn Bridge<OAuth2Agent<C>> {
    fn init(&mut self, config: AgentConfiguration<C>) {
        self.send(In::Init(config))
    }

    fn configure(&mut self, config: AgentConfiguration<C>) {
        self.send(In::Configure(config))
    }

    fn start_login_opts(&mut self, options: LoginOptions) {
        self.send(In::Login(options))
    }

    fn request_state(&mut self) {
        self.send(In::RequestState)
    }

    fn logout_opts(&mut self, options: LogoutOptions) {
        self.send(In::Logout(options))
    }
}
