use github_client::FileChange;
use gpui::{App, FontWeight, Hsla, IntoElement, RenderOnce, Window};
use ui::prelude::*;

#[derive(IntoElement)]
pub struct PRDiffView {
    file_change: FileChange,
}

impl PRDiffView {
    pub fn new(file_change: FileChange) -> Self {
        Self { file_change }
    }

    fn parse_patch(&self) -> Vec<DiffLine> {
        let Some(ref patch) = self.file_change.patch else {
            return vec![];
        };

        let mut lines = vec![];
        let mut old_line_number = 0;
        let mut new_line_number = 0;

        for line in patch.lines() {
            if line.starts_with("@@") {
                if let Some(numbers) = Self::parse_hunk_header(line) {
                    old_line_number = numbers.0;
                    new_line_number = numbers.1;
                }
                lines.push(DiffLine {
                    old_line_number: None,
                    new_line_number: None,
                    content: line.to_string(),
                    line_type: DiffLineType::Header,
                });
            } else if line.starts_with('+') && !line.starts_with("+++") {
                lines.push(DiffLine {
                    old_line_number: None,
                    new_line_number: Some(new_line_number),
                    content: line[1..].to_string(),
                    line_type: DiffLineType::Added,
                });
                new_line_number += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines.push(DiffLine {
                    old_line_number: Some(old_line_number),
                    new_line_number: None,
                    content: line[1..].to_string(),
                    line_type: DiffLineType::Removed,
                });
                old_line_number += 1;
            } else if line.starts_with(' ') {
                lines.push(DiffLine {
                    old_line_number: Some(old_line_number),
                    new_line_number: Some(new_line_number),
                    content: line[1..].to_string(),
                    line_type: DiffLineType::Unchanged,
                });
                old_line_number += 1;
                new_line_number += 1;
            } else if !line.starts_with("+++") && !line.starts_with("---") {
                lines.push(DiffLine {
                    old_line_number: None,
                    new_line_number: None,
                    content: line.to_string(),
                    line_type: DiffLineType::Header,
                });
            }
        }

        lines
    }

    fn parse_hunk_header(header: &str) -> Option<(usize, usize)> {
        let header = header.trim_start_matches("@@").trim();
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let old_start = parts[0]
            .trim_start_matches('-')
            .split(',')
            .next()
            .and_then(|s| s.parse::<usize>().ok())?;

        let new_start = parts[1]
            .trim_start_matches('+')
            .split(',')
            .next()
            .and_then(|s| s.parse::<usize>().ok())?;

        Some((old_start, new_start))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DiffLine {
    old_line_number: Option<usize>,
    new_line_number: Option<usize>,
    content: String,
    line_type: DiffLineType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLineType {
    Header,
    Added,
    Removed,
    Unchanged,
}

impl DiffLineType {
    fn background_color(&self, cx: &App) -> Hsla {
        let theme = cx.theme().colors();
        match self {
            DiffLineType::Added => theme.version_control_added,
            DiffLineType::Removed => theme.version_control_deleted,
            DiffLineType::Header => theme.surface_background,
            DiffLineType::Unchanged => theme.editor_background,
        }
    }
}

impl RenderOnce for PRDiffView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let diff_lines = self.parse_patch();

        v_flex()
            .w_full()
            .gap_0()
            .child(
                h_flex()
                    .w_full()
                    .p_2()
                    .gap_2()
                    .bg(cx.theme().colors().title_bar_background)
                    .child(
                        Icon::new(IconName::File)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(self.file_change.filename.clone())
                            .size(LabelSize::Small)
                            .weight(FontWeight::BOLD),
                    ),
            )
            .child(
                v_flex()
                    .w_full()
                    .children(diff_lines.into_iter().map(|line| {
                        let bg_color = line.line_type.background_color(cx);

                        h_flex()
                            .w_full()
                            .h_6()
                            .bg(bg_color)
                            .child(
                                div()
                                    .w_12()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .border_r_1()
                                    .border_color(cx.theme().colors().border)
                                    .when_some(line.old_line_number, |this, num| {
                                        this.child(
                                            Label::new(num.to_string())
                                                .size(LabelSize::XSmall)
                                                .color(Color::Muted),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .w_12()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .border_r_1()
                                    .border_color(cx.theme().colors().border)
                                    .when_some(line.new_line_number, |this, num| {
                                        this.child(
                                            Label::new(num.to_string())
                                                .size(LabelSize::XSmall)
                                                .color(Color::Muted),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px_2()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .child(
                                        Label::new(line.content)
                                            .size(LabelSize::Small)
                                            .into_any_element(),
                                    ),
                            )
                    })),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::FileChangeStatus;

    #[test]
    fn test_parse_empty_patch() {
        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.rs".to_string(),
            status: FileChangeStatus::Modified,
            additions: 0,
            deletions: 0,
            changes: 0,
            patch: None,
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_parse_simple_addition() {
        let patch = r#"@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello");
     let x = 5;
 }"#;

        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.rs".to_string(),
            status: FileChangeStatus::Modified,
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: Some(patch.to_string()),
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();

        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].line_type, DiffLineType::Header);
        assert_eq!(lines[1].line_type, DiffLineType::Unchanged);
        assert_eq!(lines[2].line_type, DiffLineType::Added);
        assert_eq!(lines[2].content, "    println!(\"Hello\");");
        assert_eq!(lines[3].line_type, DiffLineType::Unchanged);
    }

    #[test]
    fn test_parse_simple_deletion() {
        let patch = r#"@@ -1,4 +1,3 @@
 fn main() {
-    println!("Hello");
     let x = 5;
 }"#;

        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.rs".to_string(),
            status: FileChangeStatus::Modified,
            additions: 0,
            deletions: 1,
            changes: 1,
            patch: Some(patch.to_string()),
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();

        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].line_type, DiffLineType::Header);
        assert_eq!(lines[1].line_type, DiffLineType::Unchanged);
        assert_eq!(lines[2].line_type, DiffLineType::Removed);
        assert_eq!(lines[2].content, "    println!(\"Hello\");");
        assert_eq!(lines[3].line_type, DiffLineType::Unchanged);
    }

    #[test]
    fn test_line_numbers_for_additions() {
        let patch = r#"@@ -1,2 +1,3 @@
 line 1
+added line
 line 2"#;

        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.txt".to_string(),
            status: FileChangeStatus::Modified,
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: Some(patch.to_string()),
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();

        assert_eq!(lines[1].old_line_number, Some(1));
        assert_eq!(lines[1].new_line_number, Some(1));
        assert_eq!(lines[2].old_line_number, None);
        assert_eq!(lines[2].new_line_number, Some(2));
        assert_eq!(lines[3].old_line_number, Some(2));
        assert_eq!(lines[3].new_line_number, Some(3));
    }

    #[test]
    fn test_line_numbers_for_deletions() {
        let patch = r#"@@ -1,3 +1,2 @@
 line 1
-deleted line
 line 2"#;

        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.txt".to_string(),
            status: FileChangeStatus::Modified,
            additions: 0,
            deletions: 1,
            changes: 1,
            patch: Some(patch.to_string()),
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();

        assert_eq!(lines[1].old_line_number, Some(1));
        assert_eq!(lines[1].new_line_number, Some(1));
        assert_eq!(lines[2].old_line_number, Some(2));
        assert_eq!(lines[2].new_line_number, None);
        assert_eq!(lines[3].old_line_number, Some(3));
        assert_eq!(lines[3].new_line_number, Some(2));
    }

    #[test]
    fn test_parse_hunk_header() {
        assert_eq!(
            PRDiffView::parse_hunk_header("@@ -1,3 +1,4 @@"),
            Some((1, 1))
        );
        assert_eq!(
            PRDiffView::parse_hunk_header("@@ -10,5 +20,7 @@ fn test()"),
            Some((10, 20))
        );
        assert_eq!(
            PRDiffView::parse_hunk_header("@@ -100 +200 @@"),
            Some((100, 200))
        );
    }

    #[gpui::test]
    fn test_diff_view_renders(cx: &mut App) {
        let patch = r#"@@ -1,2 +1,3 @@
 line 1
+added line
 line 2"#;

        let file_change = FileChange {
            sha: "abc123".to_string(),
            filename: "test.txt".to_string(),
            status: FileChangeStatus::Modified,
            additions: 1,
            deletions: 0,
            changes: 1,
            patch: Some(patch.to_string()),
            previous_filename: None,
        };

        let view = PRDiffView::new(file_change);
        let lines = view.parse_patch();
        assert_eq!(lines.len(), 4);
    }
}
