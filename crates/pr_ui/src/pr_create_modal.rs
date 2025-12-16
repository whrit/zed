use editor::{Editor, EditorElement, EditorMode};
use github_client::{CreatePRParams, GitHubClient};
use gpui::*;
use language::Buffer;
use multi_buffer::MultiBuffer;
use std::sync::Arc;
use ui::{
    Button, ButtonSize, ButtonStyle, Checkbox, Label, LabelSize, ModalFooter, ModalHeader,
    ToggleState, prelude::*,
};
use workspace::{DismissDecision, ModalView};

actions!(pr_create, [CreatePullRequest, SubmitPR, CancelCreate]);

pub struct CreatePRModal {
    title_editor: Entity<Editor>,
    description_editor: Entity<Editor>,
    base_branch: String,
    head_branch: String,
    is_draft: bool,
    focus_handle: FocusHandle,
    github_client: Arc<GitHubClient>,
    owner: String,
    repo: String,
    submitting: bool,
    error: Option<String>,
}

impl EventEmitter<DismissEvent> for CreatePRModal {}

impl ModalView for CreatePRModal {
    fn on_before_dismiss(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> DismissDecision {
        DismissDecision::Dismiss(true)
    }
}

impl Focusable for CreatePRModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl CreatePRModal {
    pub fn new(
        base_branch: String,
        head_branch: String,
        github_client: Arc<GitHubClient>,
        owner: String,
        repo: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_editor = cx.new(|cx| {
            let buffer = cx.new(|cx| Buffer::local("", cx));
            let multibuffer = cx.new(|cx| MultiBuffer::singleton(buffer, cx));
            let mut editor = Editor::new(EditorMode::SingleLine, multibuffer, None, window, cx);
            editor.set_placeholder_text("Pull request title", window, cx);
            editor
        });

        let description_editor = cx.new(|cx| {
            let buffer = cx.new(|cx| Buffer::local("", cx));
            let multibuffer = cx.new(|cx| MultiBuffer::singleton(buffer, cx));
            let mut editor = Editor::new(
                EditorMode::AutoHeight {
                    min_lines: 3,
                    max_lines: Some(10),
                },
                multibuffer,
                None,
                window,
                cx,
            );
            editor.set_placeholder_text("Add a description (optional)", window, cx);
            editor
        });

        let focus_handle = cx.focus_handle();

        cx.on_focus_out(&focus_handle, window, |_this, _event, _window, cx| {
            cx.emit(DismissEvent);
        })
        .detach();

        Self {
            title_editor,
            description_editor,
            base_branch,
            head_branch,
            is_draft: false,
            focus_handle,
            github_client,
            owner,
            repo,
            submitting: false,
            error: None,
        }
    }

    fn title(&self, cx: &App) -> String {
        self.title_editor.read(cx).text(cx)
    }

    fn description(&self, cx: &App) -> String {
        self.description_editor.read(cx).text(cx)
    }

    fn can_submit(&self, cx: &App) -> bool {
        !self.title(cx).trim().is_empty()
    }

    fn toggle_draft(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_draft = !self.is_draft;
        cx.notify();
    }

    fn submit(&mut self, _action: &SubmitPR, _window: &mut Window, cx: &mut Context<Self>) {
        if self.submitting || !self.can_submit(cx) {
            return;
        }

        let title = self.title(cx);
        let description = self.description(cx);
        let is_draft = self.is_draft;
        let base_branch = self.base_branch.clone();
        let head_branch = self.head_branch.clone();
        let github_client = self.github_client.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();

        self.submitting = true;
        self.error = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let params = CreatePRParams {
                title,
                head: head_branch,
                base: base_branch,
                body: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                draft: is_draft,
            };

            let result = github_client.create_pull_request(&owner, &repo, params).await;

            this.update(cx, |this, cx| {
                this.submitting = false;
                match result {
                    Ok(_pr) => cx.emit(DismissEvent),
                    Err(e) => {
                        this.error = Some(e.to_string());
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    fn cancel(&mut self, _action: &CancelCreate, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Render for CreatePRModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let can_submit = self.can_submit(cx) && !self.submitting;

        v_flex()
            .id("pr-create-modal")
            .key_context("CreatePRModal")
            .on_action(cx.listener(Self::submit))
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
                    .headline("Create Pull Request")
                    .show_dismiss_button(true),
            )
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Label::new("Title").size(LabelSize::Small))
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
                                        &self.title_editor,
                                        editor::EditorStyle {
                                            background: cx.theme().colors().editor_background,
                                            local_player: cx.theme().players().local(),
                                            text: gpui::TextStyleRefinement {
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
                            .child(Label::new("Description").size(LabelSize::Small))
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
                                        &self.description_editor,
                                        editor::EditorStyle {
                                            background: cx.theme().colors().editor_background,
                                            local_player: cx.theme().players().local(),
                                            text: gpui::TextStyleRefinement {
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
                            .gap_2()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Label::new("Base:").size(LabelSize::Small))
                                    .child(
                                        Label::new(self.base_branch.clone())
                                            .size(LabelSize::Small)
                                            .color(Color::Accent),
                                    )
                                    .child(Label::new("←").size(LabelSize::Small))
                                    .child(Label::new("Head:").size(LabelSize::Small))
                                    .child(
                                        Label::new(self.head_branch.clone())
                                            .size(LabelSize::Small)
                                            .color(Color::Accent),
                                    ),
                            )
                            .child(
                                Checkbox::new(
                                    "draft-checkbox",
                                    if self.is_draft {
                                        ToggleState::Selected
                                    } else {
                                        ToggleState::Unselected
                                    },
                                )
                                .label("Create as draft")
                                .on_click(cx.listener(|this, _state, window, cx| {
                                    this.toggle_draft(window, cx);
                                })),
                            ),
                    )
                    .when_some(self.error.clone(), |this, error| {
                        this.child(
                            div()
                                .p_2()
                                .bg(gpui::red())
                                .rounded_md()
                                .child(Label::new(format!("Error: {}", error)).size(LabelSize::Small).color(Color::Default)),
                        )
                    }),
            )
            .child(
                ModalFooter::new()
                    .start_slot(
                        Button::new("cancel", "Cancel")
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Default)
                            .disabled(self.submitting)
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.cancel(&CancelCreate, window, cx);
                            })),
                    )
                    .end_slot(
                        Button::new("submit", if self.submitting { "Creating..." } else { "Create Pull Request" })
                            .style(ButtonStyle::Filled)
                            .size(ButtonSize::Default)
                            .disabled(!can_submit)
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.submit(&SubmitPR, window, cx);
                            })),
                    ),
            )
    }
}

