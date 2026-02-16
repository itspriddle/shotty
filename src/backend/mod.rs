pub mod dropbox;
pub mod droplr;

use anyhow::Result;

use crate::config::Config;
use crate::error::ShottyError;

/// Enum dispatch for backends — avoids dyn-incompatible async trait issue.
pub enum Backend {
    Dropbox(dropbox::DropboxBackend),
    Droplr(droplr::DroplrBackend),
}

impl Backend {
    pub async fn authenticate(&self) -> Result<()> {
        match self {
            Backend::Dropbox(b) => b.authenticate().await,
            Backend::Droplr(b) => b.authenticate().await,
        }
    }

    pub async fn upload(&self, filename: &str, data: &[u8]) -> Result<String> {
        match self {
            Backend::Dropbox(b) => b.upload(filename, data).await,
            Backend::Droplr(b) => b.upload(filename, data).await,
        }
    }
}

pub fn create_backend(config: &Config) -> Result<Backend> {
    let name = config.backend_name().ok_or(ShottyError::NoBackend)?;
    match name {
        "dropbox" => Ok(Backend::Dropbox(dropbox::DropboxBackend::new(config)?)),
        "droplr" => Ok(Backend::Droplr(droplr::DroplrBackend::new())),
        other => Err(ShottyError::UnknownBackend(other.to_string()).into()),
    }
}
