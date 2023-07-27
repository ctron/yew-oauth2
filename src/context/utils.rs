use super::*;
use yew::{context::ContextHandle, html::Scope, prelude::*};

/// Helper to get an unzipped version of the context.
pub trait UnzippedWith {
    fn unzipped_with(
        &self,
        callback: Callback<OAuth2Context>,
    ) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>);
}

/// Helper to get an unzipped version of the context.
pub trait Unzipped {
    type Message;

    fn unzipped<F>(&self, f: F) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>)
    where
        F: Fn(OAuth2Context) -> Self::Message + 'static;
}

impl<C> UnzippedWith for Context<C>
where
    C: Component,
{
    fn unzipped_with(
        &self,
        callback: Callback<OAuth2Context>,
    ) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>) {
        self.link().unzipped_with(callback)
    }
}

impl<C> UnzippedWith for Scope<C>
where
    C: Component,
{
    fn unzipped_with(
        &self,
        callback: Callback<OAuth2Context>,
    ) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>) {
        match self.context(callback) {
            Some((auth, handle)) => (Some(auth), Some(handle)),
            None => (None, None),
        }
    }
}

impl<C> Unzipped for Context<C>
where
    C: Component,
{
    type Message = C::Message;

    fn unzipped<F>(&self, f: F) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>)
    where
        F: Fn(OAuth2Context) -> Self::Message + 'static,
    {
        self.link().unzipped(f)
    }
}

impl<C> Unzipped for Scope<C>
where
    C: Component,
{
    type Message = C::Message;

    fn unzipped<F>(&self, f: F) -> (Option<OAuth2Context>, Option<ContextHandle<OAuth2Context>>)
    where
        F: Fn(OAuth2Context) -> Self::Message + 'static,
    {
        self.unzipped_with(self.callback(f))
    }
}

/// Functional component for using the context.
pub trait UseContext {
    type Message;

    fn use_context<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static;
}

impl<C> UseContext for Scope<C>
where
    C: Component,
{
    type Message = C::Message;

    fn use_context<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static,
    {
        self.context::<T>(self.callback(f)).into()
    }
}

impl<C> UseContext for Context<C>
where
    C: Component,
{
    type Message = C::Message;

    fn use_context<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static,
    {
        self.link().use_context(f)
    }
}

/// A helper which holds both value and handle in a way that it can easily be updated if it
/// is present.
pub enum ContextValue<T>
where
    T: 'static + Clone + PartialEq,
{
    Some(T, ContextHandle<T>),
    None,
}

impl<T> From<Option<(T, ContextHandle<T>)>> for ContextValue<T>
where
    T: 'static + Clone + PartialEq,
{
    fn from(value: Option<(T, ContextHandle<T>)>) -> Self {
        match value {
            Some(value) => Self::Some(value.0, value.1),
            None => Self::None,
        }
    }
}

impl<T> ContextValue<T>
where
    T: 'static + Clone + PartialEq,
{
    /// Set a new value, only if the handle is present.
    pub fn set(&mut self, new_value: T) {
        match self {
            Self::Some(value, _) => *value = new_value,
            Self::None => {}
        }
    }

    /// Get the current value.
    pub fn get(&self) -> Option<&T> {
        match &self {
            Self::Some(value, _) => Some(value),
            Self::None => None,
        }
    }

    pub fn as_ref(&self) -> Option<&T> {
        self.get()
    }
}
