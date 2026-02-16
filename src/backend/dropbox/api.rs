use anyhow::{Context, Result};
use reqwest::Client;

use super::types::*;
use crate::error::ShottyError;

const UPLOAD_URL: &str = "https://content.dropboxapi.com/2/files/upload";
const SHARED_LINK_URL: &str =
    "https://api.dropboxapi.com/2/sharing/create_shared_link_with_settings";
const TOKEN_URL: &str = "https://api.dropboxapi.com/oauth2/token";

pub async fn upload_file(
    client: &Client,
    access_token: &str,
    path: &str,
    data: &[u8],
) -> Result<UploadResult> {
    let arg = UploadArg {
        path: path.to_string(),
        mode: "add".to_string(),
        autorename: true,
        mute: false,
    };
    let arg_json = serde_json::to_string(&arg).context("Failed to serialize upload arg")?;

    let resp = client
        .post(UPLOAD_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Dropbox-API-Arg", arg_json)
        .header("Content-Type", "application/octet-stream")
        .body(data.to_vec())
        .send()
        .await
        .context("Failed to send upload request")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ShottyError::UploadFailed(format!("{status}: {body}")).into());
    }

    resp.json::<UploadResult>()
        .await
        .context("Failed to parse upload response")
}

pub async fn create_shared_link(client: &Client, access_token: &str, path: &str) -> Result<String> {
    let body = CreateSharedLinkArg {
        path: path.to_string(),
        settings: SharedLinkSettings {
            requested_visibility: "public".to_string(),
        },
    };

    let resp = client
        .post(SHARED_LINK_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&body)
        .send()
        .await
        .context("Failed to send shared link request")?;

    if resp.status().is_success() {
        let meta: SharedLinkMetadata = resp
            .json()
            .await
            .context("Failed to parse shared link response")?;
        return Ok(transform_url(&meta.url));
    }

    // Handle "shared_link_already_exists" — extract existing link from error
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.as_u16() == 409
        && let Ok(err) = serde_json::from_str::<SharedLinkError>(&body)
        && let Some(existing) = err
            .error
            .shared_link_already_exists
            .and_then(|e| e.metadata)
    {
        return Ok(transform_url(&existing.url));
    }

    Err(ShottyError::UploadFailed(format!("Shared link creation failed ({status}): {body}")).into())
}

pub async fn refresh_access_token(
    client: &Client,
    app_key: &str,
    refresh_token: &str,
) -> Result<TokenResponse> {
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", app_key),
        ])
        .send()
        .await
        .context("Failed to send token refresh request")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ShottyError::TokenRefresh(format!("{status}: {body}")).into());
    }

    resp.json::<TokenResponse>()
        .await
        .context("Failed to parse token refresh response")
}

fn transform_url(url: &str) -> String {
    // Modern Dropbox shared URLs: https://www.dropbox.com/scl/fi/HASH/file?rlkey=KEY&dl=0
    // Direct download: https://dl.dropboxusercontent.com/scl/fi/HASH/file?rlkey=KEY&dl=1
    let parsed = url::Url::parse(url);
    let Ok(parsed) = parsed else {
        return url.to_string();
    };

    let rlkey = parsed
        .query_pairs()
        .find(|(k, _)| k == "rlkey")
        .map(|(_, v)| v.into_owned());

    let base = url.split('?').next().unwrap_or(url);
    let base = base.replace("www.dropbox.com", "dl.dropboxusercontent.com");

    match rlkey {
        Some(key) => format!("{base}?rlkey={key}&dl=1"),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_url_with_rlkey() {
        let url = "https://www.dropbox.com/scl/fi/abc123/photo.png?rlkey=xyz789&dl=0";
        let result = transform_url(url);
        assert_eq!(
            result,
            "https://dl.dropboxusercontent.com/scl/fi/abc123/photo.png?rlkey=xyz789&dl=1"
        );
    }

    #[test]
    fn transform_url_without_rlkey() {
        let url = "https://www.dropbox.com/scl/fi/abc123/photo.png?dl=0";
        let result = transform_url(url);
        assert_eq!(
            result,
            "https://dl.dropboxusercontent.com/scl/fi/abc123/photo.png"
        );
    }

    #[test]
    fn transform_url_already_direct() {
        let url = "https://dl.dropboxusercontent.com/scl/fi/abc123/photo.png?rlkey=xyz789&dl=1";
        let result = transform_url(url);
        assert_eq!(
            result,
            "https://dl.dropboxusercontent.com/scl/fi/abc123/photo.png?rlkey=xyz789&dl=1"
        );
    }

    #[test]
    fn transform_url_invalid() {
        let url = "not a url";
        assert_eq!(transform_url(url), "not a url");
    }
}
