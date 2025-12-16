use editor::{Editor, EditorElement, EditorStyle};
use github_client::{MergeMethod, MergePRParams};
use gpui::*;
use ui::{
    Button, ButtonSize, ButtonStyle, Label, LabelSize, ModalFooter, ModalHeader, RadioWithLabel,
    prelude::*,
};
use workspace::{DismissDecision, ModalView};

actions!(pr_merge, [MergePullRequest, ConfirmMerge, CancelMerge]);

pub struct MergePRModal {
    pr_number: u32,
    pr_title: String,
    merge_method: MergeMethod,
    commit_title_editor: Entity<Editor>,
    commit_message_editor: Entity<Editor>,
    focus_handle: FocusHandle,
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
        cx.emit(DismissEvent);
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
        let merge_button_label = self.merge_button_label().to_string();

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
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.cancel(&CancelMerge, window, cx);
                            })),
                    )
                    .end_slot(
                        Button::new("confirm", merge_button_label)
                            .style(ButtonStyle::Filled)
                            .size(ButtonSize::Default)
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.confirm(&ConfirmMerge, window, cx);
                            })),
                    ),
            )
    }
}

