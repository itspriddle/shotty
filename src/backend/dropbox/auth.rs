use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sha2::{Digest, Sha256};
use url::Url;

use crate::credentials::{self, StoredTokens};

const AUTH_URL: &str = "https://www.dropbox.com/oauth2/authorize";
const TOKEN_URL: &str = "https://api.dropboxapi.com/oauth2/token";
const REDIRECT_URI: &str = "http://localhost:8756";

pub async fn run_pkce_flow(app_key: &str) -> Result<()> {
    let (code_verifier, code_challenge) = generate_pkce();
    let state = generate_state();

    let auth_url = format!(
        "{AUTH_URL}?client_id={app_key}\
         &response_type=code\
         &code_challenge={code_challenge}\
         &code_challenge_method=S256\
         &token_access_type=offline\
         &redirect_uri={REDIRECT_URI}\
         &state={state}"
    );

    eprintln!("Opening browser for Dropbox authorization...");
    if open::that(&auth_url).is_err() {
        eprintln!("Could not open browser. Please visit:\n{auth_url}");
    }

    let listener =
        TcpListener::bind("127.0.0.1:8756").context("Failed to bind to localhost:8756")?;
    eprintln!("Waiting for authorization callback on {REDIRECT_URI} ...");

    let (code, callback_state) = accept_callback(&listener)?;

    if callback_state != state {
        anyhow::bail!("CSRF state mismatch — possible attack. Aborting.");
    }

    let http_client = reqwest::Client::new();
    let token_response = http_client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", app_key),
            ("redirect_uri", REDIRECT_URI),
            ("code_verifier", code_verifier.as_str()),
        ])
        .send()
        .await
        .context("Failed to exchange authorization code")?;

    if !token_response.status().is_success() {
        let status = token_response.status();
        let body = token_response.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({status}): {body}");
    }

    let token_data: super::types::TokenResponse = token_response
        .json()
        .await
        .context("Failed to parse token response")?;

    let expires_at = chrono::Utc::now().timestamp() + token_data.expires_in;

    let stored = StoredTokens {
        access_token: token_data.access_token,
        refresh_token: token_data
            .refresh_token
            .context("No refresh token received — ensure token_access_type=offline")?,
        expires_at,
    };

    credentials::save_tokens("dropbox", &stored)?;
    eprintln!("Authentication successful.");

    Ok(())
}

fn generate_pkce() -> (String, String) {
    let verifier_bytes: [u8; 32] = rand::thread_rng().r#gen();
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(challenge);
    (code_verifier, code_challenge)
}

fn generate_state() -> String {
    let bytes: [u8; 16] = rand::thread_rng().r#gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn accept_callback(listener: &TcpListener) -> Result<(String, String)> {
    let (mut stream, _) = listener.accept().context("Failed to accept connection")?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .context("Failed to read request")?;

    // Parse: GET /path?query HTTP/1.1
    let path = request_line
        .split_whitespace()
        .nth(1)
        .context("Malformed HTTP request")?;

    let callback_url =
        Url::parse(&format!("http://localhost{path}")).context("Failed to parse callback URL")?;

    let code = callback_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.into_owned())
        .context("No authorization code in callback")?;

    let state = callback_url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .context("No state parameter in callback")?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h2>Shotty authorized!</h2><p>You can close this tab.</p></body></html>";
    let _ = stream.write_all(response.as_bytes());

    Ok((code, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_is_valid_base64url() {
        let (verifier, _) = generate_pkce();
        // base64url chars: A-Z, a-z, 0-9, -, _
        assert!(
            verifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn pkce_challenge_is_valid_base64url() {
        let (_, challenge) = generate_pkce();
        assert!(
            challenge
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn pkce_challenge_matches_spec() {
        let (verifier, challenge) = generate_pkce();
        // RFC 7636: challenge = BASE64URL(SHA256(verifier))
        let expected = Sha256::digest(verifier.as_bytes());
        let expected_b64 = URL_SAFE_NO_PAD.encode(expected);
        assert_eq!(challenge, expected_b64);
    }

    #[test]
    fn pkce_produces_different_values() {
        let (v1, _) = generate_pkce();
        let (v2, _) = generate_pkce();
        assert_ne!(v1, v2);
    }

    #[test]
    fn state_is_22_chars() {
        // 16 bytes → base64url without padding = 22 chars
        let state = generate_state();
        assert_eq!(state.len(), 22);
    }
}
