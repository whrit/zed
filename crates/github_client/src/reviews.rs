use crate::{
    CreatePullRequestReviewCommentParams, CreateReviewCommentReplyParams, CreateReviewParams,
    GitHubClient, PullRequestReviewComment, Review, UpdateReviewCommentParams,
};
use anyhow::Result;

impl GitHubClient {
    pub async fn list_reviews(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u32,
    ) -> Result<Vec<Review>> {
        let path = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u32,
        params: CreateReviewParams,
    ) -> Result<Review> {
        let path = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn list_review_comments(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u32,
    ) -> Result<Vec<PullRequestReviewComment>> {
        let path = format!("/repos/{}/{}/pulls/{}/comments", owner, repo, pull_number);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn get_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<PullRequestReviewComment> {
        let path = format!("/repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn create_review_comment(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u32,
        params: CreatePullRequestReviewCommentParams,
    ) -> Result<PullRequestReviewComment> {
        let path = format!("/repos/{}/{}/pulls/{}/comments", owner, repo, pull_number);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn reply_to_review_comment(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u32,
        comment_id: u64,
        params: CreateReviewCommentReplyParams,
    ) -> Result<PullRequestReviewComment> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/comments/{}/replies",
            owner, repo, pull_number, comment_id
        );
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::POST, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn update_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        params: UpdateReviewCommentParams,
    ) -> Result<PullRequestReviewComment> {
        let path = format!("/repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let body_json = serde_json::to_vec(&params)?;
        let request = self
            .build_request(http::Method::PATCH, &path)
            .header("Content-Type", "application/json")
            .body(body_json.into())?;

        self.send_request(request).await
    }

    pub async fn delete_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<()> {
        let path = format!("/repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let request = self
            .build_request(http::Method::DELETE, &path)
            .body(Default::default())?;

        self.send_request_no_content(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiffSide, ReviewEvent, ReviewState, User};
    use chrono::Utc;
    use http_client::FakeHttpClient;

    #[gpui::test]
    async fn test_list_reviews() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/reviews"));

            let reviews = vec![Review {
                id: 1001,
                author: User {
                    id: 123,
                    login: "reviewer".to_string(),
                    avatar_url: "https://example.com/avatar.png".to_string(),
                },
                body: Some("Looks good!".to_string()),
                state: ReviewState::Approved,
                submitted_at: Some(Utc::now()),
            }];

            let body = serde_json::to_vec(&reviews).expect("serialize reviews");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_reviews("owner", "repo", 42).await;

        assert!(result.is_ok());
        let reviews = result.expect("get reviews");
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].state, ReviewState::Approved);
    }

    #[gpui::test]
    async fn test_create_review() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/reviews"));

            let review = Review {
                id: 2002,
                author: User {
                    id: 456,
                    login: "newreviewer".to_string(),
                    avatar_url: "https://example.com/avatar2.png".to_string(),
                },
                body: Some("Needs changes".to_string()),
                state: ReviewState::ChangesRequested,
                submitted_at: Some(Utc::now()),
            };

            let body = serde_json::to_vec(&review).expect("serialize review");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreateReviewParams {
            body: Some("Needs changes".to_string()),
            event: ReviewEvent::RequestChanges,
            comments: vec![],
        };

        let result = client.create_review("owner", "repo", 42, params).await;

        assert!(result.is_ok());
        let review = result.expect("get review");
        assert_eq!(review.state, ReviewState::ChangesRequested);
    }

    #[gpui::test]
    async fn test_list_review_comments() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/comments"));

            let comments = vec![PullRequestReviewComment {
                id: 5001,
                author: User {
                    id: 789,
                    login: "commenter".to_string(),
                    avatar_url: "https://example.com/avatar3.png".to_string(),
                },
                body: "Great stuff!".to_string(),
                path: "src/main.rs".to_string(),
                line: Some(10),
                original_line: Some(10),
                start_line: None,
                start_side: None,
                side: Some(DiffSide::Right),
                diff_hunk: Some("@@ -8,7 +8,7 @@".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                in_reply_to_id: None,
                url: "https://github.com/owner/repo/pull/42#discussion-r5001".to_string(),
            }];

            let body = serde_json::to_vec(&comments).expect("serialize comments");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_review_comments("owner", "repo", 42).await;

        assert!(result.is_ok());
        let comments = result.expect("get comments");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].body, "Great stuff!");
        assert_eq!(comments[0].path, "src/main.rs");
    }

    #[gpui::test]
    async fn test_get_review_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/comments/5001"));

            let comment = PullRequestReviewComment {
                id: 5001,
                author: User {
                    id: 789,
                    login: "commenter".to_string(),
                    avatar_url: "https://example.com/avatar3.png".to_string(),
                },
                body: "Great stuff!".to_string(),
                path: "src/main.rs".to_string(),
                line: Some(10),
                original_line: Some(10),
                start_line: None,
                start_side: None,
                side: Some(DiffSide::Right),
                diff_hunk: Some("@@ -8,7 +8,7 @@".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                in_reply_to_id: None,
                url: "https://github.com/owner/repo/pull/42#discussion-r5001".to_string(),
            };

            let body = serde_json::to_vec(&comment).expect("serialize comment");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_review_comment("owner", "repo", 5001).await;

        assert!(result.is_ok());
        let comment = result.expect("get comment");
        assert_eq!(comment.id, 5001);
        assert_eq!(comment.body, "Great stuff!");
    }

    #[gpui::test]
    async fn test_create_review_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/comments"));

            let comment = PullRequestReviewComment {
                id: 5002,
                author: User {
                    id: 999,
                    login: "newcommenter".to_string(),
                    avatar_url: "https://example.com/avatar4.png".to_string(),
                },
                body: "This needs work".to_string(),
                path: "src/lib.rs".to_string(),
                line: Some(20),
                original_line: Some(20),
                start_line: Some(15),
                start_side: Some(DiffSide::Right),
                side: Some(DiffSide::Right),
                diff_hunk: Some("@@ -13,10 +13,10 @@".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                in_reply_to_id: None,
                url: "https://github.com/owner/repo/pull/42#discussion-r5002".to_string(),
            };

            let body = serde_json::to_vec(&comment).expect("serialize comment");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreatePullRequestReviewCommentParams {
            body: "This needs work".to_string(),
            path: "src/lib.rs".to_string(),
            line: Some(20),
            side: Some(DiffSide::Right),
            start_line: Some(15),
            start_side: Some(DiffSide::Right),
            commit_id: "abc123".to_string(),
        };

        let result = client.create_review_comment("owner", "repo", 42, params).await;

        assert!(result.is_ok());
        let comment = result.expect("get comment");
        assert_eq!(comment.id, 5002);
        assert_eq!(comment.body, "This needs work");
    }

    #[gpui::test]
    async fn test_reply_to_review_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/42/comments/5001/replies"));

            let comment = PullRequestReviewComment {
                id: 5003,
                author: User {
                    id: 888,
                    login: "replier".to_string(),
                    avatar_url: "https://example.com/avatar5.png".to_string(),
                },
                body: "I agree with this comment".to_string(),
                path: "src/main.rs".to_string(),
                line: Some(10),
                original_line: Some(10),
                start_line: None,
                start_side: None,
                side: Some(DiffSide::Right),
                diff_hunk: Some("@@ -8,7 +8,7 @@".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                in_reply_to_id: Some(5001),
                url: "https://github.com/owner/repo/pull/42#discussion-r5003".to_string(),
            };

            let body = serde_json::to_vec(&comment).expect("serialize comment");
            Ok(http::Response::builder()
                .status(201)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = CreateReviewCommentReplyParams {
            body: "I agree with this comment".to_string(),
        };

        let result = client
            .reply_to_review_comment("owner", "repo", 42, 5001, params)
            .await;

        assert!(result.is_ok());
        let comment = result.expect("get comment");
        assert_eq!(comment.in_reply_to_id, Some(5001));
        assert_eq!(comment.body, "I agree with this comment");
    }

    #[gpui::test]
    async fn test_update_review_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::PATCH);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/comments/5001"));

            let comment = PullRequestReviewComment {
                id: 5001,
                author: User {
                    id: 789,
                    login: "commenter".to_string(),
                    avatar_url: "https://example.com/avatar3.png".to_string(),
                },
                body: "Updated comment text".to_string(),
                path: "src/main.rs".to_string(),
                line: Some(10),
                original_line: Some(10),
                start_line: None,
                start_side: None,
                side: Some(DiffSide::Right),
                diff_hunk: Some("@@ -8,7 +8,7 @@".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                in_reply_to_id: None,
                url: "https://github.com/owner/repo/pull/42#discussion-r5001".to_string(),
            };

            let body = serde_json::to_vec(&comment).expect("serialize comment");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let params = UpdateReviewCommentParams {
            body: "Updated comment text".to_string(),
        };

        let result = client
            .update_review_comment("owner", "repo", 5001, params)
            .await;

        assert!(result.is_ok());
        let comment = result.expect("get comment");
        assert_eq!(comment.body, "Updated comment text");
    }

    #[gpui::test]
    async fn test_delete_review_comment() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::DELETE);
            assert!(request
                .uri()
                .path()
                .contains("/repos/owner/repo/pulls/comments/5001"));

            Ok(http::Response::builder()
                .status(204)
                .body(Vec::new().into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.delete_review_comment("owner", "repo", 5001).await;

        assert!(result.is_ok());
    }
}
