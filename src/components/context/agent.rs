use crate::agent::{self, Client};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use yew::hook;

/// A wrapper for the [`agent::Agent`].
///
/// Required as Yew has some requirements for the type of a context, like [`PartialEq`].
#[derive(Clone, Debug)]
pub struct Agent<C: Client>(agent::Agent<C>, usize);

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<C: Client> Agent<C> {
    pub fn new(agent: agent::Agent<C>) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::AcqRel);

        Self(agent, id)
    }
}

impl<C: Client> PartialEq for Agent<C> {
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl<C: Client> Deref for Agent<C> {
    type Target = agent::Agent<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Client> DerefMut for Agent<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Get the authentication agent.
#[hook]
pub fn use_auth_agent<C>() -> Option<Agent<C>>
where
    C: Client,
{
    yew::prelude::use_context()
}
