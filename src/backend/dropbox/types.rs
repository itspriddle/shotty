use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UploadArg {
    pub path: String,
    pub mode: String,
    pub autorename: bool,
    pub mute: bool,
}

#[derive(Debug, Deserialize)]
pub struct UploadResult {
    pub path_display: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SharedLinkSettings {
    pub requested_visibility: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSharedLinkArg {
    pub path: String,
    pub settings: SharedLinkSettings,
}

#[derive(Debug, Deserialize)]
pub struct SharedLinkMetadata {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SharedLinkError {
    pub error: SharedLinkErrorBody,
}

#[derive(Debug, Deserialize)]
pub struct SharedLinkErrorBody {
    #[serde(rename = ".tag")]
    #[allow(dead_code)]
    pub tag: String,
    pub shared_link_already_exists: Option<SharedLinkAlreadyExists>,
}

#[derive(Debug, Deserialize)]
pub struct SharedLinkAlreadyExists {
    pub metadata: Option<SharedLinkMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}
