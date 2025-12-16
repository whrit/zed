mod auth;
mod issues;
mod pulls;
mod repos;
mod reviews;
mod types;

pub use auth::*;
pub use types::*;

use anyhow::{Context as _, Result, bail};
use futures::AsyncReadExt;
use http::Request;
use http_client::{AsyncBody, HttpClient, HttpRequestExt, RedirectPolicy};
use std::sync::Arc;

const GITHUB_API_URL: &str = "https://api.github.com";

pub struct GitHubClient {
    http_client: Arc<dyn HttpClient>,
    token: Option<String>,
    base_url: String,
}

impl GitHubClient {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            http_client,
            token: None,
            base_url: GITHUB_API_URL.to_string(),
        }
    }

    pub fn with_token(http_client: Arc<dyn HttpClient>, token: String) -> Self {
        Self {
            http_client,
            token: Some(token),
            base_url: GITHUB_API_URL.to_string(),
        }
    }

    pub fn with_base_url(http_client: Arc<dyn HttpClient>, base_url: String) -> Self {
        Self {
            http_client,
            token: None,
            base_url,
        }
    }

    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn http_client(&self) -> Arc<dyn HttpClient> {
        Arc::clone(&self.http_client)
    }

    pub(crate) async fn send_request<T>(&self, request: Request<AsyncBody>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut response = self
            .http_client
            .send(request)
            .await
            .context("error sending request")?;

        let mut body = Vec::new();
        response
            .body_mut()
            .read_to_end(&mut body)
            .await
            .context("error reading response body")?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!(
                "HTTP error {}: {}",
                response.status().as_u16(),
                text
            );
        }

        serde_json::from_slice(&body).map_err(|err| {
            log::error!("Error deserializing response: {err:?}");
            log::error!("Response body: {:?}", String::from_utf8_lossy(&body));
            anyhow::anyhow!("error deserializing response: {err:?}")
        })
    }

    pub(crate) async fn send_request_no_content(&self, request: Request<AsyncBody>) -> Result<()> {
        let mut response = self
            .http_client
            .send(request)
            .await
            .context("error sending request")?;

        let mut body = Vec::new();
        response
            .body_mut()
            .read_to_end(&mut body)
            .await
            .context("error reading response body")?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!(
                "HTTP error {}: {}",
                response.status().as_u16(),
                text
            );
        }

        Ok(())
    }

    pub(crate) fn build_request(&self, method: http::Method, path: &str) -> http::request::Builder {
        let url = format!("{}{}", self.base_url, path);
        let mut builder = Request::builder()
            .method(method)
            .uri(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .follow_redirects(RedirectPolicy::FollowAll);

        if let Some(token) = &self.token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_client::FakeHttpClient;

    #[gpui::test]
    async fn test_new_client() {
        let http_client = FakeHttpClient::with_200_response();
        let client = GitHubClient::new(http_client);
        assert!(client.token.is_none());
        assert_eq!(client.base_url, GITHUB_API_URL);
    }

    #[gpui::test]
    async fn test_client_with_token() {
        let http_client = FakeHttpClient::with_200_response();
        let token = "test_token".to_string();
        let client = GitHubClient::with_token(http_client, token.clone());
        assert_eq!(client.token, Some(token));
    }

    #[gpui::test]
    async fn test_client_with_base_url() {
        let http_client = FakeHttpClient::with_200_response();
        let base_url = "https://api.github.example.com".to_string();
        let client = GitHubClient::with_base_url(http_client, base_url.clone());
        assert_eq!(client.base_url, base_url);
    }

    #[gpui::test]
    async fn test_set_token() {
        let http_client = FakeHttpClient::with_200_response();
        let mut client = GitHubClient::new(http_client);
        assert!(client.token.is_none());

        let token = "new_token".to_string();
        client.set_token(Some(token.clone()));
        assert_eq!(client.token, Some(token));

        client.set_token(None);
        assert!(client.token.is_none());
    }

    #[gpui::test]
    async fn test_token_accessor() {
        let http_client = FakeHttpClient::with_200_response();
        let client = GitHubClient::with_token(http_client, "my_token".to_string());
        assert_eq!(client.token(), Some("my_token"));
    }

    #[gpui::test]
    async fn test_base_url_accessor() {
        let http_client = FakeHttpClient::with_200_response();
        let client = GitHubClient::new(http_client);
        assert_eq!(client.base_url(), GITHUB_API_URL);
    }
}
