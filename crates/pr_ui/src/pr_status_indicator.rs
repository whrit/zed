use gpui::{App, IntoElement, RenderOnce, Window};
use ui::prelude::*;

/// The visual state of a pull request for display purposes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PRDisplayState {
    /// Pull request is open and ready for review
    Open,
    /// Pull request has been closed without merging
    Closed,
    /// Pull request has been successfully merged
    Merged,
    /// Pull request is in draft mode
    Draft,
}

impl PRDisplayState {
    /// Returns the icon to display for this state
    pub fn icon(&self) -> IconName {
        match self {
            PRDisplayState::Open => IconName::PullRequest,
            PRDisplayState::Closed => IconName::XCircle,
            PRDisplayState::Merged => IconName::Check,
            PRDisplayState::Draft => IconName::PullRequest,
        }
    }

    /// Returns the color to display for this state
    pub fn color(&self) -> Color {
        match self {
            PRDisplayState::Open => Color::Success,
            PRDisplayState::Closed => Color::Error,
            PRDisplayState::Merged => Color::Accent,
            PRDisplayState::Draft => Color::Muted,
        }
    }
}

/// A visual indicator showing the status of a pull request
#[derive(IntoElement)]
pub struct PRStatusIndicator {
    state: PRDisplayState,
    show_label: bool,
}

impl PRStatusIndicator {
    /// Creates a new status indicator with just an icon
    pub fn new(state: PRDisplayState) -> Self {
        Self {
            state,
            show_label: false,
        }
    }

    /// Creates a new status indicator with both an icon and text label
    pub fn with_label(state: PRDisplayState) -> Self {
        Self {
            state,
            show_label: true,
        }
    }

    fn label_text(&self) -> &'static str {
        match self.state {
            PRDisplayState::Open => "Open",
            PRDisplayState::Closed => "Closed",
            PRDisplayState::Merged => "Merged",
            PRDisplayState::Draft => "Draft",
        }
    }
}

impl RenderOnce for PRStatusIndicator {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let state_color = self.state.color();
        let state_icon = self.state.icon();

        if self.show_label {
            h_flex()
                .gap_1()
                .child(Icon::new(state_icon).size(IconSize::Small).color(state_color))
                .child(
                    Label::new(self.label_text())
                        .color(state_color)
                        .size(LabelSize::Small),
                )
                .into_any_element()
        } else {
            Icon::new(state_icon)
                .size(IconSize::Small)
                .color(state_color)
                .into_any_element()
        }
    }
}

