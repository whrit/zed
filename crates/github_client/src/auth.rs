use crate::{DeviceCodeResponse, DeviceTokenResponse, GitHubClient};
use anyhow::{Context as _, Result};

const GITHUB_OAUTH_URL: &str = "https://github.com/login";

impl GitHubClient {
    pub async fn request_device_code(&self, client_id: &str) -> Result<DeviceCodeResponse> {
        let url = format!("{}/device/code", GITHUB_OAUTH_URL);
        let body = format!("client_id={}&scope=repo", client_id);

        let request = http::Request::post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.into_bytes().into())?;

        self.send_request(request).await
    }

    pub async fn poll_device_token(
        &self,
        client_id: &str,
        device_code: &str,
    ) -> Result<DeviceTokenResponse> {
        let url = format!("{}/oauth/access_token", GITHUB_OAUTH_URL);
        let body = format!(
            "client_id={}&device_code={}&grant_type=urn:ietf:params:oauth:grant-type:device_code",
            client_id, device_code
        );

        let request = http::Request::post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.into_bytes().into())?;

        self.send_request(request).await
    }

    pub fn device_code_info(response: &DeviceCodeResponse) -> DeviceCodeInfo {
        DeviceCodeInfo {
            user_code: response.user_code.clone(),
            verification_uri: response.verification_uri.clone(),
            expires_in_seconds: response.expires_in,
            poll_interval_seconds: response.interval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in_seconds: u64,
    pub poll_interval_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_client::FakeHttpClient;

    #[gpui::test]
    async fn test_request_device_code() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request.uri().to_string().contains("/device/code"));

            let response = DeviceCodeResponse {
                device_code: "device123".to_string(),
                user_code: "USER-CODE".to_string(),
                verification_uri: "https://github.com/login/device".to_string(),
                expires_in: 900,
                interval: 5,
            };

            let body = serde_json::to_vec(&response).expect("serialize response");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::new(http_client);
        let result = client.request_device_code("test_client_id").await;

        assert!(result.is_ok());
        let response = result.expect("get response");
        assert_eq!(response.device_code, "device123");
        assert_eq!(response.user_code, "USER-CODE");
    }

    #[gpui::test]
    async fn test_poll_device_token_pending() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);
            assert!(request.uri().to_string().contains("/oauth/access_token"));

            let response = DeviceTokenResponse::Pending {
                error: "authorization_pending".to_string(),
            };

            let body = serde_json::to_vec(&response).expect("serialize response");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::new(http_client);
        let result = client
            .poll_device_token("test_client_id", "device123")
            .await;

        assert!(result.is_ok());
        match result.expect("get response") {
            DeviceTokenResponse::Pending { error } => {
                assert_eq!(error, "authorization_pending");
            }
            _ => panic!("expected pending response"),
        }
    }

    #[gpui::test]
    async fn test_poll_device_token_success() {
        let http_client = FakeHttpClient::create(|request| async move {
            assert_eq!(request.method(), http::Method::POST);

            let response = DeviceTokenResponse::Success {
                access_token: "gho_test_token_123".to_string(),
            };

            let body = serde_json::to_vec(&response).expect("serialize response");
            Ok(http::Response::builder()
                .status(200)
                .body(body.into())
                .expect("build response"))
        });

        let client = GitHubClient::new(http_client);
        let result = client
            .poll_device_token("test_client_id", "device123")
            .await;

        assert!(result.is_ok());
        match result.expect("get response") {
            DeviceTokenResponse::Success { access_token } => {
                assert_eq!(access_token, "gho_test_token_123");
            }
            _ => panic!("expected success response"),
        }
    }

    #[gpui::test]
    async fn test_device_code_info() {
        let response = DeviceCodeResponse {
            device_code: "device123".to_string(),
            user_code: "ABCD-1234".to_string(),
            verification_uri: "https://github.com/login/device".to_string(),
            expires_in: 900,
            interval: 5,
        };

        let info = GitHubClient::device_code_info(&response);
        assert_eq!(info.user_code, "ABCD-1234");
        assert_eq!(info.verification_uri, "https://github.com/login/device");
        assert_eq!(info.expires_in_seconds, 900);
        assert_eq!(info.poll_interval_seconds, 5);
    }
}
