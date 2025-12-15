use gpui::{actions, App};
use workspace::Workspace;

mod pr_list_item;
mod pr_panel;

pub use pr_list_item::{PRListItem, PullRequestData, PullRequestState};
pub use pr_panel::PRPanel;

actions!(pr_ui, [TogglePRPanel]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        pr_panel::register(workspace, cx);
    })
    .detach();
}

#[cfg(test)]
mod tests {
    #[gpui::test]
    fn test_init_compiles() {
        // This test just ensures the init function compiles correctly
        // The actual functionality would be tested in integration tests
    }
}
