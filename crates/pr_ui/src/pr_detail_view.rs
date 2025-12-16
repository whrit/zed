use github_client::{CheckConclusion, CheckRun, CheckStatus, FileChange};
use gpui::{App, IntoElement, RenderOnce, Window};
use ui::prelude::*;

use crate::{PRDisplayState, PRFileListItem, PRStatusIndicator, PullRequestData, PullRequestState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Conversation,
    FilesChanged,
    Checks,
}

impl DetailTab {
    pub fn label(&self) -> &'static str {
        match self {
            DetailTab::Conversation => "Conversation",
            DetailTab::FilesChanged => "Files Changed",
            DetailTab::Checks => "Checks",
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            DetailTab::Conversation => IconName::Chat,
            DetailTab::FilesChanged => IconName::File,
            DetailTab::Checks => IconName::Check,
        }
    }
}

#[derive(IntoElement)]
pub struct PRDetailView {
    pull_request: PullRequestData,
    description: Option<String>,
    files: Vec<FileChange>,
    check_runs: Vec<CheckRun>,
    active_tab: DetailTab,
}

impl PRDetailView {
    pub fn new(pull_request: PullRequestData) -> Self {
        Self {
            pull_request,
            description: None,
            files: vec![],
            check_runs: vec![],
            active_tab: DetailTab::Conversation,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_files(mut self, files: Vec<FileChange>) -> Self {
        self.files = files;
        self
    }

    pub fn with_check_runs(mut self, check_runs: Vec<CheckRun>) -> Self {
        self.check_runs = check_runs;
        self
    }

    pub fn with_active_tab(mut self, tab: DetailTab) -> Self {
        self.active_tab = tab;
        self
    }

    fn state_to_display_state(state: &PullRequestState) -> PRDisplayState {
        match state {
            PullRequestState::Open => PRDisplayState::Open,
            PullRequestState::Closed => PRDisplayState::Closed,
            PullRequestState::Merged => PRDisplayState::Merged,
        }
    }

    fn render_conversation_tab(&self) -> AnyElement {
        v_flex()
            .p_4()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Author").size(LabelSize::Small).color(Color::Muted))
                    .child(Label::new(self.pull_request.author.clone())),
            )
            .when_some(self.description.clone(), |this, description| {
                this.child(
                    v_flex()
                        .gap_1()
                        .child(
                            Label::new("Description")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .child(Label::new(description)),
                )
            })
            .into_any_element()
    }

    fn render_files_tab(&self) -> AnyElement {
        v_flex()
            .p_4()
            .gap_2()
            .child(
                Label::new(format!("{} file(s) changed", self.files.len()))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(self.files.iter().map(|file| {
                PRFileListItem::new(file.clone(), false)
            }))
            .into_any_element()
    }

    fn render_checks_tab(&self, cx: &App) -> AnyElement {
        v_flex()
            .p_4()
            .gap_2()
            .child(
                Label::new(format!("{} check(s)", self.check_runs.len()))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(self.check_runs.iter().map(|check| {
                Self::render_check_run(check, cx)
            }))
            .into_any_element()
    }

    fn render_check_run(check: &CheckRun, cx: &App) -> impl IntoElement {
        let (icon, color) = match check.status {
            CheckStatus::Queued => (IconName::Ellipsis, Color::Muted),
            CheckStatus::InProgress => (IconName::LoadCircle, Color::Modified),
            CheckStatus::Completed => match check.conclusion {
                Some(CheckConclusion::Success) => (IconName::Check, Color::Success),
                Some(CheckConclusion::Failure) => (IconName::XCircle, Color::Error),
                Some(CheckConclusion::Neutral) => (IconName::Dash, Color::Muted),
                Some(CheckConclusion::Cancelled) => (IconName::XCircle, Color::Muted),
                Some(CheckConclusion::Skipped) => (IconName::Dash, Color::Muted),
                Some(CheckConclusion::TimedOut) => (IconName::Ellipsis, Color::Error),
                Some(CheckConclusion::ActionRequired) => (IconName::Info, Color::Modified),
                None => (IconName::Check, Color::Muted),
            },
        };

        h_flex()
            .w_full()
            .p_2()
            .gap_2()
            .items_center()
            .hover(|style| style.bg(cx.theme().colors().element_hover))
            .child(Icon::new(icon).size(IconSize::Small).color(color))
            .child(
                Label::new(check.name.clone())
                    .size(LabelSize::Small)
                    .into_any_element(),
            )
    }
}

impl RenderOnce for PRDetailView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tab_bar = h_flex()
            .w_full()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .children([DetailTab::Conversation, DetailTab::FilesChanged, DetailTab::Checks].iter().map(|tab| {
                let is_active = *tab == self.active_tab;
                h_flex()
                    .px_4()
                    .py_2()
                    .gap_2()
                    .items_center()
                    .when(is_active, |this| {
                        this.border_b_2()
                            .border_color(cx.theme().colors().border_selected)
                    })
                    .child(
                        Icon::new(tab.icon())
                            .size(IconSize::Small)
                            .color(if is_active { Color::Default } else { Color::Muted }),
                    )
                    .child(
                        Label::new(tab.label())
                            .size(LabelSize::Small)
                            .color(if is_active { Color::Default } else { Color::Muted }),
                    )
            }));

        let tab_content = match self.active_tab {
            DetailTab::Conversation => self.render_conversation_tab(),
            DetailTab::FilesChanged => self.render_files_tab(),
            DetailTab::Checks => self.render_checks_tab(cx),
        };

        v_flex()
            .w_full()
            .gap_0()
            .child(
                v_flex()
                    .gap_2()
                    .p_4()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new(self.pull_request.title.clone()))
                            .child(
                                Label::new(format!("#{}", self.pull_request.number))
                                    .color(Color::Muted)
                                    .size(LabelSize::Small),
                            ),
                    )
                    .child(PRStatusIndicator::with_label(Self::state_to_display_state(&self.pull_request.state))),
            )
            .child(tab_bar)
            .child(tab_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use github_client::FileChangeStatus;

    #[test]
    fn test_detail_tab_labels() {
        assert_eq!(DetailTab::Conversation.label(), "Conversation");
        assert_eq!(DetailTab::FilesChanged.label(), "Files Changed");
        assert_eq!(DetailTab::Checks.label(), "Checks");
    }

    #[test]
    fn test_detail_tab_icons() {
        assert_eq!(DetailTab::Conversation.icon(), IconName::Chat);
        assert_eq!(DetailTab::FilesChanged.icon(), IconName::File);
        assert_eq!(DetailTab::Checks.icon(), IconName::Check);
    }

    #[test]
    fn test_default_active_tab() {
        let pr = PullRequestData {
            number: 1,
            title: "Test PR".to_string(),
            author: "testuser".to_string(),
            state: PullRequestState::Open,
        };
        let view = PRDetailView::new(pr);
        assert_eq!(view.active_tab, DetailTab::Conversation);
    }

    #[test]
    fn test_with_active_tab() {
        let pr = PullRequestData {
            number: 1,
            title: "Test PR".to_string(),
            author: "testuser".to_string(),
            state: PullRequestState::Open,
        };
        let view = PRDetailView::new(pr).with_active_tab(DetailTab::FilesChanged);
        assert_eq!(view.active_tab, DetailTab::FilesChanged);
    }

    #[test]
    fn test_with_check_runs() {
        let pr = PullRequestData {
            number: 1,
            title: "Test PR".to_string(),
            author: "testuser".to_string(),
            state: PullRequestState::Open,
        };

        let check_runs = vec![
            CheckRun {
                id: 1,
                name: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Success),
                url: None,
            },
            CheckRun {
                id: 2,
                name: "Lint".to_string(),
                status: CheckStatus::InProgress,
                conclusion: None,
                url: None,
            },
        ];

        let view = PRDetailView::new(pr).with_check_runs(check_runs.clone());
        assert_eq!(view.check_runs.len(), 2);
        assert_eq!(view.check_runs[0].name, "CI");
        assert_eq!(view.check_runs[1].name, "Lint");
    }

    #[test]
    fn test_with_files() {
        let pr = PullRequestData {
            number: 1,
            title: "Test PR".to_string(),
            author: "testuser".to_string(),
            state: PullRequestState::Open,
        };

        let files = vec![
            FileChange {
                sha: "abc123".to_string(),
                filename: "test.rs".to_string(),
                status: FileChangeStatus::Modified,
                additions: 5,
                deletions: 2,
                changes: 7,
                patch: None,
                previous_filename: None,
            },
        ];

        let view = PRDetailView::new(pr).with_files(files);
        assert_eq!(view.files.len(), 1);
        assert_eq!(view.files[0].filename, "test.rs");
    }
}

