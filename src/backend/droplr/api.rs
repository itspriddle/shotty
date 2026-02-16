use anyhow::{Context, Result};
use reqwest::Client;

use super::types::UploadResponse;

const API_BASE: &str = "https://api.droplr.com";

pub fn build_upload_url(filename: &str) -> String {
    let encoded = urlencoding::encode(filename);
    format!("{API_BASE}/files.json?filename={encoded}")
}

fn detect_mime_type(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

pub async fn upload_file(
    client: &Client,
    auth_token: &str,
    filename: &str,
    data: &[u8],
) -> Result<String> {
    let url = build_upload_url(filename);
    let mime = detect_mime_type(filename);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Basic {auth_token}"))
        .header("Content-Type", mime)
        .body(data.to_vec())
        .send()
        .await
        .context("Failed to send upload request")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Droplr API error ({status}): {body}");
    }

    let upload: UploadResponse = resp
        .json()
        .await
        .context("Failed to parse upload response")?;
    Ok(upload.content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_upload_url_encodes_filename() {
        let url = build_upload_url("my screenshot (1).png");
        assert_eq!(
            url,
            "https://api.droplr.com/files.json?filename=my%20screenshot%20%281%29.png"
        );
    }

    #[test]
    fn build_upload_url_simple_filename() {
        let url = build_upload_url("image.png");
        assert_eq!(url, "https://api.droplr.com/files.json?filename=image.png");
    }

    #[test]
    fn detect_mime_type_png() {
        assert_eq!(detect_mime_type("screenshot.png"), "image/png");
    }

    #[test]
    fn detect_mime_type_jpeg() {
        assert_eq!(detect_mime_type("photo.jpg"), "image/jpeg");
    }

    #[test]
    fn detect_mime_type_unknown_extension() {
        assert_eq!(
            detect_mime_type("data.xyz123unknown"),
            "application/octet-stream"
        );
    }
}
