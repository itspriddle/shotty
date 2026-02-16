use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub backend: Option<String>,

    #[serde(default)]
    pub dropbox: DropboxConfig,

    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DropboxConfig {
    pub app_key: Option<String>,
    pub upload_path: Option<String>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("shotty")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.apply_setting(key, value);
        self.save()?;
        Ok(())
    }

    fn apply_setting(&mut self, key: &str, value: &str) {
        match key {
            "backend" => self.backend = Some(value.to_string()),
            "dropbox.app_key" => self.dropbox.app_key = Some(value.to_string()),
            "dropbox.upload_path" => self.dropbox.upload_path = Some(value.to_string()),
            other => {
                // Handle dotted keys for unknown sections
                if let Some((section, field)) = other.split_once('.') {
                    let table = self
                        .extra
                        .entry(section.to_string())
                        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
                    if let toml::Value::Table(t) = table {
                        t.insert(field.to_string(), toml::Value::String(value.to_string()));
                    }
                } else {
                    self.extra
                        .insert(key.to_string(), toml::Value::String(value.to_string()));
                }
            }
        }
    }

    pub fn show(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize config")
    }

    pub fn backend_name(&self) -> Option<&str> {
        self.backend.as_deref()
    }

    pub fn dropbox_upload_path(&self) -> &str {
        self.dropbox
            .upload_path
            .as_deref()
            .unwrap_or("/Shotty/{YYYY-MM}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_upload_path() {
        let config = Config::default();
        assert_eq!(config.dropbox_upload_path(), "/Shotty/{YYYY-MM}");
    }

    #[test]
    fn custom_upload_path() {
        let config = Config {
            dropbox: DropboxConfig {
                upload_path: Some("/Custom/Path".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.dropbox_upload_path(), "/Custom/Path");
    }

    #[test]
    fn backend_name_none() {
        let config = Config::default();
        assert_eq!(config.backend_name(), None);
    }

    #[test]
    fn backend_name_some() {
        let config = Config {
            backend: Some("dropbox".to_string()),
            ..Default::default()
        };
        assert_eq!(config.backend_name(), Some("dropbox"));
    }

    #[test]
    fn apply_setting_backend() {
        let mut config = Config::default();
        config.apply_setting("backend", "dropbox");
        assert_eq!(config.backend.as_deref(), Some("dropbox"));
    }

    #[test]
    fn apply_setting_dropbox_app_key() {
        let mut config = Config::default();
        config.apply_setting("dropbox.app_key", "test_key");
        assert_eq!(config.dropbox.app_key.as_deref(), Some("test_key"));
    }

    #[test]
    fn apply_setting_dropbox_upload_path() {
        let mut config = Config::default();
        config.apply_setting("dropbox.upload_path", "/My/Path");
        assert_eq!(config.dropbox.upload_path.as_deref(), Some("/My/Path"));
    }

    #[test]
    fn apply_setting_unknown_dotted_key() {
        let mut config = Config::default();
        config.apply_setting("s3.bucket", "my-bucket");
        let table = config.extra.get("s3").unwrap();
        if let toml::Value::Table(t) = table {
            assert_eq!(
                t.get("bucket"),
                Some(&toml::Value::String("my-bucket".to_string()))
            );
        } else {
            panic!("expected table");
        }
    }

    #[test]
    fn apply_setting_unknown_flat_key() {
        let mut config = Config::default();
        config.apply_setting("theme", "dark");
        assert_eq!(
            config.extra.get("theme"),
            Some(&toml::Value::String("dark".to_string()))
        );
    }

    #[test]
    fn toml_round_trip() {
        let mut config = Config::default();
        config.apply_setting("backend", "dropbox");
        config.apply_setting("dropbox.app_key", "abc123");

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.backend.as_deref(), Some("dropbox"));
        assert_eq!(deserialized.dropbox.app_key.as_deref(), Some("abc123"));
    }
}
