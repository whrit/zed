use gpui::{App, IntoElement, RenderOnce, Window};
use ui::prelude::*;

#[derive(Clone, Debug)]
pub struct PullRequestData {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub state: PullRequestState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

impl PullRequestState {
    pub fn icon(&self) -> IconName {
        match self {
            PullRequestState::Open => IconName::PullRequest,
            PullRequestState::Closed => IconName::XCircle,
            PullRequestState::Merged => IconName::Check,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PullRequestState::Open => Color::Success,
            PullRequestState::Closed => Color::Error,
            PullRequestState::Merged => Color::Accent,
        }
    }
}

#[derive(IntoElement)]
pub struct PRListItem {
    pull_request: PullRequestData,
}

impl PRListItem {
    pub fn new(pull_request: PullRequestData) -> Self {
        Self { pull_request }
    }
}

impl RenderOnce for PRListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state_color = self.pull_request.state.color();

        h_flex()
            .w_full()
            .p_2()
            .gap_2()
            .hover(|style| style.bg(cx.theme().colors().element_hover))
            .child(
                Icon::new(self.pull_request.state.icon())
                    .size(IconSize::Small)
                    .color(state_color),
            )
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_1()
                            .child(Label::new(self.pull_request.title.clone()))
                            .child(
                                Label::new(format!("#{}", self.pull_request.number))
                                    .color(Color::Muted)
                                    .size(LabelSize::Small),
                            ),
                    )
                    .child(
                        Label::new(format!("by {}", self.pull_request.author))
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    ),
            )
    }
}

