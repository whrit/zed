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

