use api::auth::AuthResponse;
use dioxus::prelude::*;
use web_sys::window;

pub const AUTH_SESSION_KEY: &str = "auth_session";

#[derive(Clone, Copy, Debug)]
pub struct Auth {
    state: Signal<Option<AuthResponse>>,
}

impl Auth {
    pub fn new(state: Signal<Option<AuthResponse>>) -> Self {
        Self { state }
    }

    pub fn login(&mut self, response: AuthResponse) {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = storage.set_item(AUTH_SESSION_KEY, &json);
            }
        }
        self.state.set(Some(response));
    }

    pub fn logout(&mut self) {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item(AUTH_SESSION_KEY);
        }
        self.state.set(None);
    }

    pub fn token(&self) -> Option<String> {
        self.state.read().as_ref().map(|a| a.token.clone())
    }

    pub fn user_id(&self) -> Option<String> {
        self.state.read().as_ref().map(|a| a.user_id.clone())
    }

    pub fn username(&self) -> Option<String> {
        self.state.read().as_ref().map(|a| a.username.clone())
    }

    pub fn is_logged_in(&self) -> bool {
        self.state.read().is_some()
    }
}

pub fn use_auth() -> Auth {
    use_context::<Auth>()
}
