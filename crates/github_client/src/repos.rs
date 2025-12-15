use crate::{Branch, GitHubClient, Repository};
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
}
