use crate::domain::provider::Provider;
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

/// Cloudflare Workers Secrets Provider
pub struct CloudflareProvider {
    account_id: String,
    worker_name: String,
    token: String,
    client: Client,
}

#[derive(Serialize)]
struct SecretPayload {
    name: String,
    text: String,
    #[serde(rename = "type")]
    secret_type: String,
}

#[derive(Deserialize)]
struct SecretsListResponse {
    result: Vec<SecretInfo>,
    success: bool,
}

#[derive(Deserialize)]
struct SecretInfo {
    name: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    success: bool,
    errors: Vec<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

impl CloudflareProvider {
    pub fn new(account_id: String, worker_name: String, token: String) -> Self {
        Self {
            account_id,
            worker_name,
            token,
            client: Client::new(),
        }
    }

    fn with_cloudflare_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
    }

    fn secrets_url(&self) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/secrets",
            self.account_id, self.worker_name
        )
    }

    fn check_api_response(response: ApiResponse) -> Result<()> {
        if response.success {
            return Ok(());
        }
        let errors: Vec<String> = response.errors.into_iter().map(|e| e.message).collect();
        Err(anyhow!("Cloudflare API errors: {}", errors.join(", ")))
    }
}

impl Provider for CloudflareProvider {
    fn push_secret(&self, key: &str, value: &str) -> Result<()> {
        let payload = SecretPayload {
            name: key.to_string(),
            text: value.to_string(),
            secret_type: "secret_text".to_string(),
        };

        let response = self
            .with_cloudflare_headers(self.client.put(self.secrets_url()))
            .json(&payload)
            .send()
            .context("Failed to push secret to Cloudflare")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Cloudflare API error: {}",
                response.text().unwrap_or_default()
            ));
        }

        let api_response: ApiResponse = response
            .json()
            .context("Failed to parse Cloudflare API response")?;

        Self::check_api_response(api_response)
    }

    fn list_secrets(&self) -> Result<Vec<String>> {
        let response = self
            .with_cloudflare_headers(self.client.get(self.secrets_url()))
            .send()
            .context("Failed to list secrets from Cloudflare")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Cloudflare API error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            ));
        }

        let secrets_list: SecretsListResponse = response
            .json()
            .context("Failed to parse secrets list response")?;

        if !secrets_list.success {
            return Err(anyhow!("Failed to list secrets"));
        }

        Ok(secrets_list.result.into_iter().map(|s| s.name).collect())
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        let url = format!("{}/{}", self.secrets_url(), key);

        let response = self
            .with_cloudflare_headers(self.client.delete(&url))
            .send()
            .context("Failed to delete secret from Cloudflare")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Cloudflare API error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            ));
        }

        let api_response: ApiResponse = response
            .json()
            .context("Failed to parse Cloudflare API response")?;

        Self::check_api_response(api_response)
    }
}
