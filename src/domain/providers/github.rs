use crate::domain::provider::Provider;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

/// GitHub Actions Secrets Provider
pub struct GitHubProvider {
    repo: String,
    token: String,
    client: Client,
}

#[derive(Serialize)]
struct SecretPayload {
    encrypted_value: String,
    key_id: String,
}

#[derive(Deserialize)]
struct PublicKey {
    key_id: String,
    key: String,
}

#[derive(Deserialize)]
struct SecretsList {
    secrets: Vec<SecretInfo>,
}

#[derive(Deserialize)]
struct SecretInfo {
    name: String,
}

impl GitHubProvider {
    pub fn new(repo: String, token: String) -> Self {
        Self {
            repo,
            token,
            client: Client::new(),
        }
    }

    fn with_github_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "holecard-cli")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    fn check_response(response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        Err(anyhow!(
            "GitHub API error: {} - {}",
            response.status(),
            response.text().unwrap_or_default()
        ))
    }

    fn get_public_key(&self) -> Result<PublicKey> {
        let url = format!(
            "https://api.github.com/repos/{}/actions/secrets/public-key",
            self.repo
        );

        let response = self
            .with_github_headers(self.client.get(&url))
            .send()
            .context("Failed to fetch GitHub public key")?;

        Self::check_response(response)?
            .json::<PublicKey>()
            .context("Failed to parse public key response")
    }

    fn encrypt_secret(&self, value: &str, public_key: &str) -> Result<String> {
        sodiumoxide::init().map_err(|_| anyhow!("Failed to initialize sodiumoxide"))?;

        let pk_bytes = BASE64
            .decode(public_key)
            .context("Failed to decode public key")?;

        if pk_bytes.len() != sodiumoxide::crypto::box_::PUBLICKEYBYTES {
            return Err(anyhow!("Invalid public key length"));
        }

        let mut pk_array = [0u8; sodiumoxide::crypto::box_::PUBLICKEYBYTES];
        pk_array.copy_from_slice(&pk_bytes);
        let public_key = sodiumoxide::crypto::box_::PublicKey(pk_array);

        let sealed = sodiumoxide::crypto::sealedbox::seal(value.as_bytes(), &public_key);
        Ok(BASE64.encode(sealed))
    }
}

impl Provider for GitHubProvider {
    fn push_secret(&self, key: &str, value: &str) -> Result<()> {
        let public_key = self.get_public_key()?;
        let encrypted_value = self.encrypt_secret(value, &public_key.key)?;

        let url = format!(
            "https://api.github.com/repos/{}/actions/secrets/{}",
            self.repo, key
        );

        let payload = SecretPayload {
            encrypted_value,
            key_id: public_key.key_id,
        };

        let response = self
            .with_github_headers(self.client.put(&url))
            .json(&payload)
            .send()
            .context("Failed to push secret to GitHub")?;

        Self::check_response(response)?;
        Ok(())
    }

    fn list_secrets(&self) -> Result<Vec<String>> {
        let url = format!("https://api.github.com/repos/{}/actions/secrets", self.repo);

        let response = self
            .with_github_headers(self.client.get(&url))
            .send()
            .context("Failed to list secrets from GitHub")?;

        let secrets_list: SecretsList = Self::check_response(response)?
            .json()
            .context("Failed to parse secrets list response")?;

        Ok(secrets_list.secrets.into_iter().map(|s| s.name).collect())
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        let url = format!(
            "https://api.github.com/repos/{}/actions/secrets/{}",
            self.repo, key
        );

        let response = self
            .with_github_headers(self.client.delete(&url))
            .send()
            .context("Failed to delete secret from GitHub")?;

        Self::check_response(response)?;
        Ok(())
    }
}
