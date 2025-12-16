use gpui::{App, IntoElement, RenderOnce, Window};
use ui::prelude::*;

#[derive(Clone, Debug)]
pub struct CommentData {
    pub id: u64,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub body: String,
    pub created_at: String,
    pub is_resolved: bool,
}

#[derive(Clone, Debug)]
pub struct CommentThreadData {
    pub id: u64,
    pub path: String,
    pub line: Option<u32>,
    pub comments: Vec<CommentData>,
    pub is_resolved: bool,
    pub is_outdated: bool,
}

#[derive(IntoElement)]
pub struct CommentView {
    comment: CommentData,
    is_first: bool,
}

impl CommentView {
    pub fn new(comment: CommentData, is_first: bool) -> Self {
        Self { comment, is_first }
    }

    fn author_initials(&self) -> String {
        self.comment.author.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()
    }
}

impl RenderOnce for CommentView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().colors();
        let initials = self.author_initials();

        let avatar = div().flex_none().w_8().h_8().rounded_full().bg(theme.element_background).border_1().border_color(theme.border).flex().items_center().justify_center().child(Label::new(initials).size(LabelSize::Small).color(Color::Muted));

        let header = h_flex().gap_2().child(Label::new(self.comment.author.clone()).size(LabelSize::Small)).child(Label::new(self.comment.created_at.clone()).size(LabelSize::Small).color(Color::Muted));
        let body = div().child(Label::new(self.comment.body).size(LabelSize::Small));
        let content = v_flex().flex_1().gap_1().child(header).child(body);

        let mut row = h_flex().gap_2().p_2();
        if !self.is_first {
            row = row.border_t_1().border_color(theme.border);
        }
        row.child(avatar).child(content)
    }
}

#[derive(IntoElement)]
pub struct CommentThreadView {
    thread: CommentThreadData,
    show_file_path: bool,
    on_resolve: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_unresolve: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_reply: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl CommentThreadView {
    pub fn new(thread: CommentThreadData) -> Self {
        Self { thread, show_file_path: false, on_resolve: None, on_unresolve: None, on_reply: None }
    }

    pub fn show_file_path(mut self, show: bool) -> Self {
        self.show_file_path = show;
        self
    }

    pub fn on_resolve(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_resolve = Some(Box::new(handler));
        self
    }

    pub fn on_unresolve(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_unresolve = Some(Box::new(handler));
        self
    }

    pub fn on_reply(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reply = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CommentThreadView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().colors();
        let is_resolved = self.thread.is_resolved;
        let is_outdated = self.thread.is_outdated;
        let comment_count = self.thread.comments.len();
        let comments: Vec<_> = self.thread.comments.iter().enumerate().map(|(i, c)| CommentView::new(c.clone(), i == 0)).collect();

        let (status_icon, status_color, status_text) = if is_resolved {
            (IconName::Check, Color::Success, "Resolved")
        } else if is_outdated {
            (IconName::Warning, Color::Warning, "Outdated")
        } else {
            (IconName::Chat, Color::Muted, "Active")
        };
        let status = h_flex().gap_1().px_2().py_1().rounded_md().child(Icon::new(status_icon).size(IconSize::Small).color(status_color)).child(Label::new(status_text).size(LabelSize::Small).color(status_color));

        let count_label = Label::new(format!("{} comments", comment_count)).size(LabelSize::Small).color(Color::Muted);
        let header_row = h_flex().justify_between().items_center().p_2().border_b_1().border_color(theme.border).child(count_label).child(status);

        let comments_container = v_flex().children(comments);

        let mut buttons = h_flex().gap_2().p_2().border_t_1().border_color(theme.border);
        if let Some(handler) = self.on_reply {
            let btn = Button::new("reply-button", "Reply").style(ButtonStyle::Subtle).icon(IconName::ReplyArrowRight).icon_size(IconSize::Small).on_click(move |_event, window, cx| handler(window, cx));
            buttons = buttons.child(btn);
        }
        if is_resolved {
            if let Some(handler) = self.on_unresolve {
                let btn = Button::new("unresolve-button", "Unresolve").style(ButtonStyle::Subtle).icon(IconName::XCircle).icon_size(IconSize::Small).on_click(move |_event, window, cx| handler(window, cx));
                buttons = buttons.child(btn);
            }
        } else if let Some(handler) = self.on_resolve {
            let btn = Button::new("resolve-button", "Resolve").style(ButtonStyle::Subtle).icon(IconName::Check).icon_size(IconSize::Small).on_click(move |_event, window, cx| handler(window, cx));
            buttons = buttons.child(btn);
        }

        let mut container = v_flex().bg(theme.surface_background).border_1().border_color(theme.border).rounded_md().overflow_hidden();
        if self.show_file_path {
            let location = if let Some(line) = self.thread.line { format!("{}:{}", self.thread.path, line) } else { self.thread.path.clone() };
            let path_header = h_flex().gap_1().p_2().border_b_1().border_color(gpui::rgb(0x3a3a3a)).child(Icon::new(IconName::FileTextOutlined).size(IconSize::Small).color(Color::Muted)).child(Label::new(location).size(LabelSize::Small).color(Color::Muted));
            container = container.child(path_header);
        }
        container.child(header_row).child(comments_container).child(buttons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_comment(id: u64, author: &str, body: &str) -> CommentData {
        CommentData { id, author: author.to_string(), author_avatar_url: None, body: body.to_string(), created_at: "2 hours ago".to_string(), is_resolved: false }
    }

    fn create_test_thread(id: u64, path: &str, line: Option<u32>) -> CommentThreadData {
        CommentThreadData { id, path: path.to_string(), line, comments: vec![], is_resolved: false, is_outdated: false }
    }

    #[test]
    fn test_comment_data_creation() {
        let comment = create_test_comment(1, "John Doe", "This is a test comment");
        assert_eq!(comment.id, 1);
        assert_eq!(comment.author, "John Doe");
        assert_eq!(comment.body, "This is a test comment");
    }

    #[test]
    fn test_comment_thread_data_creation() {
        let thread = create_test_thread(1, "src/main.rs", Some(42));
        assert_eq!(thread.id, 1);
        assert_eq!(thread.path, "src/main.rs");
        assert_eq!(thread.line, Some(42));
    }

    #[test]
    fn test_comment_thread_with_multiple_comments() {
        let mut thread = create_test_thread(1, "src/lib.rs", Some(10));
        thread.comments = vec![create_test_comment(1, "Alice", "First"), create_test_comment(2, "Bob", "Second")];
        assert_eq!(thread.comments.len(), 2);
    }

    #[test]
    fn test_comment_view_creation() {
        let comment = create_test_comment(1, "Test User", "Test body");
        let view = CommentView::new(comment, true);
        assert!(view.is_first);
    }

    #[test]
    fn test_author_initials() {
        let comment = create_test_comment(1, "John Doe", "test");
        let view = CommentView::new(comment, true);
        assert_eq!(view.author_initials(), "JD");
    }

    #[test]
    fn test_comment_thread_view_creation() {
        let thread = create_test_thread(1, "src/main.rs", Some(10));
        let view = CommentThreadView::new(thread);
        assert!(!view.show_file_path);
    }

    #[test]
    fn test_comment_thread_view_show_file_path() {
        let thread = create_test_thread(1, "src/test.rs", None);
        let view = CommentThreadView::new(thread).show_file_path(true);
        assert!(view.show_file_path);
    }

    #[test]
    fn test_comment_thread_view_with_handlers() {
        let thread = create_test_thread(1, "src/lib.rs", Some(42));
        let view = CommentThreadView::new(thread).on_resolve(|_, _| {}).on_unresolve(|_, _| {}).on_reply(|_, _| {});
        assert!(view.on_resolve.is_some());
        assert!(view.on_unresolve.is_some());
        assert!(view.on_reply.is_some());
    }

    #[test]
    fn test_comment_data_cloneable() {
        let comment = create_test_comment(1, "Clone Test", "Original");
        let cloned = comment.clone();
        assert_eq!(comment.id, cloned.id);
    }

    #[test]
    fn test_comment_thread_data_cloneable() {
        let thread = create_test_thread(1, "clone.rs", Some(5));
        let cloned = thread.clone();
        assert_eq!(thread.id, cloned.id);
    }
}
