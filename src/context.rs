use yew::{context::ContextHandle, html::Scope, prelude::*};

#[cfg(feature = "openid")]
pub type Claims = openidconnect::IdTokenClaims<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
>;

#[derive(Clone, Debug, PartialEq)]
pub enum OAuth2Context {
    /// The agent is not initialized yet.
    NotInitialized,
    /// Not authenticated.
    NotAuthenticated {
        /// Reason why it is not authenticated.
        reason: Reason,
    },
    /// Session is authenticated.
    Authenticated {
        /// The access token
        access_token: String,
        /// An optional refresh token
        refresh_token: Option<String>,
        /// OpenID claims
        #[cfg(feature = "openid")]
        claims: Option<std::rc::Rc<Claims>>,
        expires: Option<u64>,
    },
    /// Something failed.
    Failed(String),
}

impl OAuth2Context {
    /// Get the access token, if the context is [`OAuth2Context::Authenticated`]
    pub fn access_token(&self) -> Option<&str> {
        match self {
            Self::Authenticated { access_token, .. } => Some(&access_token),
            _ => None,
        }
    }

    /// Get the claims, if the context is [`OAuth2Context::Authenticated`]
    #[cfg(feature = "openid")]
    pub fn claims(&self) -> Option<&Claims> {
        match self {
            Self::Authenticated {
                claims: Some(claims),
                ..
            } => Some(claims),
            _ => None,
        }
    }
}

/// The reason why the context is un-authenticated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    NewSession,
    Expired,
    Logout,
}

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

pub trait AsContextValue {
    type Message;

    fn as_value<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static;
}

impl<C> AsContextValue for Scope<C>
where
    C: Component,
{
    type Message = C::Message;

    fn as_value<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static,
    {
        self.context::<T>(self.callback(f)).into()
    }
}

impl<C> AsContextValue for Context<C>
where
    C: Component,
{
    type Message = C::Message;

    fn as_value<T, F>(&self, f: F) -> ContextValue<T>
    where
        T: 'static + Clone + PartialEq,
        F: Fn(T) -> Self::Message + 'static,
    {
        self.link().as_value(f)
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
}
