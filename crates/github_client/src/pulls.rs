use crate::{
    CheckRunList, Commit, CreatePRParams, FileChange, GitHubClient, MergePRParams, MergeResult,
    PRState, PullRequest, PullRequestDetail, RequestReviewersParams, ReviewRequest, UpdatePRParams,
};
use anyhow::Result;

impl GitHubClient {
    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: PRState,
    ) -> Result<Vec<PullRequest>> {
        let path = format!("/repos/{}/{}/pulls?state={}", owner, repo, state.as_str());
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
    ) -> Result<PullRequestDetail> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        params: CreatePRParams,
    ) -> Result<PullRequest> {
        let path = format!("/repos/{}/{}/pulls", owner, repo);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn update_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
        params: UpdatePRParams,
    ) -> Result<PullRequest> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, number);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::PATCH, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn list_pull_request_files(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
    ) -> Result<Vec<FileChange>> {
        let path = format!("/repos/{}/{}/pulls/{}/files", owner, repo, number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn list_pull_request_commits(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
    ) -> Result<Vec<Commit>> {
        let path = format!("/repos/{}/{}/pulls/{}/commits", owner, repo, number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn merge_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
        params: MergePRParams,
    ) -> Result<MergeResult> {
        let path = format!("/repos/{}/{}/pulls/{}/merge", owner, repo, number);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::PUT, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn request_reviewers(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
        params: RequestReviewersParams,
    ) -> Result<PullRequest> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            owner, repo, number
        );
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn get_requested_reviewers(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
    ) -> Result<ReviewRequest> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            owner, repo, number
        );
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn get_check_runs(
        &self,
        owner: &str,
        repo: &str,
        git_ref: &str,
    ) -> Result<CheckRunList> {
        let path = format!("/repos/{}/{}/commits/{}/check-runs", owner, repo, git_ref);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CheckConclusion, CheckRun, CheckStatus, CommitAuthor, CommitInfo, FileChangeStatus,
        MergeMethod, RefData, Team, User,
    };
    use chrono::Utc;
    use http_client::FakeHttpClient;

    fn make_test_pr(number: u32, title: &str) -> PullRequest {
        PullRequest {
            number,
            title: title.to_string(),
            state: PRState::Open,
            author: User {
                id: 123,
                login: "testuser".to_string(),
                avatar_url: "https://example.com/avatar.png".to_string(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            draft: false,
            url: format!("https://github.com/owner/repo/pull/{}", number),
            head_ref_data: RefData {
                ref_name: "feature-branch".to_string(),
            },
            base_ref_data: RefData {
                ref_name: "main".to_string(),
            },
        }
    }

    #[gpui::test]
    async fn test_list_pull_requests() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/pulls"));

            let prs = vec![make_test_pr(1, "Test PR")];
            let body = serde_json::to_vec(&prs).expect("serialize prs");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client
            .list_pull_requests("owner", "repo", PRState::Open)
            .await;

        assert!(result.is_ok());
        let prs = result.expect("get prs");
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 1);
        assert_eq!(prs[0].title, "Test PR");
    }

    #[gpui::test]
    async fn test_get_pull_request() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/pulls/42"));

            let pr_detail = PullRequestDetail {
                number: 42,
                title: "Detailed PR".to_string(),
                state: PRState::Open,
                author: User {
                    id: 456,
                    login: "contributor".to_string(),
                    avatar_url: "https://example.com/avatar2.png".to_string(),
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                draft: false,
                url: "https://github.com/owner/repo/pull/42".to_string(),
                head_ref_data: RefData {
                    ref_name: "fix-bug".to_string(),
                },
                base_ref_data: RefData {
                    ref_name: "main".to_string(),
                },
                body: Some("PR description".to_string()),
                additions: 100,
                deletions: 50,
                changed_files: 5,
                mergeable: Some(true),
                merged: false,
            };

            let body = serde_json::to_vec(&pr_detail).expect("serialize pr_detail");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_pull_request("owner", "repo", 42).await;

        assert!(result.is_ok());
        let pr = result.expect("get pr");
        assert_eq!(pr.number, 42);
        assert_eq!(pr.additions, 100);
        assert_eq!(pr.deletions, 50);
    }

    #[gpui::test]
    async fn test_create_pull_request() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request.uri().path().contains("/repos/owner/repo/pulls"));

            let pr = make_test_pr(99, "New PR");
            let body = serde_json::to_vec(&pr).expect("serialize pr");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreatePRParams {
            title: "New PR".to_string(),
            head: "new-feature".to_string(),
            base: "main".to_string(),
            body: Some("PR body".to_string()),
            draft: false,
        };

        let result = client.create_pull_request("owner", "repo", params).await;

        assert!(result.is_ok());
        let pr = result.expect("get pr");
        assert_eq!(pr.number, 99);
    }

    #[gpui::test]
    async fn test_list_pull_requests_error() {
        let http_client = FakeHttpClient::create(|_request| async move {
            Ok(http::Response::builder()
                .status(404)
                .body("Not Found".as_bytes().to_vec().into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client
            .list_pull_requests("owner", "repo", PRState::Open)
            .await;

        assert!(result.is_err());
    }

    #[gpui::test]
    async fn test_update_pull_request() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::PATCH);
            assert!(request.uri().path().contains("/repos/owner/repo/pulls/42"));

            let pr = make_test_pr(42, "Updated Title");
            let body = serde_json::to_vec(&pr).expect("serialize pr");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = UpdatePRParams {
            title: Some("Updated Title".to_string()),
            body: None,
            state: None,
            base: None,
        };

        let result = client.update_pull_request("owner", "repo", 42, params).await;

        assert!(result.is_ok());
        let pr = result.expect("get pr");
        assert_eq!(pr.number, 42);
    }

    #[gpui::test]
    async fn test_list_pull_request_files() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/files"));

            let files = vec![FileChange {
                sha: "abc123".to_string(),
                filename: "src/main.rs".to_string(),
                status: FileChangeStatus::Modified,
                additions: 10,
                deletions: 5,
                changes: 15,
                patch: Some("@@ -1,5 +1,10 @@".to_string()),
                previous_filename: None,
            }];

            let body = serde_json::to_vec(&files).expect("serialize files");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_pull_request_files("owner", "repo", 42).await;

        assert!(result.is_ok());
        let files = result.expect("get files");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "src/main.rs");
        assert_eq!(files[0].additions, 10);
    }

    #[gpui::test]
    async fn test_list_pull_request_commits() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/commits"));

            let commits = vec![Commit {
                sha: "abc123".to_string(),
                commit: CommitInfo {
                    message: "Fix bug".to_string(),
                    author: CommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: Utc::now(),
                    },
                    committer: CommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: Utc::now(),
                    },
                },
                author: Some(User {
                    id: 123,
                    login: "testuser".to_string(),
                    avatar_url: "https://example.com/avatar.png".to_string(),
                }),
                committer: None,
            }];

            let body = serde_json::to_vec(&commits).expect("serialize commits");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_pull_request_commits("owner", "repo", 42).await;

        assert!(result.is_ok());
        let commits = result.expect("get commits");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].commit.message, "Fix bug");
    }

    #[gpui::test]
    async fn test_merge_pull_request() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::PUT);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/merge"));

            let merge_result = MergeResult {
                sha: "merged_sha_123".to_string(),
                merged: true,
                message: "Pull Request successfully merged".to_string(),
            };

            let body = serde_json::to_vec(&merge_result).expect("serialize merge_result");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = MergePRParams {
            commit_title: Some("Merge PR #42".to_string()),
            commit_message: None,
            sha: None,
            merge_method: Some(MergeMethod::Squash),
        };

        let result = client.merge_pull_request("owner", "repo", 42, params).await;

        assert!(result.is_ok());
        let merge = result.expect("get merge result");
        assert!(merge.merged);
    }

    #[gpui::test]
    async fn test_request_reviewers() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/requested_reviewers"));

            let pr = make_test_pr(42, "PR with reviewers");
            let body = serde_json::to_vec(&pr).expect("serialize pr");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = RequestReviewersParams {
            reviewers: vec!["reviewer1".to_string()],
            team_reviewers: vec![],
        };

        let result = client.request_reviewers("owner", "repo", 42, params).await;

        assert!(result.is_ok());
    }

    #[gpui::test]
    async fn test_get_requested_reviewers() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/requested_reviewers"));

            let review_request = ReviewRequest {
                users: vec![User {
                    id: 456,
                    login: "reviewer1".to_string(),
                    avatar_url: "https://example.com/avatar.png".to_string(),
                }],
                teams: vec![Team {
                    id: 789,
                    name: "Core Team".to_string(),
                    slug: "core-team".to_string(),
                }],
            };

            let body = serde_json::to_vec(&review_request).expect("serialize review_request");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_requested_reviewers("owner", "repo", 42).await;

        assert!(result.is_ok());
        let reviewers = result.expect("get reviewers");
        assert_eq!(reviewers.users.len(), 1);
        assert_eq!(reviewers.teams.len(), 1);
    }

    #[gpui::test]
    async fn test_get_check_runs() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/commits/abc123/check-runs"));

            let check_run_list = CheckRunList {
                total_count: 2,
                check_runs: vec![
                    CheckRun {
                        id: 1,
                        name: "CI Build".to_string(),
                        status: CheckStatus::Completed,
                        conclusion: Some(CheckConclusion::Success),
                        url: Some("https://github.com/owner/repo/runs/1".to_string()),
                    },
                    CheckRun {
                        id: 2,
                        name: "Tests".to_string(),
                        status: CheckStatus::InProgress,
                        conclusion: None,
                        url: Some("https://github.com/owner/repo/runs/2".to_string()),
                    },
                ],
            };

            let body = serde_json::to_vec(&check_run_list).expect("serialize check_run_list");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_check_runs("owner", "repo", "abc123").await;

        assert!(result.is_ok());
        let checks = result.expect("get checks");
        assert_eq!(checks.total_count, 2);
        assert_eq!(checks.check_runs.len(), 2);
    }
}
