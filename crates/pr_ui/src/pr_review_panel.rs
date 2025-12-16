use gpui::{
    actions, uniform_list, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle,
    Focusable, IntoElement, Pixels, Render, RenderOnce, UniformListScrollHandle, Window, px,
};
use panel::PanelHeader;
use ui::prelude::*;
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    Workspace,
};

actions!(pr_review_panel, [ToggleReviewPanel, StartReview, SubmitReview, DiscardReview]);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReviewSessionState {
    #[default]
    NotStarted,
    InProgress,
    Submitted,
}

#[derive(Debug, Clone)]
pub struct PendingComment {
    pub path: String,
    pub line: u32,
    pub body: String,
}

#[derive(IntoElement)]
struct PendingCommentItem {
    path: String,
    line: u32,
    body: String,
    index: usize,
}

impl PendingCommentItem {
    fn new(comment: &PendingComment, index: usize) -> Self {
        Self {
            path: comment.path.clone(),
            line: comment.line,
            body: comment.body.clone(),
            index,
        }
    }
}

impl RenderOnce for PendingCommentItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let location_row = h_flex()
            .gap_2()
            .items_center()
            .child(Label::new(self.path).size(LabelSize::XSmall).color(Color::Accent))
            .child(Label::new(format!(":{}", self.line)).size(LabelSize::XSmall).color(Color::Muted));

        v_flex()
            .id(("pending-comment", self.index))
            .p_2()
            .gap_1()
            .border_b_1()
            .border_color(gpui::transparent_black())
            .child(location_row)
            .child(Label::new(self.body).size(LabelSize::Small))
    }
}

pub struct PRReviewPanel {
    focus_handle: FocusHandle,
    width: Option<Pixels>,
    session_state: ReviewSessionState,
    pending_comments: Vec<PendingComment>,
    scroll_handle: UniformListScrollHandle,
    pr_number: Option<u32>,
    pr_title: Option<String>,
}

impl PRReviewPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            width: None,
            session_state: ReviewSessionState::NotStarted,
            pending_comments: Vec::new(),
            scroll_handle: UniformListScrollHandle::new(),
            pr_number: None,
            pr_title: None,
        }
    }

    pub fn load(workspace: Entity<Workspace>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        let panel = cx.new(|cx| Self::new(cx));
        workspace.update(cx, |workspace, cx| {
            workspace.add_panel(panel.clone(), window, cx);
        });
        panel
    }

    pub fn start_review(&mut self, pr_number: u32, pr_title: String, cx: &mut Context<Self>) {
        self.pr_number = Some(pr_number);
        self.pr_title = Some(pr_title);
        self.session_state = ReviewSessionState::InProgress;
        self.pending_comments.clear();
        cx.notify();
    }

    pub fn add_pending_comment(&mut self, comment: PendingComment, cx: &mut Context<Self>) {
        self.pending_comments.push(comment);
        cx.notify();
    }

    pub fn remove_pending_comment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.pending_comments.len() {
            self.pending_comments.remove(index);
            cx.notify();
        }
    }

    pub fn pending_comment_count(&self) -> usize {
        self.pending_comments.len()
    }

    pub fn files_with_comments(&self) -> usize {
        let mut files: Vec<&str> = self.pending_comments.iter().map(|c| c.path.as_str()).collect();
        files.sort();
        files.dedup();
        files.len()
    }

    pub fn discard_review(&mut self, cx: &mut Context<Self>) {
        self.session_state = ReviewSessionState::NotStarted;
        self.pending_comments.clear();
        self.pr_number = None;
        self.pr_title = None;
        cx.notify();
    }

    pub fn mark_submitted(&mut self, cx: &mut Context<Self>) {
        self.session_state = ReviewSessionState::Submitted;
        self.pending_comments.clear();
        cx.notify();
    }
}

pub fn register(workspace: &mut Workspace, _cx: &mut Context<Workspace>) {
    workspace.register_action(|workspace, _: &ToggleReviewPanel, window, cx| {
        workspace.toggle_panel_focus::<PRReviewPanel>(window, cx);
    });
    workspace.register_action(|_workspace, _: &StartReview, _window, _cx| {});
    workspace.register_action(|_workspace, _: &SubmitReview, _window, _cx| {});
    workspace.register_action(|_workspace, _: &DiscardReview, _window, _cx| {});
}

impl EventEmitter<PanelEvent> for PRReviewPanel {}
impl EventEmitter<DismissEvent> for PRReviewPanel {}

impl Focusable for PRReviewPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PRReviewPanel {
    fn persistent_name() -> &'static str {
        "PRReviewPanel"
    }

    fn panel_key() -> &'static str {
        "pr_review_panel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }

    fn set_position(&mut self, _position: DockPosition, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn size(&self, _window: &Window, _cx: &App) -> Pixels {
        self.width.unwrap_or(px(320.0))
    }

    fn set_size(&mut self, size: Option<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.width = size;
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::Chat)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Review Comments")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(ToggleReviewPanel)
    }

    fn activation_priority(&self) -> u32 {
        4
    }
}

impl PanelHeader for PRReviewPanel {}

impl Render for PRReviewPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header_left = h_flex()
            .gap_2()
            .child(Icon::new(IconName::Chat))
            .child(Label::new("Review"));

        let count_badge = if self.session_state == ReviewSessionState::InProgress {
            Some(Label::new(format!("{}", self.pending_comments.len())).size(LabelSize::Small).color(Color::Accent))
        } else {
            None
        };

        let header_right = h_flex().gap_1().children(count_badge);

        let header_content = h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .child(header_left)
            .child(header_right);

        let header = self.panel_header_container(window, cx).child(header_content);

        let body = match self.session_state {
            ReviewSessionState::NotStarted | ReviewSessionState::Submitted => {
                let icon = Icon::new(IconName::Chat).size(IconSize::Medium).color(Color::Muted);
                let title = Label::new("No active review").color(Color::Muted).size(LabelSize::Small);
                let subtitle = Label::new("Open a PR and start a review to see pending comments here").color(Color::Muted).size(LabelSize::XSmall);
                v_flex().flex_1().items_center().justify_center().gap_2().p_4().child(icon).child(title).child(subtitle).into_any_element()
            }
            ReviewSessionState::InProgress => {
                let pending_count = self.pending_comments.len();
                let files_count = self.files_with_comments();

                let comment_label = Label::new(format!("{} comment{}", pending_count, if pending_count == 1 { "" } else { "s" })).size(LabelSize::XSmall).color(Color::Muted);
                let file_label = Label::new(format!("{} file{}", files_count, if files_count == 1 { "" } else { "s" })).size(LabelSize::XSmall).color(Color::Muted);
                let stats_row = h_flex().gap_2().child(comment_label).child(file_label);

                let mut info_content = v_flex().flex_1().gap_1();
                if let Some(title) = self.pr_title.as_ref() {
                    info_content = info_content.child(Label::new(title.clone()).size(LabelSize::Small));
                }
                info_content = info_content.child(stats_row);

                let info_box = h_flex().p_2().gap_2().bg(cx.theme().colors().surface_background).rounded_md().child(info_content);

                let list_content = if pending_count > 0 {
                    let comments = self.pending_comments.clone();
                    uniform_list("pending-comments-list", pending_count, move |range, _window, _cx| {
                        range.filter_map(|i| comments.get(i).map(|c| PendingCommentItem::new(c, i))).collect()
                    }).flex_1().track_scroll(&self.scroll_handle).into_any_element()
                } else {
                    let empty = Label::new("No pending comments yet").color(Color::Muted).size(LabelSize::Small);
                    v_flex().flex_1().items_center().justify_center().child(empty).into_any_element()
                };

                let discard_btn = Button::new("discard-review", "Discard")
                    .style(ButtonStyle::Subtle)
                    .size(ButtonSize::Compact)
                    .on_click(cx.listener(|this, _event, _window, cx| this.discard_review(cx)));

                let submit_btn = Button::new("submit-review", "Submit Review")
                    .style(ButtonStyle::Filled)
                    .size(ButtonSize::Compact)
                    .disabled(pending_count == 0)
                    .on_click(cx.listener(|_this, _event, _window, cx| cx.dispatch_action(&SubmitReview)));

                let buttons = h_flex().gap_2().p_2().child(discard_btn).child(submit_btn);

                v_flex().flex_1().gap_2().child(info_box).child(list_content).child(buttons).into_any_element()
            }
        };

        v_flex()
            .id("pr-review-panel")
            .key_context("PRReviewPanel")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(header)
            .child(body)
    }
}

