#[cfg(target_arch = "wasm32")]
use api::auth::AuthResponse;
use api::refresh_token;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use ui::{Auth, AUTH_SESSION_KEY};
use web_sys::window;

pub fn use_auth() -> Auth {
    use_context::<Auth>()
}

#[component]
pub fn AuthProvider(children: Element) -> Element {
    let mut auth_state = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(Some(auth_json)) = storage.get_item(AUTH_SESSION_KEY) {
                if let Ok(auth) = serde_json::from_str::<AuthResponse>(&auth_json) {
                    return Some(auth);
                }
            }
        }
        None
    });

    use_context_provider(|| Auth::new(auth_state));

    // Token refresh logic
    use_future(move || async move {
        loop {
            let current_auth = auth_state.read().clone();

            if let Some(auth) = current_auth {
                let now = chrono::Utc::now().timestamp();
                let expires_at = auth.expires_at;
                let buffer = 300; // 5 minutes buffer

                let delay_seconds = expires_at - now - buffer;
                let delay_ms = if delay_seconds > 0 {
                    delay_seconds * 1000
                } else {
                    0
                };

                TimeoutFuture::new(delay_ms as u32).await;

                // Check if we are still logged in before refreshing
                if auth_state.read().is_none() {
                    continue;
                }

                match refresh_token(auth.refresh_token.clone()).await {
                    Ok(new_auth) => {
                        if let Some(storage) =
                            window().and_then(|w| w.local_storage().ok().flatten())
                        {
                            if let Ok(auth_json) = serde_json::to_string(&new_auth) {
                                let _ = storage.set_item(AUTH_SESSION_KEY, &auth_json);
                            }
                        }
                        auth_state.set(Some(new_auth));
                    }
                    Err(e) => {
                        tracing::error!("Failed to refresh token: {}", e);
                        // If refresh fails, we might want to logout or just retry later?
                        // For security, if refresh fails (token revoked?), logout.
                        if let Some(storage) =
                            window().and_then(|w| w.local_storage().ok().flatten())
                        {
                            let _ = storage.remove_item(AUTH_SESSION_KEY);
                        }
                        auth_state.set(None);
                    }
                }
            } else {
                // Check again in 1 second if logged out to see if status changed
                // (Though use_future will only re-run if dependencies change, but here we loop)
                // actually, we should just wait for a signal change?
                // But in this loop structure, just polling is simple.
                TimeoutFuture::new(1000).await;
            }
        }
    });

    rsx! {
        {children}
    }
}
