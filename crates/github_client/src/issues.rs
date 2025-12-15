use crate::{Comment, CreateCommentParams, CreateIssueParams, GitHubClient, Issue, IssueState};
use anyhow::Result;

impl GitHubClient {
    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: IssueState,
    ) -> Result<Vec<Issue>> {
        let path = format!(
            "/repos/{}/{}/issues?state={}",
            owner,
            repo,
            state.as_str()
        );
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn get_issue(&self, owner: &str, repo: &str, number: u32) -> Result<Issue> {
        let path = format!("/repos/{}/{}/issues/{}", owner, repo, number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        params: CreateIssueParams,
    ) -> Result<Issue> {
        let path = format!("/repos/{}/{}/issues", owner, repo);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn add_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u32,
        params: CreateCommentParams,
    ) -> Result<Comment> {
        let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
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
    use crate::User;
    use chrono::Utc;
    use http_client::FakeHttpClient;

    fn make_test_issue(number: u32, title: &str) -> Issue {
        Issue {
            number,
            title: title.to_string(),
            state: IssueState::Open,
            author: User {
                id: 123,
                login: "testuser".to_string(),
                avatar_url: "https://example.com/avatar.png".to_string(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            url: format!("https://github.com/owner/repo/issues/{}", number),
            body: Some("Issue body".to_string()),
        }
    }

    #[gpui::test]
    async fn test_list_issues() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/issues"));

            let issues = vec![make_test_issue(1, "Test Issue")];
            let body = serde_json::to_vec(&issues).expect("serialize issues");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_issues("owner", "repo", IssueState::Open).await;

        assert!(result.is_ok());
        let issues = result.expect("get issues");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].number, 1);
    }

    #[gpui::test]
    async fn test_get_issue() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/issues/42"));

            let issue = make_test_issue(42, "Specific Issue");
            let body = serde_json::to_vec(&issue).expect("serialize issue");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_issue("owner", "repo", 42).await;

        assert!(result.is_ok());
        let issue = result.expect("get issue");
        assert_eq!(issue.number, 42);
    }

    #[gpui::test]
    async fn test_create_issue() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request.uri().path().contains("/repos/owner/repo/issues"));

            let issue = make_test_issue(99, "New Issue");
            let body = serde_json::to_vec(&issue).expect("serialize issue");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreateIssueParams {
            title: "New Issue".to_string(),
            body: Some("New issue body".to_string()),
        };

        let result = client.create_issue("owner", "repo", params).await;

        assert!(result.is_ok());
        let issue = result.expect("get issue");
        assert_eq!(issue.number, 99);
    }

    #[gpui::test]
    async fn test_add_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/issues/42/comments"));

            let comment = Comment {
                id: 1001,
                author: User {
                    id: 789,
                    login: "commenter".to_string(),
                    avatar_url: "https://example.com/avatar4.png".to_string(),
                },
                body: "Test comment".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let body = serde_json::to_vec(&comment).expect("serialize comment");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreateCommentParams {
            body: "Test comment".to_string(),
        };

        let result = client.add_comment("owner", "repo", 42, params).await;

        assert!(result.is_ok());
        let comment = result.expect("get comment");
        assert_eq!(comment.id, 1001);
        assert_eq!(comment.body, "Test comment");
    }
}
