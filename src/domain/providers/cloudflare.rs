use crate::domain::provider::Provider;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// Cloudflare Workers Secrets Provider
pub struct CloudflareProvider {
    account_id: String,
    worker_name: String,
    token: String,
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
        }
    }
}

impl Provider for CloudflareProvider {
    fn push_secret(&self, key: &str, value: &str) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/secrets",
            self.account_id, self.worker_name
        );

        let payload = SecretPayload {
            name: key.to_string(),
            text: value.to_string(),
            secret_type: "secret_text".to_string(),
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .context("Failed to push secret to Cloudflare")?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!(
                "Cloudflare API error: {}",
                error_text
            ));
        }

        let api_response: ApiResponse = response
            .json()
            .context("Failed to parse Cloudflare API response")?;

        if !api_response.success {
            let errors: Vec<String> = api_response
                .errors
                .into_iter()
                .map(|e| e.message)
                .collect();
            return Err(anyhow!("Cloudflare API errors: {}", errors.join(", ")));
        }

        Ok(())
    }

    fn list_secrets(&self) -> Result<Vec<String>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/secrets",
            self.account_id, self.worker_name
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
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
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/secrets/{}",
            self.account_id, self.worker_name, key
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
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

        if !api_response.success {
            let errors: Vec<String> = api_response
                .errors
                .into_iter()
                .map(|e| e.message)
                .collect();
            return Err(anyhow!("Cloudflare API errors: {}", errors.join(", ")));
        }

        Ok(())
    }
}
