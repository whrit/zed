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

