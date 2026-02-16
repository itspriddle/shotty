pub mod api;
pub mod types;

use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;

use crate::credentials;
use crate::error::ShottyError;

pub struct DroplrBackend {
    client: Client,
}

impl DroplrBackend {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl DroplrBackend {
    pub async fn authenticate(&self) -> Result<()> {
        println!("Enter your Droplr credentials.");

        print!("Email: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut email = String::new();
        std::io::stdin()
            .read_line(&mut email)
            .context("Failed to read email")?;
        let email = email.trim();

        if email.is_empty() {
            anyhow::bail!("Email cannot be empty");
        }

        let password =
            rpassword::prompt_password("Password: ").context("Failed to read password")?;

        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        let token = BASE64.encode(format!("{email}:{password}"));
        credentials::save_auth_token("droplr", &token)?;

        println!("Droplr credentials saved.");
        Ok(())
    }

    pub async fn upload(&self, filename: &str, data: &[u8]) -> Result<String> {
        let token = credentials::load_auth_token("droplr")?.ok_or(ShottyError::NotAuthenticated)?;

        api::upload_file(&self.client, &token, filename, data)
            .await
            .context("Droplr upload failed")
    }
}
