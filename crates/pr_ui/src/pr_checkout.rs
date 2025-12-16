use anyhow::{Context as _, Result};
use gpui::{actions, AsyncApp, Task, WeakEntity};
use workspace::Workspace;

actions!(pr_checkout, [CheckoutPullRequest]);

#[derive(Clone, Debug, PartialEq)]
pub struct CheckoutPullRequestParams {
    pub pr_number: u32,
    pub head_ref: String,
}

/// Checkout a pull request branch locally
///
/// This function:
/// 1. Gets the active repository from the workspace
/// 2. Uses the existing change_branch function to fetch and checkout the PR branch
pub fn checkout_pull_request(
    workspace: WeakEntity<Workspace>,
    params: CheckoutPullRequestParams,
    cx: &mut AsyncApp,
) -> Task<Result<()>> {
    cx.spawn(async move |cx| {
        let project = workspace
            .read_with(cx, |workspace, _| workspace.project().clone())
            .context("Failed to read workspace")?;

        let repository = project
            .read_with(cx, |project, cx| project.active_repository(cx))
            .context("Failed to read project")?
            .context("No active repository")?;

        repository
            .update(cx, |repo, _| repo.change_branch(params.head_ref.clone()))?
            .await??;

        anyhow::Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_defined() {
        // Test that the action is properly defined
        let _action = CheckoutPullRequest;
    }

    #[test]
    fn test_checkout_params_creation() {
        let params = CheckoutPullRequestParams {
            pr_number: 123,
            head_ref: "feature/test-branch".to_string(),
        };

        assert_eq!(params.pr_number, 123);
        assert_eq!(params.head_ref, "feature/test-branch");
    }

    #[test]
    fn test_checkout_params_clone() {
        let params = CheckoutPullRequestParams {
            pr_number: 456,
            head_ref: "fix/bug-fix".to_string(),
        };

        let cloned = params.clone();
        assert_eq!(params, cloned);
    }

    #[test]
    fn test_checkout_params_debug() {
        let params = CheckoutPullRequestParams {
            pr_number: 789,
            head_ref: "main".to_string(),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("789"));
        assert!(debug_str.contains("main"));
    }
}
