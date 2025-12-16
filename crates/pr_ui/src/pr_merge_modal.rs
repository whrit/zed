use editor::{Editor, EditorElement, EditorStyle};
use github_client::{GitHubClient, MergeMethod, MergePRParams};
use gpui::*;
use std::sync::Arc;
use ui::{
    Button, ButtonSize, ButtonStyle, Label, LabelSize, ModalFooter, ModalHeader, RadioWithLabel,
    prelude::*,
};
use util::ResultExt;
use workspace::{DismissDecision, ModalView};

actions!(pr_merge, [MergePullRequest, ConfirmMerge, CancelMerge]);

pub struct MergePRModal {
    pr_number: u32,
    pr_title: String,
    merge_method: MergeMethod,
    commit_title_editor: Entity<Editor>,
    commit_message_editor: Entity<Editor>,
    focus_handle: FocusHandle,
    github_client: Arc<GitHubClient>,
    owner: String,
    repo: String,
    merging: bool,
    error: Option<String>,
}

impl EventEmitter<DismissEvent> for MergePRModal {}

impl ModalView for MergePRModal {
    fn on_before_dismiss(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> DismissDecision {
        DismissDecision::Dismiss(true)
    }
}

impl Focusable for MergePRModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl MergePRModal {
    pub fn new(
        pr_number: u32,
        pr_title: String,
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let commit_title_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Optional commit title", window, cx);
            editor
        });

        let commit_message_editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(4, 10, window, cx);
            editor.set_placeholder_text("Optional commit message", window, cx);
            editor
        });

        let focus_handle = cx.focus_handle();

        cx.on_focus_out(&focus_handle, window, |_this, _event, _window, cx| {
            cx.emit(DismissEvent);
        })
        .detach();

        Self {
            pr_number,
            pr_title,
            merge_method: MergeMethod::default(),
            commit_title_editor,
            commit_message_editor,
            focus_handle,
            github_client,
            owner,
            repo,
            merging: false,
            error: None,
        }
    }

    fn commit_title(&self, cx: &App) -> String {
        self.commit_title_editor.read(cx).text(cx)
    }

    fn commit_message(&self, cx: &App) -> String {
        self.commit_message_editor.read(cx).text(cx)
    }

    fn set_merge_method(&mut self, method: MergeMethod, cx: &mut Context<Self>) {
        self.merge_method = method;
        cx.notify();
    }

    pub fn merge_params(&self, cx: &App) -> MergePRParams {
        let title = self.commit_title(cx);
        let message = self.commit_message(cx);

        MergePRParams {
            commit_title: if title.is_empty() { None } else { Some(title) },
            commit_message: if message.is_empty() {
                None
            } else {
                Some(message)
            },
            sha: None,
            merge_method: Some(self.merge_method),
        }
    }

    fn confirm(&mut self, _action: &ConfirmMerge, _window: &mut Window, cx: &mut Context<Self>) {
        if self.merging {
            return;
        }

        let params = self.merge_params(cx);
        let github_client = self.github_client.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();
        let pr_number = self.pr_number;

        self.merging = true;
        self.error = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = github_client
                .merge_pull_request(&owner, &repo, pr_number, params)
                .await;

            this.update(cx, |this, cx| {
                this.merging = false;
                match result {
                    Ok(_) => cx.emit(DismissEvent),
                    Err(e) => {
                        this.error = Some(e.to_string());
                        cx.notify();
                    }
                }
            })
            .log_err();
        })
        .detach();
    }

    fn cancel(&mut self, _action: &CancelMerge, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn merge_button_label(&self) -> &str {
        match self.merge_method {
            MergeMethod::Merge => "Merge Pull Request",
            MergeMethod::Squash => "Squash and Merge",
            MergeMethod::Rebase => "Rebase and Merge",
        }
    }

    fn should_show_commit_editors(&self) -> bool {
        matches!(self.merge_method, MergeMethod::Squash)
    }
}

impl Render for MergePRModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let show_editors = self.should_show_commit_editors();
        let merge_button_label = if self.merging {
            "Merging...".to_string()
        } else {
            self.merge_button_label().to_string()
        };
        let is_merging = self.merging;
        let error_message = self.error.clone();

        v_flex()
            .id("pr-merge-modal")
            .key_context("MergePRModal")
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .elevation_3(cx)
            .w(px(600.0))
            .bg(cx.theme().colors().elevated_surface_background)
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .overflow_hidden()
            .child(
                ModalHeader::new()
                    .headline(format!("Merge Pull Request #{}", self.pr_number))
                    .description(&self.pr_title)
                    .show_dismiss_button(true),
            )
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .when_some(error_message, |this, error| {
                        this.child(
                            div()
                                .p_2()
                                .bg(gpui::red())
                                .rounded_md()
                                .child(Label::new(format!("Error: {}", error)).color(Color::Error)),
                        )
                    })
                    .child(
                        v_flex()
                            .gap_2()
                            .child(Label::new("Merge Method").size(LabelSize::Small))
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(RadioWithLabel::new(
                                        "merge-method-merge",
                                        Label::new("Create a merge commit"),
                                        self.merge_method == MergeMethod::Merge,
                                        cx.listener(|this, _selected, _window, cx| {
                                            this.set_merge_method(MergeMethod::Merge, cx);
                                        }),
                                    ))
                                    .child(RadioWithLabel::new(
                                        "merge-method-squash",
                                        Label::new("Squash and merge"),
                                        self.merge_method == MergeMethod::Squash,
                                        cx.listener(|this, _selected, _window, cx| {
                                            this.set_merge_method(MergeMethod::Squash, cx);
                                        }),
                                    ))
                                    .child(RadioWithLabel::new(
                                        "merge-method-rebase",
                                        Label::new("Rebase and merge"),
                                        self.merge_method == MergeMethod::Rebase,
                                        cx.listener(|this, _selected, _window, cx| {
                                            this.set_merge_method(MergeMethod::Rebase, cx);
                                        }),
                                    )),
                            ),
                    )
                    .when(show_editors, |this| {
                        this.child(
                            v_flex()
                                .gap_1()
                                .child(Label::new("Commit Title").size(LabelSize::Small))
                                .child(
                                    div()
                                        .h(px(32.0))
                                        .w_full()
                                        .border_1()
                                        .border_color(cx.theme().colors().border)
                                        .rounded_md()
                                        .bg(cx.theme().colors().editor_background)
                                        .px_2()
                                        .child(EditorElement::new(
                                            &self.commit_title_editor,
                                            EditorStyle {
                                                background: cx.theme().colors().editor_background,
                                                local_player: cx.theme().players().local(),
                                                text: TextStyleRefinement {
                                                    font_size: Some(px(14.0).into()),
                                                    ..Default::default()
                                                }.into(),
                                                ..Default::default()
                                            },
                                        )),
                                ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(Label::new("Commit Message").size(LabelSize::Small))
                                .child(
                                    div()
                                        .h(px(120.0))
                                        .w_full()
                                        .border_1()
                                        .border_color(cx.theme().colors().border)
                                        .rounded_md()
                                        .bg(cx.theme().colors().editor_background)
                                        .p_2()
                                        .child(EditorElement::new(
                                            &self.commit_message_editor,
                                            EditorStyle {
                                                background: cx.theme().colors().editor_background,
                                                local_player: cx.theme().players().local(),
                                                text: TextStyleRefinement {
                                                    font_size: Some(px(14.0).into()),
                                                    ..Default::default()
                                                }.into(),
                                                ..Default::default()
                                            },
                                        )),
                                ),
                        )
                    }),
            )
            .child(
                ModalFooter::new()
                    .start_slot(
                        Button::new("cancel", "Cancel")
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Default)
                            .disabled(is_merging)
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.cancel(&CancelMerge, window, cx);
                            })),
                    )
                    .end_slot(
                        Button::new("confirm", merge_button_label)
                            .style(ButtonStyle::Filled)
                            .size(ButtonSize::Default)
                            .disabled(is_merging)
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.confirm(&ConfirmMerge, window, cx);
                            })),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::MergeResult;
    use http_client::FakeHttpClient;

    #[gpui::test]
    fn test_merge_params_generation_with_empty_fields(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(200)
                .body(vec![].into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            let params = modal.merge_params(cx);

            assert_eq!(params.commit_title, None);
            assert_eq!(params.commit_message, None);
            assert_eq!(params.sha, None);
            assert_eq!(params.merge_method, Some(MergeMethod::Merge));
        });
    }

    #[gpui::test]
    fn test_merge_params_generation_with_custom_title_and_message(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(200)
                .body(vec![].into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            modal.commit_title_editor.update(cx, |editor, cx| {
                editor.set_text("Custom title", window, cx);
            });
            modal.commit_message_editor.update(cx, |editor, cx| {
                editor.set_text("Custom message", window, cx);
            });
            modal.set_merge_method(MergeMethod::Squash, cx);

            let params = modal.merge_params(cx);

            assert_eq!(params.commit_title, Some("Custom title".to_string()));
            assert_eq!(params.commit_message, Some("Custom message".to_string()));
            assert_eq!(params.merge_method, Some(MergeMethod::Squash));
        });
    }

    #[gpui::test]
    async fn test_confirm_sets_merging_state(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let merge_result = MergeResult {
                sha: "merged_sha".to_string(),
                merged: true,
                message: "Successfully merged".to_string(),
            };
            let body = serde_json::to_vec(&merge_result).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            assert!(!modal.merging);
            modal.confirm(&ConfirmMerge, window, cx);
            assert!(modal.merging);
        });

        cx.background_executor.run_until_parked();

        window.read_with(cx, |modal, _cx| {
            assert!(!modal.merging);
            assert!(modal.error.is_none());
        });
    }

    #[gpui::test]
    async fn test_confirm_handles_error(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(422)
                .body(b"{\"message\":\"Pull request is not mergeable\"}".to_vec().into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            modal.confirm(&ConfirmMerge, window, cx);
        });

        cx.background_executor.run_until_parked();

        window.read_with(cx, |modal, _cx| {
            assert!(!modal.merging);
            assert!(modal.error.is_some());
        });
    }

    #[gpui::test]
    fn test_confirm_prevents_duplicate_merges(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            let merge_result = MergeResult {
                sha: "merged_sha".to_string(),
                merged: true,
                message: "Successfully merged".to_string(),
            };
            let body = serde_json::to_vec(&merge_result).unwrap();
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            modal.confirm(&ConfirmMerge, window, cx);
            assert!(modal.merging);

            modal.confirm(&ConfirmMerge, window, cx);
            assert!(modal.merging);
        });
    }

    #[gpui::test]
    fn test_merge_method_selection(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(200)
                .body(vec![].into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            assert_eq!(modal.merge_method, MergeMethod::Merge);

            modal.set_merge_method(MergeMethod::Squash, cx);
            assert_eq!(modal.merge_method, MergeMethod::Squash);

            modal.set_merge_method(MergeMethod::Rebase, cx);
            assert_eq!(modal.merge_method, MergeMethod::Rebase);
        });
    }

    #[gpui::test]
    fn test_should_show_commit_editors_only_for_squash(cx: &mut gpui::TestAppContext) {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(200)
                .body(vec![].into())
                .unwrap())
        });
        let github_client = Arc::new(GitHubClient::with_token(http_client, "test_token".to_string()));

        let window = cx.new_window(|window, cx| {
            MergePRModal::new(
                42,
                "Test PR".to_string(),
                github_client.clone(),
                "owner".to_string(),
                "repo".to_string(),
                window,
                cx,
            )
        });

        window.update(cx, |modal, window, cx| {
            modal.set_merge_method(MergeMethod::Merge, cx);
            assert!(!modal.should_show_commit_editors());

            modal.set_merge_method(MergeMethod::Squash, cx);
            assert!(modal.should_show_commit_editors());

            modal.set_merge_method(MergeMethod::Rebase, cx);
            assert!(!modal.should_show_commit_editors());
        });
    }
}
