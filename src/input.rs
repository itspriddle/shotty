use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{Context, Result};

use crate::error::ShottyError;

pub struct FileInput {
    pub filename: String,
    pub data: Vec<u8>,
}

pub fn resolve_input(file: Option<&str>, name: Option<&str>) -> Result<FileInput> {
    if let Some(path_str) = file {
        let path = Path::new(path_str);
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "upload".to_string());
        let data = fs::read(path).with_context(|| format!("Failed to read file: {path_str}"))?;
        Ok(FileInput { filename, data })
    } else if !io::stdin().is_terminal() {
        let mut data = Vec::new();
        io::stdin()
            .read_to_end(&mut data)
            .context("Failed to read from stdin")?;
        let filename = name.map(String::from).unwrap_or_else(|| {
            let now = chrono::Local::now();
            now.format("upload-%Y%m%d-%H%M%S.png").to_string()
        });
        Ok(FileInput { filename, data })
    } else {
        Err(ShottyError::NoInput.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_file_input() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        std::fs::write(&file_path, b"fake png data").unwrap();

        let input = resolve_input(Some(file_path.to_str().unwrap()), None).unwrap();
        assert_eq!(input.filename, "test.png");
        assert_eq!(input.data, b"fake png data");
    }

    #[test]
    fn missing_file_input() {
        let result = resolve_input(Some("/nonexistent/file.png"), None);
        assert!(result.is_err());
    }
}
