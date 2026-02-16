pub mod api;
pub mod auth;
pub mod types;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::config::Config;
use crate::credentials::{self, StoredTokens};
use crate::error::ShottyError;

pub struct DropboxBackend {
    app_key: String,
    upload_path_template: String,
    client: Client,
}

impl DropboxBackend {
    pub fn new(config: &Config) -> Result<Self> {
        let app_key = config
            .dropbox
            .app_key
            .as_deref()
            .ok_or(ShottyError::MissingAppKey)?
            .to_string();

        Ok(Self {
            app_key,
            upload_path_template: config.dropbox_upload_path().to_string(),
            client: Client::new(),
        })
    }

    /// Get a valid access token, refreshing if expired.
    async fn access_token(&self) -> Result<String> {
        let mut tokens =
            credentials::load_tokens("dropbox")?.ok_or(ShottyError::NotAuthenticated)?;

        if tokens.is_expired() {
            let refreshed =
                api::refresh_access_token(&self.client, &self.app_key, &tokens.refresh_token)
                    .await?;

            tokens = StoredTokens {
                access_token: refreshed.access_token,
                refresh_token: refreshed.refresh_token.unwrap_or(tokens.refresh_token),
                expires_at: chrono::Utc::now().timestamp() + refreshed.expires_in,
            };

            credentials::save_tokens("dropbox", &tokens)?;
        }

        Ok(tokens.access_token)
    }

    fn resolve_upload_path(&self, filename: &str) -> String {
        let now = chrono::Local::now();
        let path = self
            .upload_path_template
            .replace("{YYYY-MM}", &now.format("%Y-%m").to_string());
        format!("{path}/{filename}")
    }
}

impl DropboxBackend {
    pub async fn authenticate(&self) -> Result<()> {
        auth::run_pkce_flow(&self.app_key).await
    }

    pub async fn upload(&self, filename: &str, data: &[u8]) -> Result<String> {
        let token = self.access_token().await?;
        let path = self.resolve_upload_path(filename);

        let result = api::upload_file(&self.client, &token, &path, data)
            .await
            .context("Dropbox upload failed")?;

        let uploaded_path = result.path_display.unwrap_or(path);

        let url = api::create_shared_link(&self.client, &token, &uploaded_path)
            .await
            .context("Failed to create shared link")?;

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_upload_path_format() {
        let backend = DropboxBackend {
            app_key: "test".into(),
            upload_path_template: "/Shotty/{YYYY-MM}".into(),
            client: Client::new(),
        };
        let path = backend.resolve_upload_path("screenshot.png");
        assert!(path.starts_with("/Shotty/"));
        assert!(path.ends_with("/screenshot.png"));
    }
}
