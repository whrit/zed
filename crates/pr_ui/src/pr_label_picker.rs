use fuzzy::StringMatchCandidate;
use github_client::{GitHubClient, Label as GitHubLabel};
use gpui::*;
use picker::{Picker, PickerDelegate};
use std::sync::Arc;
use ui::{prelude::*, Label, ListItem, ListItemSpacing};
use util::ResultExt;

#[derive(Clone, Debug, PartialEq)]
struct LabelEntry {
    label: GitHubLabel,
    positions: Vec<usize>,
}

pub struct LabelPicker {
    picker: Entity<Picker<LabelPickerDelegate>>,
    focus_handle: FocusHandle,
}

impl LabelPicker {
    pub fn new(
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        on_select: Arc<dyn Fn(GitHubLabel, &mut Window, &mut App) + Send + Sync + 'static>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = LabelPickerDelegate::new(github_client.clone(), owner, repo, on_select);
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        let focus_handle = picker.focus_handle(cx);

        let github_client_clone = github_client.clone();
        let owner_clone = picker.read(cx).delegate.owner.clone();
        let repo_clone = picker.read(cx).delegate.repo.clone();

        cx.spawn_in(window, async move |this, cx| {
            let labels = github_client_clone
                .list_labels(&owner_clone, &repo_clone)
                .await
                .unwrap_or_else(|_| Vec::new());

            this.update(cx, |this, cx| {
                this.picker.update(cx, |picker, _cx| {
                    picker.delegate.all_labels = labels;
                    picker.delegate.matches = picker
                        .delegate
                        .all_labels
                        .iter()
                        .map(|label| LabelEntry {
                            label: label.clone(),
                            positions: Vec::new(),
                        })
                        .collect();
                });
            })
            .ok();
        })
        .detach();

        Self {
            picker,
            focus_handle,
        }
    }

    pub fn selected_labels(&self, cx: &App) -> Vec<GitHubLabel> {
        self.picker
            .read(cx)
            .delegate
            .selected_labels
            .iter()
            .cloned()
            .collect()
    }
}

impl EventEmitter<DismissEvent> for LabelPicker {}

impl Focusable for LabelPicker {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LabelPicker {
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

pub struct LabelPickerDelegate {
    github_client: Arc<GitHubClient>,
    owner: String,
    repo: String,
    all_labels: Vec<GitHubLabel>,
    matches: Vec<LabelEntry>,
    selected_index: usize,
    selected_labels: Vec<GitHubLabel>,
    on_select: Arc<dyn Fn(GitHubLabel, &mut Window, &mut App) + Send + Sync + 'static>,
}

impl LabelPickerDelegate {
    fn new(
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        on_select: Arc<dyn Fn(GitHubLabel, &mut Window, &mut App) + Send + Sync + 'static>,
    ) -> Self {
        Self {
            github_client,
            owner,
            repo,
            all_labels: Vec::new(),
            matches: Vec::new(),
            selected_index: 0,
            selected_labels: Vec::new(),
            on_select,
        }
    }

    fn hex_to_rgb(&self, hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r, g, b);
            }
        }
        (128, 128, 128)
    }
}

impl PickerDelegate for LabelPickerDelegate {
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
        "Search labels...".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let all_labels = self.all_labels.clone();

        cx.spawn_in(window, async move |picker, cx| {
            let candidates: Vec<StringMatchCandidate> = all_labels
                .iter()
                .enumerate()
                .map(|(index, label)| StringMatchCandidate::new(index, &label.name))
                .collect();

            let matches = if query.is_empty() {
                all_labels
                    .into_iter()
                    .map(|label| LabelEntry {
                        label,
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
                        let label = all_labels[match_result.candidate_id].clone();
                        LabelEntry {
                            label,
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
            let label = entry.label.clone();
            if !self.selected_labels.iter().any(|l| l.id == label.id) {
                self.selected_labels.push(label.clone());
            }
            (self.on_select)(label, window, cx);
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
        let is_selected = self.selected_labels.iter().any(|l| l.id == entry.label.id);
        let (r, g, b) = self.hex_to_rgb(&entry.label.color);

        Some(
            ListItem::new(format!("label-{}", entry.label.id))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(
                            div()
                                .w(px(12.0))
                                .h(px(12.0))
                                .rounded(px(6.0))
                                .bg(gpui::rgb(
                                    r as f32 / 255.0,
                                    g as f32 / 255.0,
                                    b as f32 / 255.0,
                                )),
                        )
                        .child(
                            Label::new(entry.label.name.clone())
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
    async fn test_label_loading_on_init(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let labels = vec![
                GitHubLabel {
                    id: 1,
                    name: "bug".to_string(),
                    color: "d73a4a".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
                GitHubLabel {
                    id: 2,
                    name: "enhancement".to_string(),
                    color: "a2eeef".to_string(),
                    description: Some("New feature or request".to_string()),
                },
            ];
            let body = serde_json::to_vec(&labels).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_label: GitHubLabel, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            LabelPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        cx.background_executor.run_until_parked();

        window.read_with(cx, |picker, cx| {
            let labels = picker.picker.read(cx).delegate.all_labels.clone();
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].name, "bug");
            assert_eq!(labels[1].name, "enhancement");
        });
    }

    #[gpui::test]
    async fn test_label_search_filtering(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let labels = vec![
                GitHubLabel {
                    id: 1,
                    name: "bug".to_string(),
                    color: "d73a4a".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
                GitHubLabel {
                    id: 2,
                    name: "enhancement".to_string(),
                    color: "a2eeef".to_string(),
                    description: Some("New feature or request".to_string()),
                },
                GitHubLabel {
                    id: 3,
                    name: "documentation".to_string(),
                    color: "0075ca".to_string(),
                    description: Some("Improvements to documentation".to_string()),
                },
            ];
            let body = serde_json::to_vec(&labels).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_label: GitHubLabel, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            LabelPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        cx.background_executor.run_until_parked();

        window
            .update_in(cx, |picker, window, cx| {
                picker.picker.update(cx, |picker, cx| {
                    picker.delegate.update_matches("doc".to_string(), window, cx)
                })
            })
            .await;

        cx.background_executor.run_until_parked();

        window.read_with(cx, |picker, cx| {
            let matches = picker.picker.read(cx).delegate.matches.clone();
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].label.name, "documentation");
        });
    }

    #[gpui::test]
    async fn test_label_selection_tracking(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let labels = vec![GitHubLabel {
                id: 1,
                name: "bug".to_string(),
                color: "d73a4a".to_string(),
                description: Some("Something isn't working".to_string()),
            }];
            let body = serde_json::to_vec(&labels).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });

        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let (selected_tx, mut selected_rx) = futures::channel::mpsc::unbounded();
        let on_select = Arc::new(move |label: GitHubLabel, _window: &mut Window, _cx: &mut App| {
            selected_tx.unbounded_send(label).ok();
        });

        let window = cx.new_window(|window, cx| {
            LabelPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        cx.background_executor.run_until_parked();

        window.update_in(cx, |picker, window, cx| {
            picker.picker.update(cx, |picker, cx| {
                picker.delegate.confirm(false, window, cx);
            });
        });

        cx.background_executor.run_until_parked();

        let selected_label = selected_rx.try_next().unwrap().unwrap();
        assert_eq!(selected_label.name, "bug");

        window.read_with(cx, |picker, cx| {
            let selected = picker.selected_labels(cx);
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].name, "bug");
        });
    }

    #[gpui::test]
    fn test_hex_to_rgb_conversion(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_label: GitHubLabel, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            LabelPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        window.read_with(cx, |picker, cx| {
            let delegate = &picker.picker.read(cx).delegate;

            let (r, g, b) = delegate.hex_to_rgb("d73a4a");
            assert_eq!((r, g, b), (215, 58, 74));

            let (r, g, b) = delegate.hex_to_rgb("#a2eeef");
            assert_eq!((r, g, b), (162, 238, 239));

            let (r, g, b) = delegate.hex_to_rgb("invalid");
            assert_eq!((r, g, b), (128, 128, 128));
        });
    }

    #[gpui::test]
    fn test_duplicate_label_selection_prevention(cx: &mut TestAppContext) {
        let http_client = FakeHttpClient::with_200_response();
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let on_select = Arc::new(|_label: GitHubLabel, _window: &mut Window, _cx: &mut App| {});

        let window = cx.new_window(|window, cx| {
            LabelPicker::new(
                github_client,
                "owner".to_string(),
                "repo".to_string(),
                on_select,
                window,
                cx,
            )
        });

        window.update(cx, |picker, window, cx| {
            let label = GitHubLabel {
                id: 1,
                name: "bug".to_string(),
                color: "d73a4a".to_string(),
                description: Some("Something isn't working".to_string()),
            };

            picker.picker.update(cx, |picker, cx| {
                picker.delegate.matches = vec![LabelEntry {
                    label: label.clone(),
                    positions: Vec::new(),
                }];
                picker.delegate.confirm(false, window, cx);
                picker.delegate.confirm(false, window, cx);
            });

            let selected = picker.selected_labels(cx);
            assert_eq!(selected.len(), 1);
        });
    }
}
