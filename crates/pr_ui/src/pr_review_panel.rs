use std::collections::HashMap;

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

use crate::inline_comment::InlineCommentData;

actions!(pr_review_panel, [ToggleReviewPanel, StartReview, SubmitReview, DiscardReview]);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReviewSessionState {
    #[default]
    NotStarted,
    InProgress,
    Submitted,
}

#[derive(Debug, Clone)]
pub enum PRReviewPanelEvent {
    CommentLineClicked { path: String, line: u32 },
    CommentAdded(InlineCommentData),
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
    comment_store: PRCommentStore,
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
            comment_store: PRCommentStore::new(),
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

    pub fn comment_store(&self) -> &PRCommentStore {
        &self.comment_store
    }

    pub fn comment_store_mut(&mut self) -> &mut PRCommentStore {
        &mut self.comment_store
    }

    pub fn load_comments_from_github(
        &mut self,
        comments: Vec<github_client::PullRequestReviewComment>,
        cx: &mut Context<Self>,
    ) {
        self.comment_store.clear();

        for github_comment in comments {
            let line = github_comment.line.or(github_comment.original_line).unwrap_or(0);

            let side = match github_comment.side {
                Some(github_client::DiffSide::Left) => crate::CommentSide::Left,
                Some(github_client::DiffSide::Right) | None => crate::CommentSide::Right,
            };

            let comment = InlineCommentData {
                id: github_comment.id,
                author: github_comment.author.login,
                body: github_comment.body,
                path: github_comment.path,
                line,
                side,
                created_at: github_comment.created_at.to_rfc3339(),
                in_reply_to: github_comment.in_reply_to_id,
                is_pending: false,
            };

            self.comment_store.add_comment(comment);
        }

        cx.notify();
    }

    pub fn clear_comments(&mut self, cx: &mut Context<Self>) {
        self.comment_store.clear();
        cx.notify();
    }

    pub fn get_line_comments(&self, path: &str, line: u32) -> Vec<&InlineCommentData> {
        self.comment_store.get_line_comments(path, line)
    }

    pub fn has_comments_for_line(&self, path: &str, line: u32) -> bool {
        self.comment_store.has_comments_for_line(path, line)
    }

    pub fn files_with_inline_comments(&self) -> Vec<&str> {
        self.comment_store.files_with_comments()
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
impl EventEmitter<PRReviewPanelEvent> for PRReviewPanel {}

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

/// PRCommentStore manages inline comments for pull request files
/// It provides efficient lookup by file path and line number
#[derive(Default, Debug, Clone)]
pub struct PRCommentStore {
    comments_by_file: HashMap<String, Vec<InlineCommentData>>,
}

impl PRCommentStore {
    pub fn new() -> Self {
        Self {
            comments_by_file: HashMap::new(),
        }
    }

    /// Set all comments for a specific file path
    pub fn set_comments(&mut self, path: &str, comments: Vec<InlineCommentData>) {
        self.comments_by_file.insert(path.to_string(), comments);
    }

    /// Get all comments for a specific file path
    pub fn get_comments(&self, path: &str) -> Option<&Vec<InlineCommentData>> {
        self.comments_by_file.get(path)
    }

    /// Get comments for a specific line in a file
    pub fn get_line_comments(&self, path: &str, line: u32) -> Vec<&InlineCommentData> {
        self.comments_by_file
            .get(path)
            .map(|comments| comments.iter().filter(|c| c.line == line).collect())
            .unwrap_or_default()
    }

    /// Add a single comment to a file
    pub fn add_comment(&mut self, comment: InlineCommentData) {
        let path = comment.path.clone();
        self.comments_by_file
            .entry(path)
            .or_insert_with(Vec::new)
            .push(comment);
    }

    /// Remove all comments for a specific file
    pub fn remove_file_comments(&mut self, path: &str) {
        self.comments_by_file.remove(path);
    }

    /// Clear all comments from the store
    pub fn clear(&mut self) {
        self.comments_by_file.clear();
    }

    /// Get all file paths that have comments
    pub fn files_with_comments(&self) -> Vec<&str> {
        self.comments_by_file.keys().map(|s| s.as_str()).collect()
    }

    /// Get total number of comments across all files
    pub fn total_comment_count(&self) -> usize {
        self.comments_by_file
            .values()
            .map(|comments| comments.len())
            .sum()
    }

    /// Check if a specific file has comments
    pub fn has_comments_for_file(&self, path: &str) -> bool {
        self.comments_by_file
            .get(path)
            .map(|comments| !comments.is_empty())
            .unwrap_or(false)
    }

    /// Check if a specific line in a file has comments
    pub fn has_comments_for_line(&self, path: &str, line: u32) -> bool {
        !self.get_line_comments(path, line).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_comment_store_new() {
        let store = PRCommentStore::new();
        assert_eq!(store.total_comment_count(), 0);
        assert!(store.files_with_comments().is_empty());
    }

    #[test]
    fn test_set_and_get_comments() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Great work!".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Needs improvement".to_string(),
            "src/main.rs".to_string(),
            15,
            2,
        );

        store.set_comments("src/main.rs", vec![comment1.clone(), comment2.clone()]);

        let comments = store.get_comments("src/main.rs");
        assert!(comments.is_some());
        assert_eq!(comments.unwrap().len(), 2);
    }

    #[test]
    fn test_get_line_comments() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment on line 10".to_string(),
            "src/lib.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Another comment on line 10".to_string(),
            "src/lib.rs".to_string(),
            10,
            2,
        );

        let comment3 = InlineCommentData::new(
            "user3".to_string(),
            "Comment on line 20".to_string(),
            "src/lib.rs".to_string(),
            20,
            3,
        );

        store.set_comments("src/lib.rs", vec![comment1, comment2, comment3]);

        let line_10_comments = store.get_line_comments("src/lib.rs", 10);
        assert_eq!(line_10_comments.len(), 2);

        let line_20_comments = store.get_line_comments("src/lib.rs", 20);
        assert_eq!(line_20_comments.len(), 1);

        let line_30_comments = store.get_line_comments("src/lib.rs", 30);
        assert_eq!(line_30_comments.len(), 0);
    }

    #[test]
    fn test_add_comment() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "First comment".to_string(),
            "src/test.rs".to_string(),
            5,
            1,
        );

        store.add_comment(comment1);
        assert_eq!(store.total_comment_count(), 1);

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Second comment".to_string(),
            "src/test.rs".to_string(),
            10,
            2,
        );

        store.add_comment(comment2);
        assert_eq!(store.total_comment_count(), 2);

        let comments = store.get_comments("src/test.rs");
        assert_eq!(comments.unwrap().len(), 2);
    }

    #[test]
    fn test_remove_file_comments() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment 1".to_string(),
            "src/file1.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Comment 2".to_string(),
            "src/file2.rs".to_string(),
            20,
            2,
        );

        store.add_comment(comment1);
        store.add_comment(comment2);
        assert_eq!(store.files_with_comments().len(), 2);

        store.remove_file_comments("src/file1.rs");
        assert_eq!(store.files_with_comments().len(), 1);
        assert!(!store.has_comments_for_file("src/file1.rs"));
        assert!(store.has_comments_for_file("src/file2.rs"));
    }

    #[test]
    fn test_clear() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment 1".to_string(),
            "src/file1.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Comment 2".to_string(),
            "src/file2.rs".to_string(),
            20,
            2,
        );

        store.add_comment(comment1);
        store.add_comment(comment2);
        assert_eq!(store.total_comment_count(), 2);

        store.clear();
        assert_eq!(store.total_comment_count(), 0);
        assert!(store.files_with_comments().is_empty());
    }

    #[test]
    fn test_files_with_comments() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment 1".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Comment 2".to_string(),
            "src/lib.rs".to_string(),
            20,
            2,
        );

        let comment3 = InlineCommentData::new(
            "user3".to_string(),
            "Comment 3".to_string(),
            "src/test.rs".to_string(),
            30,
            3,
        );

        store.add_comment(comment1);
        store.add_comment(comment2);
        store.add_comment(comment3);

        let files = store.files_with_comments();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"src/main.rs"));
        assert!(files.contains(&"src/lib.rs"));
        assert!(files.contains(&"src/test.rs"));
    }

    #[test]
    fn test_total_comment_count() {
        let mut store = PRCommentStore::new();
        assert_eq!(store.total_comment_count(), 0);

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment 1".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );
        store.add_comment(comment1);
        assert_eq!(store.total_comment_count(), 1);

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Comment 2".to_string(),
            "src/main.rs".to_string(),
            20,
            2,
        );
        store.add_comment(comment2);
        assert_eq!(store.total_comment_count(), 2);

        let comment3 = InlineCommentData::new(
            "user3".to_string(),
            "Comment 3".to_string(),
            "src/lib.rs".to_string(),
            15,
            3,
        );
        store.add_comment(comment3);
        assert_eq!(store.total_comment_count(), 3);
    }

    #[test]
    fn test_has_comments_for_file() {
        let mut store = PRCommentStore::new();

        let comment = InlineCommentData::new(
            "user1".to_string(),
            "Comment".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );

        store.add_comment(comment);

        assert!(store.has_comments_for_file("src/main.rs"));
        assert!(!store.has_comments_for_file("src/lib.rs"));
    }

    #[test]
    fn test_has_comments_for_line() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "Comment on line 10".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Comment on line 20".to_string(),
            "src/main.rs".to_string(),
            20,
            2,
        );

        store.add_comment(comment1);
        store.add_comment(comment2);

        assert!(store.has_comments_for_line("src/main.rs", 10));
        assert!(store.has_comments_for_line("src/main.rs", 20));
        assert!(!store.has_comments_for_line("src/main.rs", 30));
        assert!(!store.has_comments_for_line("src/lib.rs", 10));
    }

    #[test]
    fn test_get_comments_for_nonexistent_file() {
        let store = PRCommentStore::new();

        let comments = store.get_comments("nonexistent.rs");
        assert!(comments.is_none());

        let line_comments = store.get_line_comments("nonexistent.rs", 10);
        assert!(line_comments.is_empty());
    }

    #[test]
    fn test_multiple_comments_same_line() {
        let mut store = PRCommentStore::new();

        let comment1 = InlineCommentData::new(
            "user1".to_string(),
            "First comment".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );

        let comment2 = InlineCommentData::new(
            "user2".to_string(),
            "Second comment".to_string(),
            "src/main.rs".to_string(),
            10,
            2,
        );

        let comment3 = InlineCommentData::new(
            "user3".to_string(),
            "Third comment".to_string(),
            "src/main.rs".to_string(),
            10,
            3,
        );

        store.add_comment(comment1);
        store.add_comment(comment2);
        store.add_comment(comment3);

        let line_comments = store.get_line_comments("src/main.rs", 10);
        assert_eq!(line_comments.len(), 3);
    }

    #[gpui::test]
    fn test_pr_review_panel_creation(cx: &mut gpui::TestAppContext) {
        let panel = cx.new(|cx| PRReviewPanel::new(cx));

        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.comment_store().total_comment_count(), 0);
            assert_eq!(panel.session_state, ReviewSessionState::NotStarted);
        });
    }

    #[gpui::test]
    fn test_pr_review_panel_start_review(cx: &mut gpui::TestAppContext) {
        let panel = cx.new(|cx| PRReviewPanel::new(cx));

        panel.update(cx, |panel, cx| {
            panel.start_review(42, "Test PR".to_string(), cx);
        });

        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.session_state, ReviewSessionState::InProgress);
            assert_eq!(panel.pr_number, Some(42));
            assert_eq!(panel.pr_title, Some("Test PR".to_string()));
        });
    }

    #[gpui::test]
    fn test_pr_review_panel_comment_store_integration(cx: &mut gpui::TestAppContext) {
        let panel = cx.new(|cx| PRReviewPanel::new(cx));

        panel.update(cx, |panel, cx| {
            let comment = InlineCommentData::new(
                "test_user".to_string(),
                "This is a comment".to_string(),
                "src/main.rs".to_string(),
                42,
                1,
            );

            panel.comment_store_mut().add_comment(comment);
            cx.notify();
        });

        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.comment_store().total_comment_count(), 1);
            assert!(panel.has_comments_for_line("src/main.rs", 42));
            assert!(!panel.has_comments_for_line("src/main.rs", 100));

            let line_comments = panel.get_line_comments("src/main.rs", 42);
            assert_eq!(line_comments.len(), 1);
            assert_eq!(line_comments[0].author, "test_user");
            assert_eq!(line_comments[0].body, "This is a comment");
        });
    }

    #[gpui::test]
    fn test_pr_review_panel_clear_comments(cx: &mut gpui::TestAppContext) {
        let panel = cx.new(|cx| PRReviewPanel::new(cx));

        panel.update(cx, |panel, cx| {
            let comment1 = InlineCommentData::new(
                "user1".to_string(),
                "Comment 1".to_string(),
                "src/main.rs".to_string(),
                10,
                1,
            );

            let comment2 = InlineCommentData::new(
                "user2".to_string(),
                "Comment 2".to_string(),
                "src/lib.rs".to_string(),
                20,
                2,
            );

            panel.comment_store_mut().add_comment(comment1);
            panel.comment_store_mut().add_comment(comment2);
        });

        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.comment_store().total_comment_count(), 2);
        });

        panel.update(cx, |panel, cx| {
            panel.clear_comments(cx);
        });

        panel.read_with(cx, |panel, _cx| {
            assert_eq!(panel.comment_store().total_comment_count(), 0);
        });
    }

    #[gpui::test]
    fn test_pr_review_panel_files_with_comments(cx: &mut gpui::TestAppContext) {
        let panel = cx.new(|cx| PRReviewPanel::new(cx));

        panel.update(cx, |panel, cx| {
            let comment1 = InlineCommentData::new(
                "user1".to_string(),
                "Comment 1".to_string(),
                "src/main.rs".to_string(),
                10,
                1,
            );

            let comment2 = InlineCommentData::new(
                "user2".to_string(),
                "Comment 2".to_string(),
                "src/lib.rs".to_string(),
                20,
                2,
            );

            let comment3 = InlineCommentData::new(
                "user3".to_string(),
                "Comment 3".to_string(),
                "src/test.rs".to_string(),
                30,
                3,
            );

            panel.comment_store_mut().add_comment(comment1);
            panel.comment_store_mut().add_comment(comment2);
            panel.comment_store_mut().add_comment(comment3);
            cx.notify();
        });

        panel.read_with(cx, |panel, _cx| {
            let files = panel.files_with_inline_comments();
            assert_eq!(files.len(), 3);
            assert!(files.contains(&"src/main.rs"));
            assert!(files.contains(&"src/lib.rs"));
            assert!(files.contains(&"src/test.rs"));
        });
    }
}

