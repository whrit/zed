use github_client::FileChange;
use gpui::{App, IntoElement, RenderOnce, Window};
use ui::{prelude::*, Divider};

use crate::{PRDisplayState, PRStatusIndicator, PullRequestData, PullRequestState};

#[derive(IntoElement)]
pub struct PRDetailView {
    pull_request: PullRequestData,
    description: Option<String>,
    files: Vec<FileChange>,
}

impl PRDetailView {
    pub fn new(pull_request: PullRequestData) -> Self {
        Self {
            pull_request,
            description: None,
            files: vec![],
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

    fn state_to_display_state(state: &PullRequestState) -> PRDisplayState {
        match state {
            PullRequestState::Open => PRDisplayState::Open,
            PullRequestState::Closed => PRDisplayState::Closed,
            PullRequestState::Merged => PRDisplayState::Merged,
        }
    }
}

impl RenderOnce for PRDetailView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .p_4()
            .child(
                v_flex()
                    .gap_2()
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
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Author").size(LabelSize::Small).color(Color::Muted))
                    .child(Label::new(self.pull_request.author.clone())),
            )
            .when_some(self.description, |this, description| {
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
            .child(Divider::horizontal())
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new("Changed Files")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(format!("{} file(s) changed", self.files.len()))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
    }
}

