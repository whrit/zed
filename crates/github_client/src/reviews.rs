use crate::{CreateReviewParams, GitHubClient, Review};
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReviewEvent, ReviewState, User};
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
}
