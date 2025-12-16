use gpui::{
    actions, prelude::*, App, Context, DismissEvent, EventEmitter, FocusHandle,
    Focusable, Render, Task, Window,
};
use std::sync::Arc;
use std::time::Duration;
use ui::prelude::*;
use workspace::ModalView;

use github_client::{DeviceCodeResponse, DeviceTokenResponse, GitHubClient};

actions!(github_auth, [StartAuth, CancelAuth, CopyUserCode]);

const DEFAULT_CLIENT_ID: &str = "Ov23liPKoik1DQnfMsG7";
const POLL_INTERVAL_SECONDS: u64 = 5;

pub struct GitHubAuthModal {
    focus_handle: FocusHandle,
    github_client: Arc<GitHubClient>,
    state: AuthState,
    user_code: Option<String>,
    verification_uri: Option<String>,
    device_code: Option<String>,
    error_message: Option<String>,
    _poll_task: Option<Task<()>>,
    poll_interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Initial,
    RequestingCode,
    WaitingForUser,
    Polling,
    Success(String),
    Error,
}

pub enum GitHubAuthEvent {
    Authenticated(String),
    Cancelled,
}

impl EventEmitter<GitHubAuthEvent> for GitHubAuthModal {}
impl EventEmitter<DismissEvent> for GitHubAuthModal {}

impl GitHubAuthModal {
    pub fn new(github_client: Arc<GitHubClient>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle);

        Self {
            focus_handle,
            github_client,
            state: AuthState::Initial,
            user_code: None,
            verification_uri: None,
            device_code: None,
            error_message: None,
            _poll_task: None,
            poll_interval: Duration::from_secs(POLL_INTERVAL_SECONDS),
        }
    }

    pub fn state(&self) -> &AuthState {
        &self.state
    }

    pub fn user_code(&self) -> Option<&str> {
        self.user_code.as_deref()
    }

    pub fn verification_uri(&self) -> Option<&str> {
        self.verification_uri.as_deref()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn start_auth(&mut self, cx: &mut Context<Self>) {
        self.state = AuthState::RequestingCode;
        self.error_message = None;
        cx.notify();

        let github_client = self.github_client.clone();
        let task = cx.spawn(async move |this, cx| {
            let result = github_client.request_device_code(DEFAULT_CLIENT_ID).await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(response) => {
                        this.handle_device_code_response(response, cx);
                    }
                    Err(err) => {
                        this.state = AuthState::Error;
                        this.error_message = Some(format!("Failed to request device code: {}", err));
                        cx.notify();
                    }
                }
            }).ok();
        });

        self._poll_task = Some(task);
    }

    fn handle_device_code_response(&mut self, response: DeviceCodeResponse, cx: &mut Context<Self>) {
        let info = GitHubClient::device_code_info(&response);

        self.user_code = Some(info.user_code.clone());
        self.verification_uri = Some(info.verification_uri.clone());
        self.device_code = Some(response.device_code);
        self.poll_interval = Duration::from_secs(info.poll_interval_seconds);
        self.state = AuthState::WaitingForUser;

        cx.notify();

        if let Some(uri) = &self.verification_uri {
            cx.open_url(uri);
        }

        self.start_polling(cx);
    }

    fn start_polling(&mut self, cx: &mut Context<Self>) {
        let Some(device_code) = self.device_code.clone() else {
            return;
        };

        self.state = AuthState::Polling;
        cx.notify();

        let github_client = self.github_client.clone();
        let poll_interval = self.poll_interval;

        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(poll_interval).await;

                let result = github_client.poll_device_token(DEFAULT_CLIENT_ID, &device_code).await;

                let should_continue = this.update(cx, |this, cx| {
                    match result {
                        Ok(DeviceTokenResponse::Success { access_token }) => {
                            this.state = AuthState::Success(access_token.clone());
                            cx.emit(GitHubAuthEvent::Authenticated(access_token));
                            cx.emit(DismissEvent);
                            cx.notify();
                            false
                        }
                        Ok(DeviceTokenResponse::Pending { .. }) => {
                            true
                        }
                        Err(err) => {
                            this.state = AuthState::Error;
                            this.error_message = Some(format!("Authentication failed: {}", err));
                            cx.notify();
                            false
                        }
                    }
                }).unwrap_or(false);

                if !should_continue {
                    break;
                }
            }
        });

        self._poll_task = Some(task);
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        self._poll_task = None;
        cx.emit(GitHubAuthEvent::Cancelled);
        cx.emit(DismissEvent);
    }

    fn copy_user_code(&mut self, cx: &mut Context<Self>) {
        if let Some(code) = &self.user_code {
            cx.write_to_clipboard(code.clone().into());
        }
    }
}

impl Focusable for GitHubAuthModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for GitHubAuthModal {}

impl Render for GitHubAuthModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match &self.state {
            AuthState::Initial => {
                v_flex()
                    .gap_4()
                    .child(Label::new("Sign in to GitHub").size(LabelSize::Large))
                    .child(Label::new("Click below to start the authentication process"))
                    .child(
                        Button::new("start-auth", "Start Authentication")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.start_auth(cx);
                            })),
                    )
            }
            AuthState::RequestingCode => {
                v_flex()
                    .gap_4()
                    .child(Label::new("Requesting device code...").color(Color::Muted))
            }
            AuthState::WaitingForUser | AuthState::Polling => {
                let user_code = self.user_code.as_deref().unwrap_or("Loading...").to_string();
                let verification_uri = self.verification_uri.clone().unwrap_or_default();

                v_flex()
                    .gap_4()
                    .child(Label::new("GitHub Authentication").size(LabelSize::Large))
                    .child(Label::new("Copy the code below and paste it on GitHub:"))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(cx.theme().colors().editor_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border)
                                    .rounded_md()
                                    .child(Label::new(user_code).size(LabelSize::Large))
                            )
                            .child(
                                Button::new("copy-code", "Copy")
                                    .style(ButtonStyle::Subtle)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.copy_user_code(cx);
                                    })),
                            )
                    )
                    .child(Label::new(format!("Verification URL: {}", verification_uri)).color(Color::Muted))
                    .when(matches!(self.state, AuthState::Polling), |this| {
                        this.child(Label::new("Waiting for authorization...").color(Color::Muted))
                    })
            }
            AuthState::Success(_) => {
                v_flex()
                    .gap_4()
                    .child(Label::new("Authentication Successful!").size(LabelSize::Large).color(Color::Success))
                    .child(Label::new("You can now close this window"))
            }
            AuthState::Error => {
                let error = self.error_message.clone().unwrap_or_else(|| "Unknown error".to_string());
                v_flex()
                    .gap_4()
                    .child(Label::new("Authentication Failed").size(LabelSize::Large).color(Color::Error))
                    .child(Label::new(error).color(Color::Error))
                    .child(
                        Button::new("retry", "Try Again")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.state = AuthState::Initial;
                                this.error_message = None;
                                this.user_code = None;
                                this.verification_uri = None;
                                this.device_code = None;
                                cx.notify();
                            })),
                    )
            }
        };

        v_flex()
            .id("github-auth-modal")
            .track_focus(&self.focus_handle)
            .elevation_3(cx)
            .p_4()
            .gap_4()
            .w(px(500.0))
            .max_w(px(600.0))
            .child(content)
            .when(!matches!(self.state, AuthState::Success(_)), |this| {
                this.child(
                    h_flex()
                        .justify_end()
                        .child(
                            Button::new("cancel", "Cancel")
                                .style(ButtonStyle::Subtle)
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.cancel(cx);
                                })),
                        )
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::{DeviceCodeResponse, DeviceTokenResponse, GitHubClient};
    use gpui::TestAppContext;
    use http_client::FakeHttpClient;
    use std::sync::Arc;

    #[gpui::test]
    async fn test_auth_modal_creation(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));

        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.read_with(cx, |modal, _cx| {
            assert!(matches!(modal.state, AuthState::Initial));
            assert!(modal.user_code.is_none());
            assert!(modal.verification_uri.is_none());
            assert!(modal.error_message.is_none());
        });
    }

    #[gpui::test]
    async fn test_start_auth_requests_device_code(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|request| async move {
            assert!(request.uri().to_string().contains("/device/code"));

            let response = DeviceCodeResponse {
                device_code: "device123".to_string(),
                user_code: "ABCD-1234".to_string(),
                verification_uri: "https://github.com/login/device".to_string(),
                expires_in: 900,
                interval: 5,
            };

            let body = serde_json::to_vec(&response).expect("serialize response");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, cx| {
            modal.start_auth(cx);
            assert!(matches!(modal.state, AuthState::RequestingCode));
        });

        cx.run_until_parked();

        modal.read_with(cx, |modal, _cx| {
            assert!(matches!(modal.state, AuthState::Polling));
            assert_eq!(modal.user_code.as_deref(), Some("ABCD-1234"));
            assert_eq!(modal.verification_uri.as_deref(), Some("https://github.com/login/device"));
            assert_eq!(modal.device_code.as_deref(), Some("device123"));
        });
    }

    #[gpui::test]
    async fn test_start_auth_handles_error(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(500)
                .body("Server Error".as_bytes().to_vec().into())
                .expect("build response"))
        });

        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, cx| {
            modal.start_auth(cx);
        });

        cx.run_until_parked();

        modal.read_with(cx, |modal, _cx| {
            assert!(matches!(modal.state, AuthState::Error));
            assert!(modal.error_message.is_some());
        });
    }

    #[gpui::test]
    async fn test_polling_success(cx: &mut TestAppContext) {
        let mut call_count = 0;
        let http_client = FakeHttpClient::create(move |request| {
            let current_call = call_count;
            call_count += 1;

            async move {
                if request.uri().to_string().contains("/device/code") {
                    let response = DeviceCodeResponse {
                        device_code: "device123".to_string(),
                        user_code: "ABCD-1234".to_string(),
                        verification_uri: "https://github.com/login/device".to_string(),
                        expires_in: 900,
                        interval: 1,
                    };
                    let body = serde_json::to_vec(&response).expect("serialize response");
                    Ok(http::Response::builder()
                        .status(200)
                        .body(body.into())
                        .expect("build response"))
                } else {
                    let response = if current_call < 2 {
                        DeviceTokenResponse::Pending {
                            error: "authorization_pending".to_string(),
                        }
                    } else {
                        DeviceTokenResponse::Success {
                            access_token: "gho_test_token".to_string(),
                        }
                    };
                    let body = serde_json::to_vec(&response).expect("serialize response");
                    Ok(http::Response::builder()
                        .status(200)
                        .body(body.into())
                        .expect("build response"))
                }
            }
        });

        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| {
            let mut modal = GitHubAuthModal::new(github_client, window, cx);
            modal.poll_interval = Duration::from_millis(100);
            modal
        });

        modal.update(cx, |modal, cx| {
            modal.start_auth(cx);
        });

        cx.run_until_parked();
        cx.background_executor.advance_clock(Duration::from_millis(150));
        cx.run_until_parked();
        cx.background_executor.advance_clock(Duration::from_millis(150));
        cx.run_until_parked();

        modal.read_with(cx, |modal, _cx| {
            if let AuthState::Success(token) = &modal.state {
                assert_eq!(token, "gho_test_token");
            } else {
                panic!("Expected Success state, got {:?}", modal.state);
            }
        });
    }

    #[gpui::test]
    async fn test_polling_error(cx: &mut TestAppContext) {
        let mut call_count = 0;
        let http_client = FakeHttpClient::create(move |request| {
            let current_call = call_count;
            call_count += 1;

            async move {
                if request.uri().to_string().contains("/device/code") {
                    let response = DeviceCodeResponse {
                        device_code: "device123".to_string(),
                        user_code: "ABCD-1234".to_string(),
                        verification_uri: "https://github.com/login/device".to_string(),
                        expires_in: 900,
                        interval: 1,
                    };
                    let body = serde_json::to_vec(&response).expect("serialize response");
                    Ok(http::Response::builder()
                        .status(200)
                        .body(body.into())
                        .expect("build response"))
                } else if current_call < 2 {
                    let response = DeviceTokenResponse::Pending {
                        error: "authorization_pending".to_string(),
                    };
                    let body = serde_json::to_vec(&response).expect("serialize response");
                    Ok(http::Response::builder()
                        .status(200)
                        .body(body.into())
                        .expect("build response"))
                } else {
                    Ok(http::Response::builder()
                        .status(403)
                        .body("Access denied".as_bytes().to_vec().into())
                        .expect("build response"))
                }
            }
        });

        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| {
            let mut modal = GitHubAuthModal::new(github_client, window, cx);
            modal.poll_interval = Duration::from_millis(100);
            modal
        });

        modal.update(cx, |modal, cx| {
            modal.start_auth(cx);
        });

        cx.run_until_parked();
        cx.background_executor.advance_clock(Duration::from_millis(150));
        cx.run_until_parked();
        cx.background_executor.advance_clock(Duration::from_millis(150));
        cx.run_until_parked();

        modal.read_with(cx, |modal, _cx| {
            assert!(matches!(modal.state, AuthState::Error));
            assert!(modal.error_message.is_some());
        });
    }

    #[gpui::test]
    async fn test_cancel_emits_cancelled_event(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        let mut cancelled = false;
        cx.subscribe(&modal, |_modal, event, _cx| {
            if matches!(event, GitHubAuthEvent::Cancelled) {
                cancelled = true;
            }
        }).detach();

        modal.update(cx, |modal, cx| {
            modal.cancel(cx);
        });

        assert!(cancelled);
    }

    #[gpui::test]
    async fn test_success_emits_authenticated_event(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|request| async move {
            if request.uri().to_string().contains("/device/code") {
                let response = DeviceCodeResponse {
                    device_code: "device123".to_string(),
                    user_code: "ABCD-1234".to_string(),
                    verification_uri: "https://github.com/login/device".to_string(),
                    expires_in: 900,
                    interval: 1,
                };
                let body = serde_json::to_vec(&response).expect("serialize response");
                Ok(http::Response::builder()
                    .status(200)
                    .body(body.into())
                    .expect("build response"))
            } else {
                let response = DeviceTokenResponse::Success {
                    access_token: "gho_test_token".to_string(),
                };
                let body = serde_json::to_vec(&response).expect("serialize response");
                Ok(http::Response::builder()
                    .status(200)
                    .body(body.into())
                    .expect("build response"))
            }
        });

        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| {
            let mut modal = GitHubAuthModal::new(github_client, window, cx);
            modal.poll_interval = Duration::from_millis(100);
            modal
        });

        let mut received_token = None;
        cx.subscribe(&modal, |_modal, event, _cx| {
            if let GitHubAuthEvent::Authenticated(token) = event {
                received_token = Some(token.clone());
            }
        }).detach();

        modal.update(cx, |modal, cx| {
            modal.start_auth(cx);
        });

        cx.run_until_parked();
        cx.background_executor.advance_clock(Duration::from_millis(150));
        cx.run_until_parked();

        assert_eq!(received_token, Some("gho_test_token".to_string()));
    }

    #[gpui::test]
    async fn test_handle_device_code_response_sets_fields(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        let response = DeviceCodeResponse {
            device_code: "device123".to_string(),
            user_code: "TEST-CODE".to_string(),
            verification_uri: "https://github.com/login/device".to_string(),
            expires_in: 900,
            interval: 10,
        };

        modal.update(cx, |modal, cx| {
            modal.handle_device_code_response(response, cx);
        });

        cx.run_until_parked();

        modal.read_with(cx, |modal, _cx| {
            assert_eq!(modal.user_code.as_deref(), Some("TEST-CODE"));
            assert_eq!(modal.verification_uri.as_deref(), Some("https://github.com/login/device"));
            assert_eq!(modal.device_code.as_deref(), Some("device123"));
            assert_eq!(modal.poll_interval, Duration::from_secs(10));
        });
    }

    #[gpui::test]
    async fn test_copy_user_code(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, cx| {
            modal.user_code = Some("TEST-1234".to_string());
            modal.copy_user_code(cx);
        });
    }

    #[gpui::test]
    async fn test_state_accessor(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.read_with(cx, |modal, _cx| {
            assert!(matches!(modal.state(), &AuthState::Initial));
        });
    }

    #[gpui::test]
    async fn test_user_code_accessor(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, _cx| {
            modal.user_code = Some("ABCD-1234".to_string());
        });

        modal.read_with(cx, |modal, _cx| {
            assert_eq!(modal.user_code(), Some("ABCD-1234"));
        });
    }

    #[gpui::test]
    async fn test_verification_uri_accessor(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, _cx| {
            modal.verification_uri = Some("https://github.com/login/device".to_string());
        });

        modal.read_with(cx, |modal, _cx| {
            assert_eq!(modal.verification_uri(), Some("https://github.com/login/device"));
        });
    }

    #[gpui::test]
    async fn test_error_message_accessor(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::new(http_client));
        let modal = cx.new(|window, cx| GitHubAuthModal::new(github_client, window, cx));

        modal.update(cx, |modal, _cx| {
            modal.error_message = Some("Test error".to_string());
        });

        modal.read_with(cx, |modal, _cx| {
            assert_eq!(modal.error_message(), Some("Test error"));
        });
    }
}
