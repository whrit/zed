use crate::{Branch, GitHubClient, Label, Repository, User};
use anyhow::Result;

impl GitHubClient {
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository> {
        let path = format!("/repos/{}/{}", owner, repo);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        let path = format!("/repos/{}/{}/branches", owner, repo);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }

    pub async fn search_users(&self, owner: &str, repo: &str, query: &str) -> Result<Vec<User>> {
        let path = format!("/repos/{}/{}/collaborators", owner, repo);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        let all_users: Vec<User> = self.send_request(request).await?;

        let query_lower = query.to_lowercase();
        Ok(all_users
            .into_iter()
            .filter(|user| user.login.to_lowercase().contains(&query_lower))
            .collect())
    }

    pub async fn list_labels(&self, owner: &str, repo: &str) -> Result<Vec<Label>> {
        let path = format!("/repos/{}/{}/labels", owner, repo);
        let request = self
            .build_request(http::Method::GET, &path)
            .body(Default::default())?;

        self.send_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommitRef;
    use http_client::FakeHttpClient;

    #[gpui::test]
    async fn test_get_repository() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo"));

            let repo = Repository {
                id: 12345,
                name: "repo".to_string(),
                full_name: "owner/repo".to_string(),
                url: "https://github.com/owner/repo".to_string(),
                description: Some("Test repository".to_string()),
                private: false,
                default_branch: "main".to_string(),
            };

            let body = serde_json::to_vec(&repo).expect("serialize repo");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.get_repository("owner", "repo").await;

        assert!(result.is_ok());
        let repo = result.expect("get repo");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.full_name, "owner/repo");
    }

    #[gpui::test]
    async fn test_list_branches() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/branches"));

            let branches = vec![
                Branch {
                    name: "main".to_string(),
                    commit: CommitRef {
                        sha: "abc123".to_string(),
                    },
                    protected: true,
                },
                Branch {
                    name: "develop".to_string(),
                    commit: CommitRef {
                        sha: "def456".to_string(),
                    },
                    protected: false,
                },
            ];

            let body = serde_json::to_vec(&branches).expect("serialize branches");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_branches("owner", "repo").await;

        assert!(result.is_ok());
        let branches = result.expect("get branches");
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "main");
        assert_eq!(branches[1].name, "develop");
    }

    #[gpui::test]
    async fn test_search_users() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/collaborators"));

            let users = vec![
                User {
                    id: 1,
                    login: "alice".to_string(),
                    avatar_url: "https://example.com/alice.png".to_string(),
                },
                User {
                    id: 2,
                    login: "bob".to_string(),
                    avatar_url: "https://example.com/bob.png".to_string(),
                },
                User {
                    id: 3,
                    login: "charlie".to_string(),
                    avatar_url: "https://example.com/charlie.png".to_string(),
                },
            ];

            let body = serde_json::to_vec(&users).expect("serialize users");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.search_users("owner", "repo", "ali").await;

        assert!(result.is_ok());
        let users = result.expect("get users");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].login, "alice");
    }

    #[gpui::test]
    async fn test_list_labels() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::GET);
            assert!(request.uri().path().contains("/repos/owner/repo/labels"));

            let labels = vec![
                Label {
                    id: 1,
                    name: "bug".to_string(),
                    color: "d73a4a".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
                Label {
                    id: 2,
                    name: "enhancement".to_string(),
                    color: "a2eeef".to_string(),
                    description: Some("New feature or request".to_string()),
                },
            ];

            let body = serde_json::to_vec(&labels).expect("serialize labels");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::with_token(http_client, "test_token".to_string());
        let result = client.list_labels("owner", "repo").await;

        assert!(result.is_ok());
        let labels = result.expect("get labels");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[1].name, "enhancement");
    }
}
