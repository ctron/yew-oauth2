use super::*;
use crate::context::OAuth2Context;
use std::ops::{Deref, DerefMut};
use yew::{html::Scope, Callback, Component};
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub struct OAuth2Dispatcher(Dispatcher<OAuth2Agent>);

impl OAuth2Dispatcher {
    pub fn new() -> Self {
        Self(OAuth2Agent::dispatcher())
    }
}

impl Default for OAuth2Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for OAuth2Dispatcher {
    type Target = Dispatcher<OAuth2Agent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OAuth2Dispatcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct OAuth2Bridge(Box<dyn Bridge<OAuth2Agent>>);

impl OAuth2Bridge {
    pub fn new(callback: Callback<Out>) -> OAuth2Bridge {
        Self(OAuth2Agent::bridge(callback))
    }

    pub fn from<C, F>(link: &Scope<C>, f: F) -> Self
    where
        C: Component,
        F: Fn(OAuth2Context) -> C::Message + 'static,
    {
        let callback = link.batch_callback(move |msg| match msg {
            Out::ContextUpdate(data) => vec![f(data)],
            _ => vec![],
        });
        Self::new(callback)
    }
}

impl Deref for OAuth2Bridge {
    type Target = Box<dyn Bridge<OAuth2Agent>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for OAuth2Bridge {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait OAuth2Operations {
    fn init(&mut self, config: AgentConfiguration);
    fn configure(&mut self, config: AgentConfiguration);
    fn start_login(&mut self);
    fn request_state(&mut self);
    fn logout(&mut self);
}

impl OAuth2Operations for dyn Bridge<OAuth2Agent> {
    fn init(&mut self, config: AgentConfiguration) {
        self.send(In::Init(config))
    }

    fn configure(&mut self, config: AgentConfiguration) {
        self.send(In::Configure(config))
    }

    fn start_login(&mut self) {
        self.send(In::Login)
    }

    fn request_state(&mut self) {
        self.send(In::RequestState)
    }

    fn logout(&mut self) {
        self.send(In::Logout)
    }
}
