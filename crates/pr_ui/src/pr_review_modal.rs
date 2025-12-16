use editor::{Editor, EditorElement, EditorStyle};
use gpui::*;
use ui::{
    Button, ButtonSize, ButtonStyle, Label, LabelSize, ModalFooter, ModalHeader, RadioWithLabel,
    prelude::*,
};
use workspace::{DismissDecision, ModalView};

actions!(pr_review, [SubmitReview, CancelReview]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReviewAction {
    #[default]
    Comment,
    Approve,
    RequestChanges,
}

impl ReviewAction {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewAction::Comment => "Comment",
            ReviewAction::Approve => "Approve",
            ReviewAction::RequestChanges => "Request Changes",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ReviewAction::Comment => "Submit general feedback without explicit approval",
            ReviewAction::Approve => "Approve this pull request and allow merging",
            ReviewAction::RequestChanges => "Request changes before this PR can be merged",
        }
    }

    pub fn button_label(&self) -> &'static str {
        match self {
            ReviewAction::Comment => "Submit Review",
            ReviewAction::Approve => "Approve",
            ReviewAction::RequestChanges => "Request Changes",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PendingReviewSummary {
    pub comment_count: usize,
    pub files_commented: usize,
}

pub struct SubmitReviewModal {
    pr_number: u32,
    pr_title: String,
    review_action: ReviewAction,
    comment_editor: Entity<Editor>,
    pending_summary: PendingReviewSummary,
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for SubmitReviewModal {}

impl ModalView for SubmitReviewModal {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> DismissDecision {
        DismissDecision::Dismiss(true)
    }
}

impl Focusable for SubmitReviewModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl SubmitReviewModal {
    pub fn new(
        pr_number: u32,
        pr_title: String,
        pending_summary: PendingReviewSummary,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let comment_editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(4, 10, window, cx);
            editor.set_placeholder_text("Leave a comment (optional)", window, cx);
            editor
        });

        let focus_handle = cx.focus_handle();
        cx.on_focus_out(&focus_handle, window, |_this, _event, _window, cx| {
            cx.emit(DismissEvent);
        }).detach();

        Self {
            pr_number,
            pr_title,
            review_action: ReviewAction::default(),
            comment_editor,
            pending_summary,
            focus_handle,
        }
    }

    #[allow(dead_code)]
    pub fn review_comment(&self, cx: &App) -> String {
        self.comment_editor.read(cx).text(cx)
    }

    #[allow(dead_code)]
    pub fn review_action(&self) -> ReviewAction {
        self.review_action
    }

    fn set_review_action(&mut self, action: ReviewAction, cx: &mut Context<Self>) {
        self.review_action = action;
        cx.notify();
    }

    fn can_submit(&self, cx: &App) -> bool {
        self.pending_summary.comment_count > 0
            || !self.review_comment(cx).trim().is_empty()
            || self.review_action != ReviewAction::Comment
    }

    fn submit(&mut self, _action: &SubmitReview, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_submit(cx) {
            return;
        }
        cx.emit(DismissEvent);
    }

    fn cancel(&mut self, _action: &CancelReview, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Render for SubmitReviewModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let can_submit = self.can_submit(cx);
        let button_label = self.review_action.button_label();

        let header = ModalHeader::new()
            .headline(format!("Submit Review for PR #{}", self.pr_number))
            .description(&self.pr_title)
            .show_dismiss_button(true);

        let pending_icon = Icon::new(IconName::Chat).size(IconSize::Small).color(Color::Accent);
        let pending_text = format!(
            "{} pending comment{} on {} file{}",
            self.pending_summary.comment_count,
            if self.pending_summary.comment_count == 1 { "" } else { "s" },
            self.pending_summary.files_commented,
            if self.pending_summary.files_commented == 1 { "" } else { "s" },
        );
        let pending_label = Label::new(pending_text).size(LabelSize::Small);
        let pending_box = h_flex().gap_2().p_2().rounded_md().bg(cx.theme().colors().surface_background).child(pending_icon).child(pending_label);

        let radio_comment = RadioWithLabel::new(
            "review-action-comment",
            Label::new(ReviewAction::Comment.label()),
            self.review_action == ReviewAction::Comment,
            cx.listener(|this, _selected, _window, cx| this.set_review_action(ReviewAction::Comment, cx)),
        );
        let radio_approve = RadioWithLabel::new(
            "review-action-approve",
            Label::new(ReviewAction::Approve.label()),
            self.review_action == ReviewAction::Approve,
            cx.listener(|this, _selected, _window, cx| this.set_review_action(ReviewAction::Approve, cx)),
        );
        let radio_changes = RadioWithLabel::new(
            "review-action-request-changes",
            Label::new(ReviewAction::RequestChanges.label()),
            self.review_action == ReviewAction::RequestChanges,
            cx.listener(|this, _selected, _window, cx| this.set_review_action(ReviewAction::RequestChanges, cx)),
        );

        let radios = v_flex().gap_2().child(radio_comment).child(radio_approve).child(radio_changes);
        let desc_label = Label::new(self.review_action.description()).size(LabelSize::XSmall).color(Color::Muted);
        let action_section = v_flex().gap_2().child(Label::new("Review Action").size(LabelSize::Small)).child(radios).child(desc_label);

        let editor_style = EditorStyle {
            background: cx.theme().colors().editor_background,
            local_player: cx.theme().players().local(),
            text: TextStyleRefinement { font_size: Some(px(14.0).into()), ..Default::default() }.into(),
            ..Default::default()
        };
        let editor_element = EditorElement::new(&self.comment_editor, editor_style);
        let editor_container = div().min_h(px(100.0)).w_full().border_1().border_color(cx.theme().colors().border).rounded_md().bg(cx.theme().colors().editor_background).p_2().child(editor_element);
        let comment_section = v_flex().gap_1().child(Label::new("Review Comment").size(LabelSize::Small)).child(editor_container);

        let mut body = v_flex().p_4().gap_3();
        if self.pending_summary.comment_count > 0 {
            body = body.child(pending_box);
        }
        body = body.child(action_section).child(comment_section);

        let cancel_btn = Button::new("cancel", "Cancel")
            .style(ButtonStyle::Subtle)
            .size(ButtonSize::Default)
            .on_click(cx.listener(|this, _event, window, cx| this.cancel(&CancelReview, window, cx)));

        let submit_btn = Button::new("submit", button_label)
            .style(ButtonStyle::Filled)
            .size(ButtonSize::Default)
            .disabled(!can_submit)
            .on_click(cx.listener(|this, _event, window, cx| this.submit(&SubmitReview, window, cx)));

        let footer = ModalFooter::new().start_slot(cancel_btn).end_slot(submit_btn);

        v_flex()
            .id("submit-review-modal")
            .key_context("SubmitReviewModal")
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::cancel))
            .elevation_3(cx)
            .w(px(500.0))
            .bg(cx.theme().colors().elevated_surface_background)
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .overflow_hidden()
            .child(header)
            .child(body)
            .child(footer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_action_labels() {
        assert_eq!(ReviewAction::Comment.label(), "Comment");
        assert_eq!(ReviewAction::Approve.label(), "Approve");
        assert_eq!(ReviewAction::RequestChanges.label(), "Request Changes");
    }

    #[test]
    fn test_review_action_descriptions() {
        assert!(!ReviewAction::Comment.description().is_empty());
        assert!(!ReviewAction::Approve.description().is_empty());
        assert!(!ReviewAction::RequestChanges.description().is_empty());
    }

    #[test]
    fn test_review_action_button_labels() {
        assert_eq!(ReviewAction::Comment.button_label(), "Submit Review");
        assert_eq!(ReviewAction::Approve.button_label(), "Approve");
        assert_eq!(ReviewAction::RequestChanges.button_label(), "Request Changes");
    }

    #[test]
    fn test_review_action_default() {
        assert_eq!(ReviewAction::default(), ReviewAction::Comment);
    }

    #[test]
    fn test_pending_review_summary_default() {
        let summary = PendingReviewSummary::default();
        assert_eq!(summary.comment_count, 0);
        assert_eq!(summary.files_commented, 0);
    }
}
