use fuzzy::StringMatchCandidate;
use github_client::{GitHubClient, User};
use gpui::*;
use picker::{Picker, PickerDelegate};
use std::sync::Arc;
use ui::{prelude::*, ListItem, ListItemSpacing};
use util::ResultExt;

#[derive(Clone, Debug, PartialEq)]
struct UserEntry {
    user: User,
    positions: Vec<usize>,
}

pub struct UserPicker {
    picker: Entity<Picker<UserPickerDelegate>>,
    focus_handle: FocusHandle,
}

impl UserPicker {
    pub fn new(
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        on_select: Arc<dyn Fn(User, &mut Window, &mut App) + Send + Sync + 'static>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = UserPickerDelegate::new(github_client, owner, repo, on_select);
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        let focus_handle = picker.focus_handle(cx);

        Self {
            picker,
            focus_handle,
        }
    }

    pub fn selected_users(&self, cx: &App) -> Vec<User> {
        self.picker
            .read(cx)
            .delegate
            .selected_users
            .iter()
            .cloned()
            .collect()
    }
}

impl EventEmitter<DismissEvent> for UserPicker {}

impl Focusable for UserPicker {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for UserPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(400.0))
            .child(self.picker.clone())
            .on_mouse_down_out(cx.listener(move |this, _, window, cx| {
                this.picker.update(cx, |picker, cx| {
                    picker.cancel(&Default::default(), window, cx);
                })
            }))
    }
}

pub struct UserPickerDelegate {
    github_client: Arc<GitHubClient>,
    owner: String,
    repo: String,
    matches: Vec<UserEntry>,
    selected_index: usize,
    selected_users: Vec<User>,
    on_select: Arc<dyn Fn(User, &mut Window, &mut App) + Send + Sync + 'static>,
}

impl UserPickerDelegate {
    fn new(
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        on_select: Arc<dyn Fn(User, &mut Window, &mut App) + Send + Sync + 'static>,
    ) -> Self {
        Self {
            github_client,
            owner,
            repo,
            matches: Vec::new(),
            selected_index: 0,
            selected_users: Vec::new(),
            on_select,
        }
    }
}

impl PickerDelegate for UserPickerDelegate {
    type ListItem = ListItem;

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(
        &mut self,
        index: usize,
        _window: &mut Window,
        _cx: &mut Context<Picker<Self>>,
    ) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        "Search users...".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let github_client = self.github_client.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();

        cx.spawn_in(window, async move |picker, cx| {
            let users = if query.is_empty() {
                Vec::new()
            } else {
                github_client
                    .search_users(&owner, &repo, &query)
                    .await
                    .unwrap_or_else(|_| Vec::new())
            };

            let candidates: Vec<StringMatchCandidate> = users
                .iter()
                .enumerate()
                .map(|(index, user)| StringMatchCandidate::new(index, &user.login))
                .collect();

            let matches = if query.is_empty() {
                users
                    .into_iter()
                    .map(|user| UserEntry {
                        user,
                        positions: Vec::new(),
                    })
                    .collect()
            } else {
                let fuzzy_matches = fuzzy::match_strings(
                    &candidates,
                    &query,
                    true,
                    true,
                    10000,
                    &Default::default(),
                    cx.background_executor().clone(),
                )
                .await;

                fuzzy_matches
                    .into_iter()
                    .map(|match_result| {
                        let user = users[match_result.candidate_id].clone();
                        UserEntry {
                            user,
                            positions: match_result.positions,
                        }
                    })
                    .collect()
            };

            picker
                .update(cx, |picker, _cx| {
                    picker.delegate.matches = matches;
                    picker.delegate.selected_index = 0;
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _secondary: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        if let Some(entry) = self.matches.get(self.selected_index) {
            let user = entry.user.clone();
            if !self.selected_users.iter().any(|u| u.id == user.id) {
                self.selected_users.push(user.clone());
            }
            (self.on_select)(user, window, cx);
        }
    }

    fn dismissed(&mut self, _window: &mut Window, cx: &mut Context<Picker<Self>>) {
        cx.emit(DismissEvent);
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let entry = self.matches.get(index)?;
        let is_selected = self.selected_users.iter().any(|u| u.id == entry.user.id);

        Some(
            ListItem::new(format!("user-{}", entry.user.id))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(
                            div()
                                .w(px(20.0))
                                .h(px(20.0))
                                .rounded(px(10.0))
                                .bg(cx.theme().colors().element_background)
                                .child(
                                    img(entry.user.avatar_url.clone())
                                        .w_full()
                                        .h_full()
                                        .rounded(px(10.0)),
                                ),
                        )
                        .child(
                            Label::new(entry.user.login.clone())
                                .color(if is_selected {
                                    Color::Accent
                                } else {
                                    Color::Default
                                })
                                .single_line()
                                .truncate(),
                        ),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::GitHubClient;
    use http_client::FakeHttpClient;

    #[gpui::test]
    async fn test_user_search_with_empty_query(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_user: User, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            UserPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        window
            .update_in(cx, |picker, window, cx| {
                picker.picker.update(cx, |picker, cx| {
                    picker.delegate.update_matches(String::new(), window, cx)
                })
            })
            .await;

        cx.background_executor.run_until_parked();

        window.read_with(cx, |picker, cx| {
            picker.picker.read(cx).delegate.matches.is_empty();
        });
    }

    #[gpui::test]
    async fn test_user_selection_tracking(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let users = vec![
                User {
                    id: 1,
                    login: "user1".to_string(),
                    avatar_url: "https://example.com/avatar1.png".to_string(),
                },
                User {
                    id: 2,
                    login: "user2".to_string(),
                    avatar_url: "https://example.com/avatar2.png".to_string(),
                },
            ];
            let body = serde_json::to_vec(&users).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let (selected_tx, mut selected_rx) = futures::channel::mpsc::unbounded();
        let on_select = Arc::new(move |user: User, _window: &mut Window, _cx: &mut App| {
            selected_tx.unbounded_send(user).ok();
        });

        let window = cx.new_window(|window, cx| {
            UserPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        window
            .update_in(cx, |picker, window, cx| {
                picker.picker.update(cx, |picker, cx| {
                    picker.delegate.update_matches("user".to_string(), window, cx)
                })
            })
            .await;

        cx.background_executor.run_until_parked();

        window.update_in(cx, |picker, window, cx| {
            picker.picker.update(cx, |picker, cx| {
                picker.delegate.confirm(false, window, cx);
            });
        });

        cx.background_executor.run_until_parked();

        let selected_user = selected_rx.try_next().unwrap().unwrap();
        assert_eq!(selected_user.login, "user1");

        window.read_with(cx, |picker, cx| {
            let selected = picker.selected_users(cx);
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].login, "user1");
        });
    }

    #[gpui::test]
    fn test_duplicate_selection_prevention(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_user: User, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            UserPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        window.update(cx, |picker, window, cx| {
            let user = User {
                id: 1,
                login: "user1".to_string(),
                avatar_url: "https://example.com/avatar1.png".to_string(),
            };

            picker.picker.update(cx, |picker, cx| {
                picker.delegate.matches = vec![UserEntry {
                    user: user.clone(),
                    positions: Vec::new(),
                }];
                picker.delegate.confirm(false, window, cx);
                picker.delegate.confirm(false, window, cx);
            });

            let selected = picker.selected_users(cx);
            assert_eq!(selected.len(), 1);
        });
    }
}
