use github_client::{FileChange, FileChangeStatus};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, UniformListScrollHandle,
    Window, uniform_list,
};
use ui::{
    prelude::FluentBuilder, Color, Icon, IconName, Label, LabelCommon, LabelSize, div, h_flex, v_flex,
};

#[derive(Clone)]
pub struct PRFileListItem {
    file: FileChange,
    is_selected: bool,
}

impl PRFileListItem {
    pub fn new(file: FileChange, is_selected: bool) -> Self {
        Self { file, is_selected }
    }

    fn status_icon(&self) -> (IconName, Color) {
        match self.file.status {
            FileChangeStatus::Added => (IconName::SquarePlus, Color::Success),
            FileChangeStatus::Removed => (IconName::SquareMinus, Color::Error),
            FileChangeStatus::Modified => (IconName::SquareDot, Color::Modified),
            FileChangeStatus::Renamed => (IconName::ArrowRight, Color::Modified),
            FileChangeStatus::Copied => (IconName::Copy, Color::Modified),
            FileChangeStatus::Changed => (IconName::SquareDot, Color::Modified),
            FileChangeStatus::Unchanged => (IconName::Check, Color::Muted),
        }
    }
}

impl IntoElement for PRFileListItem {
    type Element = gpui::Stateful<gpui::Div>;

    fn into_element(self) -> Self::Element {
        let (icon_name, icon_color) = self.status_icon();

        div()
            .id(SharedString::from(format!("pr-file-{}", self.file.filename)))
            .h_7()
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .when(self.is_selected, |this| {
                this.bg(gpui::blue())
            })
            .child(
                Icon::new(icon_name)
                    .color(icon_color)
                    .size(ui::IconSize::Small),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        Label::new(self.file.filename.clone())
                            .size(LabelSize::Small)
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(format!("+{}", self.file.additions))
                            .size(LabelSize::Small)
                            .color(Color::Success),
                    )
                    .child(
                        Label::new(format!("-{}", self.file.deletions))
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    ),
            )
    }
}

pub struct PRFileList {
    files: Vec<FileChange>,
    selected_index: Option<usize>,
    scroll_handle: UniformListScrollHandle,
}

impl PRFileList {
    pub fn new(files: Vec<FileChange>) -> Self {
        Self {
            files,
            selected_index: None,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    pub fn select_file(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn selected_file(&self) -> Option<&FileChange> {
        self.selected_index
            .and_then(|index| self.files.get(index))
    }
}

impl Render for PRFileList {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let file_count = self.files.len();
        let selected_index = self.selected_index;

        v_flex()
            .size_full()
            .child({
                let files = self.files.clone();
                uniform_list(
                    "pr-file-list",
                    file_count,
                    move |range, _window, _cx| {
                        let mut items = Vec::with_capacity(range.end - range.start);
                        for index in range {
                            if let Some(file) = files.get(index) {
                                let is_selected = selected_index == Some(index);
                                items.push(PRFileListItem::new(file.clone(), is_selected));
                            }
                        }
                        items
                    },
                )
                .flex_1()
                .size_full()
                .track_scroll(&self.scroll_handle)
            })
    }
}

