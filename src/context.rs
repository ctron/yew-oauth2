#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OAuth2Context {
    NotInitialized,
    NotAuthenticated {
        reason: Reason,
    },
    Authenticated {
        access_token: String,
        refresh_token: Option<String>,
        expires: Option<u64>,
    },
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    NewSession,
    Expired,
    Logout,
}
