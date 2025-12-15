use crate::{CreatePRParams, GitHubClient, PRState, PullRequest, PullRequestDetail};
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RefData, User};
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
}
