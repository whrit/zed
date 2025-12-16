#![recursion_limit = "4096"]

use gpui::{actions, App};
use workspace::Workspace;

mod inline_comment;
mod pr_checkout;
mod pr_comment_thread;
mod pr_create_modal;
mod pr_detail_view;
mod pr_file_list;
mod pr_list_item;
mod pr_merge_modal;
mod pr_panel;
mod pr_review_modal;
mod pr_review_panel;
mod pr_status_indicator;

pub use inline_comment::{
    CommentGutterIndicator, CommentSide, InlineCommentBadge, InlineCommentData,
    InlineCommentView, LineComments,
};
pub use pr_checkout::{checkout_pull_request, CheckoutPullRequest, CheckoutPullRequestParams};
pub use pr_comment_thread::{CommentData, CommentThreadData, CommentThreadView, CommentView};
pub use pr_create_modal::{CancelCreate, CreatePRModal, CreatePullRequest, SubmitPR};
pub use pr_detail_view::PRDetailView;
pub use pr_file_list::{PRFileList, PRFileListItem};
pub use pr_list_item::{PRListItem, PullRequestData, PullRequestState};
pub use pr_merge_modal::{CancelMerge, ConfirmMerge, MergePRModal, MergePullRequest};
pub use pr_panel::PRPanel;
pub use pr_review_modal::{
    CancelReview, PendingReviewSummary, ReviewAction, SubmitReview, SubmitReviewModal,
};
pub use pr_review_panel::{
    DiscardReview, PRReviewPanel, PendingComment, ReviewSessionState, StartReview,
    ToggleReviewPanel,
};
pub use pr_status_indicator::{PRDisplayState, PRStatusIndicator};

actions!(pr_ui, [TogglePRPanel]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        pr_panel::register(workspace, cx);
        pr_review_panel::register(workspace, cx);
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_actions() {
        let _toggle = TogglePRPanel;
        let _checkout = CheckoutPullRequest;
        let _create = CreatePullRequest;
        let _merge = MergePullRequest;
        let _submit = SubmitPR;
        let _cancel = CancelCreate;
    }

    #[test]
    fn test_module_exports_types() {
        use crate::PullRequestState;

        let _pr_data = PullRequestData {
            number: 1,
            title: "Test".to_string(),
            author: "test".to_string(),
            state: PullRequestState::Open,
        };

        let _checkout_params = CheckoutPullRequestParams {
            pr_number: 1,
            head_ref: "test".to_string(),
        };
    }

    #[test]
    fn test_render_once_types() {
        let pr_data = PullRequestData {
            number: 1,
            title: "Test".to_string(),
            author: "test".to_string(),
            state: PullRequestState::Open,
        };

        let _list_item = PRListItem::new(pr_data);
        let _status_indicator = PRStatusIndicator::new(PRDisplayState::Open);
    }
}
