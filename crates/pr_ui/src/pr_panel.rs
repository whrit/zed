use gpui::{
    actions, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    Render, Window, px,
};
use panel::PanelHeader;
use ui::prelude::*;
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    Workspace,
};

use crate::TogglePRPanel;

actions!(pr_panel, [Refresh, Close]);

pub struct PRPanel {
    focus_handle: FocusHandle,
    width: Option<Pixels>,
}

impl PRPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            width: None,
        }
    }

    pub fn load(
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let panel = cx.new(|cx| Self::new(cx));

        workspace.update(cx, |workspace, cx| {
            workspace.add_panel(panel.clone(), window, cx);
        });

        panel
    }
}

pub fn register(workspace: &mut Workspace, _cx: &mut Context<Workspace>) {
    workspace.register_action(|workspace, _: &TogglePRPanel, window, cx| {
        workspace.toggle_panel_focus::<PRPanel>(window, cx);
    });
}

impl EventEmitter<PanelEvent> for PRPanel {}
impl EventEmitter<DismissEvent> for PRPanel {}

impl Focusable for PRPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PRPanel {
    fn persistent_name() -> &'static str {
        "PRPanel"
    }

    fn panel_key() -> &'static str {
        "pr_panel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn size(&self, _window: &Window, _cx: &App) -> Pixels {
        self.width.unwrap_or(px(360.0))
    }

    fn set_size(&mut self, size: Option<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.width = size;
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::GitPullRequest)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Pull Requests")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(TogglePRPanel)
    }

    fn activation_priority(&self) -> u32 {
        3
    }
}

impl PanelHeader for PRPanel {}

impl Render for PRPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("pr-panel")
            .key_context("PRPanel")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(
                self.panel_header_container(window, cx).child(
                    h_flex()
                        .w_full()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(Icon::new(IconName::GitPullRequest))
                                .child(Label::new("Pull Requests")),
                        ),
                ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .p_4()
                    .child(Label::new("No pull requests to display").color(Color::Muted)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn test_pr_panel_persistent_name() {
        assert_eq!(PRPanel::persistent_name(), "PRPanel");
    }

    #[gpui::test]
    fn test_pr_panel_panel_key() {
        assert_eq!(PRPanel::panel_key(), "pr_panel");
    }
}
