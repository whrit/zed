#![recursion_limit = "4096"]

use gpui::{actions, App};
use workspace::Workspace;

mod github_auth_modal;
mod github_remote;
mod inline_comment;
mod pr_actions;
mod pr_editor_integration;

pub use github_auth_modal::{CancelAuth, GitHubAuthEvent, GitHubAuthModal, StartAuth};
pub use github_remote::GitHubRepo;
mod pr_checkout;
mod pr_comment_thread;
mod pr_create_modal;
mod pr_detail_view;
mod pr_diff_view;
mod pr_file_list;
mod pr_label_picker;
mod pr_list_item;
mod pr_merge_modal;
mod pr_panel;
mod pr_review_modal;
mod pr_review_panel;
mod pr_status_indicator;
mod pr_user_picker;

pub use inline_comment::{
    CommentGutterIndicator, CommentSide, InlineCommentBadge, InlineCommentData,
    InlineCommentView, LineComments,
};
pub use pr_actions::{PRActionEvent, PRActions, PRActionsData, PRActionsView};
pub use pr_checkout::{checkout_pull_request, CheckoutPullRequest, CheckoutPullRequestParams};
pub use pr_comment_thread::{CommentData, CommentThreadData, CommentThreadView, CommentView};
pub use pr_create_modal::{CancelCreate, CreatePRModal, CreatePullRequest, SubmitPR};
pub use pr_detail_view::{DetailTab, PRDetailView};
pub use pr_diff_view::PRDiffView;
pub use pr_editor_integration::{
    AddPRComment, GlobalPREditorIntegration, PRCommentContextMenu, PREditorIntegration,
    PREditorState, get_pr_editor_state, update_pr_editor_state,
};
pub use pr_file_list::{PRFileList, PRFileListItem};
pub use pr_list_item::{PRListItem, PullRequestData, PullRequestState};
pub use pr_merge_modal::{CancelMerge, ConfirmMerge, MergePRModal, MergePullRequest};
pub use pr_panel::PRPanel;
pub use pr_review_modal::{
    CancelReview, PendingReviewSummary, ReviewAction, SubmitReview, SubmitReviewModal,
};
pub use pr_review_panel::{
    DiscardReview, PRCommentStore, PRReviewPanel, PRReviewPanelEvent, PendingComment,
    ReviewSessionState, StartReview, ToggleReviewPanel,
};
pub use pr_status_indicator::{PRDisplayState, PRStatusIndicator};
pub use pr_user_picker::UserPicker;
pub use pr_label_picker::LabelPicker;

actions!(pr_ui, [TogglePRPanel]);

pub fn init(cx: &mut App) {
    pr_editor_integration::init(cx);
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        pr_panel::register(workspace, cx);
        pr_review_panel::register(workspace, cx);
    })
    .detach();
}

