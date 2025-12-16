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

