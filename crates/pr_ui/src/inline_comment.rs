use gpui::{App, IntoElement, RenderOnce, SharedString, Window};
use ui::prelude::*;

/// The side of a diff that a comment is attached to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommentSide {
    #[default]
    Right,
    Left,
}

/// Data for an inline comment attached to a specific line
#[derive(Debug, Clone)]
pub struct InlineCommentData {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub path: String,
    pub line: u32,
    pub side: CommentSide,
    pub created_at: String,
    pub in_reply_to: Option<u64>,
    pub is_pending: bool,
}

impl InlineCommentData {
    pub fn new(author: String, body: String, path: String, line: u32, id: u64) -> Self {
        Self {
            id,
            author,
            body,
            path,
            line,
            side: CommentSide::Right,
            created_at: String::new(),
            in_reply_to: None,
            is_pending: false,
        }
    }

    pub fn with_side(mut self, side: CommentSide) -> Self {
        self.side = side;
        self
    }

    pub fn with_created_at(mut self, created_at: String) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_reply_to(mut self, reply_to: u64) -> Self {
        self.in_reply_to = Some(reply_to);
        self
    }

    pub fn as_pending(mut self) -> Self {
        self.is_pending = true;
        self
    }
}

/// A collection of inline comments for a single line
#[derive(Debug, Clone, Default)]
pub struct LineComments {
    pub line: u32,
    pub path: String,
    pub comments: Vec<InlineCommentData>,
}

impl LineComments {
    pub fn new(line: u32, path: String) -> Self {
        Self {
            line,
            path,
            comments: Vec::new(),
        }
    }

    pub fn add_comment(&mut self, comment: InlineCommentData) {
        self.comments.push(comment);
    }

    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    pub fn has_pending(&self) -> bool {
        self.comments.iter().any(|c| c.is_pending)
    }
}

/// A small badge showing the number of comments on a line
/// Used in gutter/margin to indicate comments exist
#[derive(IntoElement)]
pub struct InlineCommentBadge {
    count: usize,
    has_pending: bool,
}

impl InlineCommentBadge {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            has_pending: false,
        }
    }

    pub fn with_pending(mut self, has_pending: bool) -> Self {
        self.has_pending = has_pending;
        self
    }
}

impl RenderOnce for InlineCommentBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let bg_color = if self.has_pending {
            cx.theme().status().warning_background
        } else {
            cx.theme().colors().ghost_element_background
        };

        div()
            .id("inline-comment-badge")
            .flex()
            .items_center()
            .justify_center()
            .w_5()
            .h_5()
            .rounded_full()
            .bg(bg_color)
            .child(
                Label::new(format!("{}", self.count))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
    }
}

/// Renders a single inline comment
#[derive(IntoElement)]
pub struct InlineCommentView {
    comment: InlineCommentData,
    compact: bool,
}

impl InlineCommentView {
    pub fn new(comment: InlineCommentData) -> Self {
        Self {
            comment,
            compact: false,
        }
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

impl RenderOnce for InlineCommentView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let warning_color = cx.theme().status().warning;
        let is_pending = self.comment.is_pending;
        let show_timestamp = !self.compact && !self.comment.created_at.is_empty();
        let created_at = self.comment.created_at;
        let author = self.comment.author;
        let body = self.comment.body;

        v_flex()
            .gap_1()
            .p_2()
            .when(is_pending, |this| {
                this.border_l_2().border_color(warning_color)
            })
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Label::new(author)
                            .size(LabelSize::Small)
                            .color(Color::Default),
                    )
                    .when(show_timestamp, |this| {
                        this.child(
                            Label::new(created_at)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                    })
                    .when(is_pending, |this| {
                        this.child(
                            Label::new("Pending")
                                .size(LabelSize::XSmall)
                                .color(Color::Warning),
                        )
                    }),
            )
            .child(Label::new(body).size(LabelSize::Small))
    }
}

/// A gutter indicator that can be clicked to view/add comments
#[derive(IntoElement)]
pub struct CommentGutterIndicator {
    line: u32,
    comment_count: usize,
    has_pending: bool,
}

impl CommentGutterIndicator {
    pub fn new(line: u32) -> Self {
        Self {
            line,
            comment_count: 0,
            has_pending: false,
        }
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.comment_count = count;
        self
    }

    pub fn with_pending(mut self, has_pending: bool) -> Self {
        self.has_pending = has_pending;
        self
    }
}

impl RenderOnce for CommentGutterIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon_color = if self.has_pending {
            Color::Warning
        } else if self.comment_count > 0 {
            Color::Accent
        } else {
            Color::Muted
        };

        h_flex()
            .id(SharedString::from(format!(
                "comment-gutter-{}",
                self.line
            )))
            .items_center()
            .justify_center()
            .w_6()
            .h_full()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
            .child(
                Icon::new(IconName::Chat)
                    .size(IconSize::Small)
                    .color(icon_color),
            )
            .when(self.comment_count > 0, |this| {
                this.child(
                    Label::new(format!("{}", self.comment_count))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_comment_data_creation() {
        let comment = InlineCommentData::new(
            "testuser".to_string(),
            "Test comment".to_string(),
            "src/main.rs".to_string(),
            42,
            1,
        );

        assert_eq!(comment.id, 1);
        assert_eq!(comment.author, "testuser");
        assert_eq!(comment.line, 42);
        assert_eq!(comment.side, CommentSide::Right);
        assert!(!comment.is_pending);
    }

    #[test]
    fn test_inline_comment_builder_methods() {
        let comment = InlineCommentData::new(
            "user".to_string(),
            "body".to_string(),
            "file.rs".to_string(),
            10,
            1,
        )
        .with_side(CommentSide::Left)
        .with_created_at("2 hours ago".to_string())
        .with_reply_to(5)
        .as_pending();

        assert_eq!(comment.side, CommentSide::Left);
        assert_eq!(comment.created_at, "2 hours ago");
        assert_eq!(comment.in_reply_to, Some(5));
        assert!(comment.is_pending);
    }

    #[test]
    fn test_line_comments() {
        let mut line_comments = LineComments::new(10, "test.rs".to_string());
        assert_eq!(line_comments.comment_count(), 0);
        assert!(!line_comments.has_pending());

        line_comments.add_comment(InlineCommentData::new(
            "user".to_string(),
            "comment".to_string(),
            "test.rs".to_string(),
            10,
            1,
        ));
        assert_eq!(line_comments.comment_count(), 1);

        line_comments.add_comment(
            InlineCommentData::new(
                "user2".to_string(),
                "pending".to_string(),
                "test.rs".to_string(),
                10,
                2,
            )
            .as_pending(),
        );
        assert_eq!(line_comments.comment_count(), 2);
        assert!(line_comments.has_pending());
    }

    #[test]
    fn test_comment_side_default() {
        assert_eq!(CommentSide::default(), CommentSide::Right);
    }

    #[test]
    fn test_inline_comment_badge_creation() {
        let badge = InlineCommentBadge::new(3);
        assert_eq!(badge.count, 3);
        assert!(!badge.has_pending);
    }

    #[test]
    fn test_inline_comment_badge_with_pending() {
        let badge = InlineCommentBadge::new(2).with_pending(true);
        assert_eq!(badge.count, 2);
        assert!(badge.has_pending);
    }

    #[test]
    fn test_inline_comment_view_creation() {
        let comment = InlineCommentData::new(
            "testuser".to_string(),
            "Test body".to_string(),
            "file.rs".to_string(),
            5,
            1,
        );
        let view = InlineCommentView::new(comment);
        assert!(!view.compact);
    }

    #[test]
    fn test_inline_comment_view_compact() {
        let comment = InlineCommentData::new(
            "testuser".to_string(),
            "Test body".to_string(),
            "file.rs".to_string(),
            5,
            1,
        );
        let view = InlineCommentView::new(comment).compact(true);
        assert!(view.compact);
    }

    #[test]
    fn test_comment_gutter_indicator_creation() {
        let indicator = CommentGutterIndicator::new(42);
        assert_eq!(indicator.line, 42);
        assert_eq!(indicator.comment_count, 0);
        assert!(!indicator.has_pending);
    }

    #[test]
    fn test_comment_gutter_indicator_with_count() {
        let indicator = CommentGutterIndicator::new(10).with_count(5);
        assert_eq!(indicator.comment_count, 5);
    }

    #[test]
    fn test_comment_gutter_indicator_with_pending() {
        let indicator = CommentGutterIndicator::new(10).with_pending(true);
        assert!(indicator.has_pending);
    }

    #[test]
    fn test_comment_gutter_indicator_builder_chain() {
        let indicator = CommentGutterIndicator::new(20).with_count(3).with_pending(true);
        assert_eq!(indicator.line, 20);
        assert_eq!(indicator.comment_count, 3);
        assert!(indicator.has_pending);
    }

    #[test]
    fn test_line_comments_empty() {
        let line_comments = LineComments::new(1, "empty.rs".to_string());
        assert_eq!(line_comments.line, 1);
        assert_eq!(line_comments.path, "empty.rs");
        assert_eq!(line_comments.comment_count(), 0);
        assert!(!line_comments.has_pending());
    }

    #[test]
    fn test_comment_side_equality() {
        assert_eq!(CommentSide::Left, CommentSide::Left);
        assert_eq!(CommentSide::Right, CommentSide::Right);
        assert_ne!(CommentSide::Left, CommentSide::Right);
    }

    #[test]
    fn test_comment_side_can_be_copied() {
        let side = CommentSide::Left;
        let side_copy = side;
        assert_eq!(side, side_copy);
    }

    #[test]
    fn test_inline_comment_data_clone() {
        let comment = InlineCommentData::new(
            "user".to_string(),
            "body".to_string(),
            "file.rs".to_string(),
            10,
            1,
        );
        let cloned = comment.clone();
        assert_eq!(comment.id, cloned.id);
        assert_eq!(comment.author, cloned.author);
        assert_eq!(comment.body, cloned.body);
    }

    #[test]
    fn test_line_comments_clone() {
        let mut line_comments = LineComments::new(5, "test.rs".to_string());
        line_comments.add_comment(InlineCommentData::new(
            "user".to_string(),
            "comment".to_string(),
            "test.rs".to_string(),
            5,
            1,
        ));

        let cloned = line_comments.clone();
        assert_eq!(line_comments.line, cloned.line);
        assert_eq!(line_comments.path, cloned.path);
        assert_eq!(line_comments.comment_count(), cloned.comment_count());
    }

    #[test]
    fn test_line_comments_default() {
        let line_comments = LineComments::default();
        assert_eq!(line_comments.line, 0);
        assert_eq!(line_comments.path, "");
        assert_eq!(line_comments.comment_count(), 0);
    }

    #[test]
    fn test_inline_comment_with_reply_to() {
        let comment = InlineCommentData::new(
            "user".to_string(),
            "reply".to_string(),
            "file.rs".to_string(),
            10,
            2,
        )
        .with_reply_to(1);

        assert_eq!(comment.in_reply_to, Some(1));
    }

    #[test]
    fn test_inline_comment_pending_flag() {
        let comment = InlineCommentData::new(
            "user".to_string(),
            "pending comment".to_string(),
            "file.rs".to_string(),
            15,
            1,
        )
        .as_pending();

        assert!(comment.is_pending);
    }
}
