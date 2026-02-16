use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "shotty";

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

impl StoredTokens {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at
    }
}

pub fn load_tokens(backend: &str) -> Result<Option<StoredTokens>> {
    let entry = keyring::Entry::new(SERVICE_NAME, backend).context("Failed to access keyring")?;

    match entry.get_password() {
        Ok(json) => {
            let tokens: StoredTokens =
                serde_json::from_str(&json).context("Failed to parse stored tokens")?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Keyring error: {e}")),
    }
}

pub fn save_tokens(backend: &str, tokens: &StoredTokens) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, backend).context("Failed to access keyring")?;
    let json = serde_json::to_string(tokens).context("Failed to serialize tokens")?;
    entry
        .set_password(&json)
        .context("Failed to save tokens to keyring")?;
    Ok(())
}

#[allow(dead_code)]
pub fn delete_tokens(backend: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, backend).context("Failed to access keyring")?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Keyring error: {e}")),
    }
}

pub fn load_auth_token(backend: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE_NAME, backend).context("Failed to access keyring")?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Keyring error: {e}")),
    }
}

pub fn save_auth_token(backend: &str, token: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, backend).context("Failed to access keyring")?;
    entry
        .set_password(token)
        .context("Failed to save token to keyring")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_expired_past_timestamp() {
        let tokens = StoredTokens {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: 0,
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn is_expired_future_timestamp() {
        let tokens = StoredTokens {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn is_expired_current_timestamp() {
        // Uses >= so current timestamp should be expired
        let tokens = StoredTokens {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: chrono::Utc::now().timestamp(),
        };
        assert!(tokens.is_expired());
    }
}
