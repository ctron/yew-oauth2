pub const STORAGE_KEY_CSRF_TOKEN: &str = "ctron/oauth2/csrfToken";
pub const STORAGE_KEY_LOGIN_STATE: &str = "ctron/oauth2/loginState";
pub const STORAGE_KEY_REDIRECT_URL: &str = "ctron/oauth2/redirectUrl";
<<<<<<< HEAD
pub const STORAGE_KEY_POST_LOGIN_URL: &str = "ctron/oauth2/postLoginUrl";
=======
pub const STORAGE_KEY_CURRENT_URL: &str = "ctron/oauth2/currentUrl";
>>>>>>> 6abf190 (Resolved conflict)

#[derive(Debug)]
pub struct State {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}
