use gpui::{
    actions, uniform_list, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle,
    Focusable, Pixels, Render, Task, UniformListScrollHandle, Window, px,
};
use panel::PanelHeader;
use ui::{prelude::*, Tooltip};
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    Workspace,
};
use github_client::{GitHubClient, PRState};
use std::sync::Arc;

use crate::{CheckoutPullRequest, CreatePullRequest, MergePullRequest, TogglePRPanel, PullRequestData, PullRequestState, GitHubRepo};

actions!(pr_panel, [Refresh, Close, SignInToGitHub]);

pub struct PRPanel {
    focus_handle: FocusHandle,
    width: Option<Pixels>,
    github_client: Option<Arc<GitHubClient>>,
    github_repo: Option<GitHubRepo>,
    pull_requests: Vec<PullRequestData>,
    loading: bool,
    error: Option<String>,
    _load_task: Option<Task<()>>,
    selected_index: Option<usize>,
    scroll_handle: UniformListScrollHandle,
}

impl PRPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            width: None,
            github_client: None,
            github_repo: None,
            pull_requests: Vec::new(),
            loading: false,
            error: None,
            _load_task: None,
            selected_index: None,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    pub fn load(
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let http_client = workspace.read(cx).client().http_client();

        let panel = cx.new(|cx| {
            let mut panel = Self::new(cx);
            panel.github_client = Some(Arc::new(GitHubClient::new(http_client)));
            panel
        });

        workspace.update(cx, |workspace, cx| {
            workspace.add_panel(panel.clone(), window, cx);
        });

        panel
    }

    pub fn set_github_token(&mut self, token: String, cx: &mut Context<Self>) {
        if let Some(client) = &self.github_client {
            let http_client = client.http_client();
            self.github_client = Some(Arc::new(GitHubClient::with_token(http_client, token)));
        }
        cx.notify();
    }

    pub fn has_github_token(&self) -> bool {
        self.github_client
            .as_ref()
            .and_then(|client| client.token())
            .is_some()
    }

    pub fn on_repo_detected(&mut self, repo: GitHubRepo, cx: &mut Context<Self>) {
        self.github_repo = Some(repo);
        if self.has_github_token() {
            self.refresh_pull_requests(cx);
        }
    }

    pub fn refresh_pull_requests(&mut self, cx: &mut Context<Self>) {
        if let Some(repo) = &self.github_repo {
            let owner = repo.owner.clone();
            let repo_name = repo.repo.clone();
            self.load_pull_requests(&owner, &repo_name, cx);
        }
    }

    pub fn load_pull_requests(&mut self, owner: &str, repo: &str, cx: &mut Context<Self>) {
        let Some(github_client) = self.github_client.clone() else {
            self.error = Some("GitHub client not configured".to_string());
            cx.notify();
            return;
        };

        self.loading = true;
        self.error = None;
        cx.notify();

        let owner = owner.to_string();
        let repo = repo.to_string();

        let task = cx.spawn(async move |this, cx| {
            let result = github_client.list_pull_requests(&owner, &repo, PRState::Open).await;

            this.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(prs) => {
                        this.pull_requests = prs.into_iter().map(|pr| {
                            let state = match pr.state {
                                PRState::Open => PullRequestState::Open,
                                PRState::Closed => {
                                    PullRequestState::Closed
                                }
                                PRState::All => PullRequestState::Open,
                            };
                            PullRequestData {
                                number: pr.number,
                                title: pr.title,
                                author: pr.author.login,
                                state,
                            }
                        }).collect();
                        this.error = None;
                    }
                    Err(err) => {
                        this.error = Some(err.to_string());
                    }
                }
                cx.notify();
            }).ok();
        });

        self._load_task = Some(task);
    }
}

pub fn register(workspace: &mut Workspace, _cx: &mut Context<Workspace>) {
    workspace.register_action(|workspace, _: &TogglePRPanel, window, cx| {
        workspace.toggle_panel_focus::<PRPanel>(window, cx);
    });

    workspace.register_action(|workspace, _: &SignInToGitHub, window, cx| {
        use crate::GitHubAuthModal;

        let Some(panel) = workspace.panel::<PRPanel>(cx) else {
            return;
        };

        let github_client = panel.read_with(cx, |panel, _cx| {
            panel.github_client.clone()
        });

        if let Some(github_client) = github_client {
            workspace.toggle_modal(window, cx, |window, cx| {
                GitHubAuthModal::new(github_client.clone(), window, cx)
            });
        }
    });

    workspace.register_action(|workspace, _: &CheckoutPullRequest, _window, cx| {
        let Some(panel) = workspace.panel::<PRPanel>(cx) else {
            return;
        };

        let pr_data = panel.read_with(cx, |panel, _cx| {
            let selected = panel.selected_index?;
            let pr = panel.pull_requests.get(selected)?;
            Some((pr.number, pr.title.clone()))
        });

        if let Some((pr_number, head_ref)) = pr_data {
            use crate::pr_checkout::{checkout_pull_request, CheckoutPullRequestParams};

            let workspace_weak = workspace.weak_handle();
            let params = CheckoutPullRequestParams {
                pr_number,
                head_ref,
            };

            cx.spawn(async move |_workspace, cx| {
                checkout_pull_request(workspace_weak, params, cx).await
            }).detach();
        }
    });

    workspace.register_action(|workspace, _: &CreatePullRequest, window, cx| {
        use crate::pr_create_modal::CreatePRModal;

        let Some(panel) = workspace.panel::<PRPanel>(cx) else {
            return;
        };

        let github_client = panel.read_with(cx, |panel, _cx| {
            panel.github_client.clone()
        });

        if let Some(github_client) = github_client {
            let project = workspace.project().clone();
            workspace.toggle_modal(window, cx, |window, cx| {
                CreatePRModal::new(
                    "main".to_string(),
                    "feature".to_string(),
                    github_client,
                    "owner".to_string(),
                    "repo".to_string(),
                    project,
                    window,
                    cx,
                )
            });
        }
    });

    workspace.register_action(|workspace, _: &MergePullRequest, window, cx| {
        use crate::pr_merge_modal::MergePRModal;

        let Some(panel) = workspace.panel::<PRPanel>(cx) else {
            return;
        };

        let data = panel.read_with(cx, |panel, _cx| {
            let selected = panel.selected_index?;
            let pr = panel.pull_requests.get(selected)?;
            let github_client = panel.github_client.clone()?;
            Some((pr.number, pr.title.clone(), github_client))
        });

        if let Some((pr_number, pr_title, github_client)) = data {
            workspace.toggle_modal(window, cx, |window, cx| {
                MergePRModal::new(
                    pr_number,
                    pr_title,
                    github_client,
                    "owner".to_string(),
                    "repo".to_string(),
                    window,
                    cx,
                )
            });
        }
    });
}

impl EventEmitter<PanelEvent> for PRPanel {}
impl EventEmitter<DismissEvent> for PRPanel {}

impl Focusable for PRPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PRPanel {
    fn persistent_name() -> &'static str {
        "PRPanel"
    }

    fn panel_key() -> &'static str {
        "pr_panel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn size(&self, _window: &Window, _cx: &App) -> Pixels {
        self.width.unwrap_or(px(360.0))
    }

    fn set_size(&mut self, size: Option<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.width = size;
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::PullRequest)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Pull Requests")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(TogglePRPanel)
    }

    fn activation_priority(&self) -> u32 {
        3
    }
}

impl PanelHeader for PRPanel {}

impl Render for PRPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::PRListItem;

        let content = if !self.has_github_token() {
            v_flex()
                .flex_1()
                .justify_center()
                .items_center()
                .gap_4()
                .child(Label::new("Sign in to GitHub to view pull requests").color(Color::Muted))
                .child(
                    Button::new("sign-in-github", "Sign in to GitHub")
                        .style(ButtonStyle::Filled)
                        .on_click(|_event, _window, cx| {
                            cx.dispatch_action(&SignInToGitHub);
                        }),
                )
        } else if self.loading {
            v_flex()
                .flex_1()
                .justify_center()
                .items_center()
                .child(Label::new("Loading pull requests...").color(Color::Muted))
        } else if let Some(error) = &self.error {
            v_flex()
                .flex_1()
                .p_4()
                .child(Label::new(format!("Error: {}", error)).color(Color::Error))
        } else if self.pull_requests.is_empty() {
            v_flex()
                .flex_1()
                .p_4()
                .child(Label::new("No pull requests to display").color(Color::Muted))
        } else {
            let pr_count = self.pull_requests.len();

            v_flex()
                .flex_1()
                .child(
                    uniform_list(
                        "pr-list",
                        pr_count,
                        cx.processor(|this: &mut PRPanel, range: std::ops::Range<usize>, _window: &mut gpui::Window, cx: &mut gpui::Context<PRPanel>| {
                            let mut items = Vec::with_capacity(range.end - range.start);
                            for index in range {
                                if let Some(pr) = this.pull_requests.get(index) {
                                    let pr_clone = pr.clone();
                                    let is_selected = this.selected_index == Some(index);
                                    let item = div()
                                        .id(("pr-item", pr.number))
                                        .when(is_selected, |div| {
                                            div.bg(cx.theme().colors().element_selected)
                                        })
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.selected_index = Some(index);
                                            cx.notify();
                                        }))
                                        .child(PRListItem::new(pr_clone));
                                    items.push(item);
                                }
                            }
                            items
                        }),
                    )
                    .flex_1()
                    .size_full()
                    .track_scroll(&self.scroll_handle),
                )
        };

        v_flex()
            .id("pr-panel")
            .key_context("PRPanel")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(
                self.panel_header_container(window, cx).child(
                    h_flex()
                        .w_full()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(Icon::new(IconName::PullRequest))
                                .child(Label::new("Pull Requests")),
                        )
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    IconButton::new("create-pr", IconName::Plus)
                                        .tooltip(Tooltip::text("Create Pull Request"))
                                        .on_click(|_event, _window, cx| {
                                            cx.dispatch_action(&CreatePullRequest);
                                        }),
                                )
                                .child(
                                    IconButton::new("refresh-prs", IconName::ArrowCircle)
                                        .tooltip(Tooltip::text("Refresh"))
                                        .on_click(|_event, _window, cx| {
                                            cx.dispatch_action(&Refresh);
                                        }),
                                ),
                        ),
                ),
            )
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::{GitHubClient, PRState, PullRequest, RefData, User};
    use gpui::TestAppContext;
    use http_client::FakeHttpClient;
    use std::sync::Arc;

    #[gpui::test]
    async fn test_pr_panel_creation(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));
        panel.read_with(cx, |panel, _cx| {
            assert!(panel.width.is_none());
        });
    }

    #[gpui::test]
    async fn test_pr_panel_has_github_client_field(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));
        panel.read_with(cx, |panel, _cx| {
            assert!(panel.github_client.is_none());
        });
    }

    #[gpui::test]
    async fn test_pr_panel_initial_state(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));
        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.pull_requests.len(), 0);
            assert!(!panel.loading);
            assert!(panel.error.is_none());
            assert!(panel.selected_index.is_none());
        });
    }

    #[gpui::test]
    async fn test_load_pull_requests_sets_loading_state(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let prs: Vec<PullRequest> = vec![];
            let body = serde_json::to_vec(&prs).expect("serialize prs");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_client = Some(github_client);
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.load_pull_requests("owner", "repo", cx);
            assert!(panel.loading);
        });
    }

    #[gpui::test]
    async fn test_load_pull_requests_success(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let prs = vec![
                PullRequest {
                    number: 1,
                    title: "Test PR 1".to_string(),
                    state: PRState::Open,
                    author: User {
                        id: 123,
                        login: "testuser".to_string(),
                        avatar_url: "https://example.com/avatar.png".to_string(),
                    },
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    draft: false,
                    url: "https://github.com/owner/repo/pull/1".to_string(),
                    head_ref_data: RefData {
                        ref_name: "feature".to_string(),
                    },
                    base_ref_data: RefData {
                        ref_name: "main".to_string(),
                    },
                },
            ];
            let body = serde_json::to_vec(&prs).expect("serialize prs");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_client = Some(github_client);
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.load_pull_requests("owner", "repo", cx);
        });

        cx.run_until_parked();

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.loading);
            assert!(panel.error.is_none());
            assert_eq!(panel.pull_requests.len(), 1);
            assert_eq!(panel.pull_requests[0].number, 1);
            assert_eq!(panel.pull_requests[0].title, "Test PR 1");
        });
    }

    #[gpui::test]
    async fn test_load_pull_requests_error(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(404)
                .body("Not Found".as_bytes().to_vec().into())
                .expect("build response"))
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_client = Some(github_client);
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.load_pull_requests("owner", "repo", cx);
        });

        cx.run_until_parked();

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.loading);
            assert!(panel.error.is_some());
            assert_eq!(panel.pull_requests.len(), 0);
        });
    }

    #[gpui::test]
    async fn test_load_pull_requests_without_client(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));

        panel.update(cx, |panel, cx| {
            panel.load_pull_requests("owner", "repo", cx);
        });

        cx.run_until_parked();

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.loading);
            assert!(panel.error.is_some());
        });
    }

    #[gpui::test]
    async fn test_panel_persistent_name() {
        assert_eq!(PRPanel::persistent_name(), "PRPanel");
    }

    #[gpui::test]
    async fn test_panel_icon() {
        let mut cx = TestAppContext::default();
        let panel = cx.new(|cx| PRPanel::new(cx));
        panel.read_with(&cx, |panel, window, cx| {
            assert_eq!(panel.icon(window, cx), Some(IconName::PullRequest));
        });
    }

    #[gpui::test]
    async fn test_panel_new_initializes_github_client(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));
        panel.read_with(cx, |panel, _cx| {
            assert!(panel.github_client.is_some(), "GitHub client should be initialized");
        });
    }

    #[gpui::test]
    async fn test_set_github_token(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));

        panel.update(cx, |panel, cx| {
            panel.set_github_token("test_token".to_string(), cx);
        });

        panel.read_with(cx, |panel, _cx| {
            assert!(panel.has_github_token());
            if let Some(client) = &panel.github_client {
                assert_eq!(client.token(), Some("test_token"));
            }
        });
    }

    #[gpui::test]
    async fn test_has_github_token_returns_false_without_token(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.has_github_token());
        });
    }

    #[gpui::test]
    async fn test_has_github_token_returns_true_with_token(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| PRPanel::new(cx));

        panel.update(cx, |panel, cx| {
            panel.set_github_token("test_token".to_string(), cx);
        });

        panel.read_with(cx, |panel, _cx| {
            assert!(panel.has_github_token());
        });
    }

    #[gpui::test]
    async fn test_github_repo_tracking(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_repo = Some(crate::GitHubRepo {
                owner: "test_owner".to_string(),
                repo: "test_repo".to_string(),
            });
            panel
        });

        panel.read_with(cx, |panel, _cx| {
            assert!(panel.github_repo.is_some());
            let repo = panel.github_repo.as_ref().unwrap();
            assert_eq!(repo.owner, "test_owner");
            assert_eq!(repo.repo, "test_repo");
        });
    }

    #[gpui::test]
    async fn test_refresh_pull_requests_with_both_client_and_repo(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let prs: Vec<PullRequest> = vec![];
            let body = serde_json::to_vec(&prs).expect("serialize prs");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));
            panel.github_client = Some(github_client);
            panel.github_repo = Some(crate::GitHubRepo {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
            });
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.refresh_pull_requests(cx);
            assert!(panel.loading);
        });

        cx.run_until_parked();

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.loading);
        });
    }

    #[gpui::test]
    async fn test_refresh_pull_requests_without_client(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_client = None;
            panel.github_repo = Some(crate::GitHubRepo {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
            });
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.refresh_pull_requests(cx);
            assert!(!panel.loading);
        });
    }

    #[gpui::test]
    async fn test_refresh_pull_requests_without_repo(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));
            panel.github_client = Some(github_client);
            panel.github_repo = None;
            panel
        });

        panel.update(cx, |panel, cx| {
            panel.refresh_pull_requests(cx);
            assert!(!panel.loading);
        });
    }

    #[gpui::test]
    async fn test_on_repo_detected_with_token_loads_prs(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let prs: Vec<PullRequest> = vec![];
            let body = serde_json::to_vec(&prs).expect("serialize prs");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));
            panel.github_client = Some(github_client);
            panel
        });

        panel.update(cx, |panel, cx| {
            let repo = crate::GitHubRepo {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
            };
            panel.on_repo_detected(repo, cx);
            assert!(panel.loading);
        });

        cx.run_until_parked();

        panel.read_with(cx, |panel, _cx| {
            assert!(!panel.loading);
        });
    }

    #[gpui::test]
    async fn test_on_repo_detected_without_token_does_not_load(cx: &mut TestAppContext) {
        let panel = cx.new(|cx| {
            let mut panel = PRPanel::new(cx);
            panel.github_client = None;
            panel
        });

        panel.update(cx, |panel, cx| {
            let repo = crate::GitHubRepo {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
            };
            panel.on_repo_detected(repo, cx);
            assert!(!panel.loading);
        });
    }
}

