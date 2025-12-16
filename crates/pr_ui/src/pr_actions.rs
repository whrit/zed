use gpui::{prelude::*, App, Entity, EventEmitter, IntoElement, RenderOnce, Window};
use ui::{prelude::*, Button, ButtonStyle};

use crate::PullRequestState;

pub enum PRActionEvent {
    Closed,
    Reopened,
    ConvertedToDraft,
    MarkedReadyForReview,
    Error(String),
}

pub struct PRActionsData {
    pub state: PullRequestState,
    pub draft: bool,
    pub on_close: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    pub on_reopen: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    pub on_convert_to_draft: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    pub on_mark_ready: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl PRActionsData {
    pub fn new(state: PullRequestState, draft: bool) -> Self {
        Self {
            state,
            draft,
            on_close: None,
            on_reopen: None,
            on_convert_to_draft: None,
            on_mark_ready: None,
        }
    }

    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_close = Some(Box::new(callback));
        self
    }

    pub fn on_reopen<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_reopen = Some(Box::new(callback));
        self
    }

    pub fn on_convert_to_draft<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_convert_to_draft = Some(Box::new(callback));
        self
    }

    pub fn on_mark_ready<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_mark_ready = Some(Box::new(callback));
        self
    }
}

pub struct PRActions {
    state: PullRequestState,
    draft: bool,
    loading: bool,
}

impl PRActions {
    pub fn new(state: PullRequestState, draft: bool) -> Entity<Self> {
        |cx: &mut App| {
            cx.new(|_cx| Self {
                state,
                draft,
                loading: false,
            })
        }
    }

    pub fn set_loading(&mut self, loading: bool, cx: &mut App) {
        self.loading = loading;
        cx.notify();
    }

    pub fn update_state(&mut self, state: PullRequestState, draft: bool, cx: &mut App) {
        self.state = state;
        self.draft = draft;
        self.loading = false;
        cx.notify();
    }
}

impl EventEmitter<PRActionEvent> for PRActions {}

impl RenderOnce for PRActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let can_close = matches!(self.state, PullRequestState::Open);
        let can_reopen = matches!(self.state, PullRequestState::Closed);
        let can_convert_to_draft = matches!(self.state, PullRequestState::Open) && !self.draft;
        let can_mark_ready = matches!(self.state, PullRequestState::Open) && self.draft;

        h_flex()
            .gap_2()
            .when(can_close, |this| {
                this.child(
                    Button::new("close-pr", "Close PR")
                        .style(ButtonStyle::Subtle)
                        .disabled(self.loading),
                )
            })
            .when(can_reopen, |this| {
                this.child(
                    Button::new("reopen-pr", "Reopen PR")
                        .style(ButtonStyle::Subtle)
                        .disabled(self.loading),
                )
            })
            .when(can_convert_to_draft, |this| {
                this.child(
                    Button::new("convert-to-draft", "Convert to Draft")
                        .style(ButtonStyle::Subtle)
                        .disabled(self.loading),
                )
            })
            .when(can_mark_ready, |this| {
                this.child(
                    Button::new("mark-ready", "Mark Ready for Review")
                        .style(ButtonStyle::Subtle)
                        .disabled(self.loading),
                )
            })
    }
}

#[derive(IntoElement)]
pub struct PRActionsView {
    data: PRActionsData,
}

impl PRActionsView {
    pub fn new(data: PRActionsData) -> Self {
        Self { data }
    }
}

impl RenderOnce for PRActionsView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let can_close = matches!(self.data.state, PullRequestState::Open);
        let can_reopen = matches!(self.data.state, PullRequestState::Closed);
        let can_convert_to_draft =
            matches!(self.data.state, PullRequestState::Open) && !self.data.draft;
        let can_mark_ready = matches!(self.data.state, PullRequestState::Open) && self.data.draft;

        h_flex()
            .gap_2()
            .when(can_close, |this| {
                let on_close = self.data.on_close.as_ref().map(|cb| {
                    let cb = cb as *const _;
                    move |_event: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                        let cb = unsafe { &*(cb as *const Box<dyn Fn(&mut Window, &mut App)>) };
                        cb(window, cx);
                    }
                });

                let button = Button::new("close-pr", "Close PR").style(ButtonStyle::Subtle);
                if let Some(handler) = on_close {
                    this.child(button.on_click(handler))
                } else {
                    this.child(button)
                }
            })
            .when(can_reopen, |this| {
                let on_reopen = self.data.on_reopen.as_ref().map(|cb| {
                    let cb = cb as *const _;
                    move |_event: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                        let cb = unsafe { &*(cb as *const Box<dyn Fn(&mut Window, &mut App)>) };
                        cb(window, cx);
                    }
                });

                let button = Button::new("reopen-pr", "Reopen PR").style(ButtonStyle::Subtle);
                if let Some(handler) = on_reopen {
                    this.child(button.on_click(handler))
                } else {
                    this.child(button)
                }
            })
            .when(can_convert_to_draft, |this| {
                let on_convert = self.data.on_convert_to_draft.as_ref().map(|cb| {
                    let cb = cb as *const _;
                    move |_event: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                        let cb = unsafe { &*(cb as *const Box<dyn Fn(&mut Window, &mut App)>) };
                        cb(window, cx);
                    }
                });

                let button =
                    Button::new("convert-to-draft", "Convert to Draft").style(ButtonStyle::Subtle);
                if let Some(handler) = on_convert {
                    this.child(button.on_click(handler))
                } else {
                    this.child(button)
                }
            })
            .when(can_mark_ready, |this| {
                let on_mark_ready = self.data.on_mark_ready.as_ref().map(|cb| {
                    let cb = cb as *const _;
                    move |_event: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                        let cb = unsafe { &*(cb as *const Box<dyn Fn(&mut Window, &mut App)>) };
                        cb(window, cx);
                    }
                });

                let button = Button::new("mark-ready", "Mark Ready for Review")
                    .style(ButtonStyle::Subtle);
                if let Some(handler) = on_mark_ready {
                    this.child(button.on_click(handler))
                } else {
                    this.child(button)
                }
            })
    }
}
