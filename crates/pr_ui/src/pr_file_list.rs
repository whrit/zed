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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(filename: &str, status: FileChangeStatus, additions: usize, deletions: usize) -> FileChange {
        FileChange {
            sha: "test-sha".to_string(),
            filename: filename.to_string(),
            status,
            additions,
            deletions,
            changes: additions + deletions,
            patch: None,
            previous_filename: None,
        }
    }

    #[test]
    fn test_status_icon_added() {
        let file = create_test_file("new.rs", FileChangeStatus::Added, 10, 0);
        let item = PRFileListItem::new(file, false);
        let (icon, color) = item.status_icon();

        assert_eq!(icon, IconName::SquarePlus);
        assert_eq!(color, Color::Success);
    }

    #[test]
    fn test_status_icon_removed() {
        let file = create_test_file("old.rs", FileChangeStatus::Removed, 0, 10);
        let item = PRFileListItem::new(file, false);
        let (icon, color) = item.status_icon();

        assert_eq!(icon, IconName::SquareMinus);
        assert_eq!(color, Color::Error);
    }

    #[test]
    fn test_status_icon_modified() {
        let file = create_test_file("changed.rs", FileChangeStatus::Modified, 5, 3);
        let item = PRFileListItem::new(file, false);
        let (icon, color) = item.status_icon();

        assert_eq!(icon, IconName::SquareDot);
        assert_eq!(color, Color::Modified);
    }

    #[test]
    fn test_status_icon_renamed() {
        let file = create_test_file("renamed.rs", FileChangeStatus::Renamed, 0, 0);
        let item = PRFileListItem::new(file, false);
        let (icon, color) = item.status_icon();

        assert_eq!(icon, IconName::ArrowRight);
        assert_eq!(color, Color::Modified);
    }

    #[test]
    fn test_file_list_creation() {
        let files = vec![
            create_test_file("file1.rs", FileChangeStatus::Added, 10, 0),
            create_test_file("file2.rs", FileChangeStatus::Modified, 5, 3),
        ];

        let file_list = PRFileList::new(files);

        assert_eq!(file_list.files.len(), 2);
        assert_eq!(file_list.selected_index, None);
    }

    #[test]
    fn test_select_file() {
        let files = vec![
            create_test_file("file1.rs", FileChangeStatus::Added, 10, 0),
            create_test_file("file2.rs", FileChangeStatus::Modified, 5, 3),
        ];

        let mut file_list = PRFileList::new(files);

        file_list.select_file(1);
        assert_eq!(file_list.selected_index, Some(1));
        assert_eq!(file_list.selected_file().unwrap().filename, "file2.rs");
    }

    #[test]
    fn test_select_file_out_of_bounds() {
        let files = vec![
            create_test_file("file1.rs", FileChangeStatus::Added, 10, 0),
        ];

        let mut file_list = PRFileList::new(files);

        file_list.select_file(5);
        assert_eq!(file_list.selected_index, None);
    }

    #[test]
    fn test_selected_file_none() {
        let files = vec![
            create_test_file("file1.rs", FileChangeStatus::Added, 10, 0),
        ];

        let file_list = PRFileList::new(files);

        assert_eq!(file_list.selected_file(), None);
    }

    #[test]
    fn test_file_list_item_creation() {
        let file = create_test_file("test.rs", FileChangeStatus::Modified, 10, 5);
        let item = PRFileListItem::new(file, false);

        assert!(!item.is_selected);
        assert_eq!(item.file.filename, "test.rs");
    }

    #[test]
    fn test_file_list_scroll_handle() {
        let files = vec![
            create_test_file("file1.rs", FileChangeStatus::Added, 10, 0),
            create_test_file("file2.rs", FileChangeStatus::Modified, 5, 3),
        ];

        let file_list = PRFileList::new(files);
        assert_eq!(file_list.files.len(), 2);
    }
}
