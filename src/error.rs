use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ShottyError {
    #[error("No backend configured. Run `shotty config set backend dropbox` first.")]
    NoBackend,

    #[error("Unknown backend: {0}. Supported backends: dropbox, droplr")]
    UnknownBackend(String),

    #[error("Not authenticated. Run `shotty auth` first.")]
    NotAuthenticated,

    #[error("Token refresh failed: {0}")]
    TokenRefresh(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error(
        "No input provided. Pass a file path or pipe data via stdin.\n  shotty upload <file>\n  echo data | shotty upload --name file.txt"
    )]
    NoInput,

    #[error("Config error: {0}")]
    Config(String),

    #[error("Dropbox app_key not configured. Run `shotty config set dropbox.app_key <key>` first.")]
    MissingAppKey,
}
