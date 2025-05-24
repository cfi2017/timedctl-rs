use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use config::{Config, File, FileFormat};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info};

const APP_NAME: &str = "timedctl";
const DEFAULT_TIMED_URL: &str = "https://timed.example.com";
const DEFAULT_SSO_DISCOVERY_URL: &str = "https://sso.example.com/realms/example";
const DEFAULT_SSO_CLIENT_ID: &str = "timed-client";

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Failed to load configuration: {0}")]
    LoadError(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimedConfig {
    pub username: String,
    pub timed_url: String,
    pub sso_discovery_url: String,
    pub sso_client_id: String,
}

impl Default for TimedConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            timed_url: DEFAULT_TIMED_URL.to_string(),
            sso_discovery_url: DEFAULT_SSO_DISCOVERY_URL.to_string(),
            sso_client_id: DEFAULT_SSO_CLIENT_ID.to_string(),
        }
    }
}

impl TimedConfig {
    /// Load configuration from the default location or a specified path
    pub fn load(custom_path: Option<&Path>) -> Result<Self, ConfigurationError> {
        let config_path = match custom_path {
            Some(path) => path.to_path_buf(),
            None => get_default_config_path()?,
        };

        debug!("Loading configuration from: {:?}", config_path);

        // Create default config if it doesn't exist
        if !config_path.exists() {
            info!(
                "Config file doesn't exist, creating default at {:?}",
                config_path
            );
            Self::create_default_config(&config_path)?;
        }

        let settings = Config::builder()
            .add_source(File::new(config_path.to_str().unwrap(), FileFormat::Toml))
            .build()?;

        let config: TimedConfig = settings.try_deserialize()?;

        // Validate config
        if config.username.is_empty() {
            return Err(ConfigurationError::MissingConfig("username".to_string()));
        }

        Ok(config)
    }

    /// Create a default configuration file
    pub fn create_default_config(path: &Path) -> Result<(), ConfigurationError> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = Self::default();
        let toml = toml::to_string_pretty(&default_config).map_err(|e| {
            ConfigurationError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize default configuration: {}", e),
            ))
        })?;

        fs::write(path, toml)?;
        info!("Created default configuration at {:?}", path);

        Ok(())
    }

    /// Store a token in the keyring
    pub fn store_token(&self, token: &str) -> Result<(), ConfigurationError> {
        let entry = Entry::new(APP_NAME, &self.username)?;
        entry.set_password(token)?;
        debug!("Stored token in keyring for user: {}", self.username);
        Ok(())
    }

    /// Store a refresh token in the keyring
    pub fn store_refresh_token(&self, token: &str) -> Result<(), ConfigurationError> {
        let entry = Entry::new(format!("{}_refresh", APP_NAME).as_str(), &self.username)?;
        entry.set_password(token)?;
        debug!(
            "Stored refresh token in keyring for user: {}",
            self.username
        );
        Ok(())
    }

    /// Retrieve a token from the keyring
    pub fn get_token(&self) -> Result<String, ConfigurationError> {
        let entry = Entry::new(APP_NAME, &self.username)?;
        let token = entry.get_password()?;
        debug!("Retrieved token from keyring for user: {}", self.username);
        Ok(token)
    }

    /// Retrieve a refresh token from the keyring
    pub fn get_refresh_token(&self) -> Result<String, ConfigurationError> {
        let entry = Entry::new(format!("{}_refresh", APP_NAME).as_str(), &self.username)?;
        let token = entry.get_password()?;
        debug!(
            "Retrieved refresh token from keyring for user: {}",
            self.username
        );
        Ok(token)
    }

    /// Delete a token from the keyring
    pub fn delete_token(&self) -> Result<(), ConfigurationError> {
        let entry = Entry::new(APP_NAME, &self.username)?;
        entry.delete_password()?;
        debug!("Deleted token from keyring for user: {}", self.username);

        // Also attempt to delete refresh token
        if let Ok(entry) = Entry::new(format!("{}_refresh", APP_NAME).as_str(), &self.username) {
            let _ = entry.delete_password();
            debug!(
                "Deleted refresh token from keyring for user: {}",
                self.username
            );
        }

        Ok(())
    }

    /// Check if a token exists in the keyring
    pub fn has_token(&self) -> bool {
        let entry = Entry::new(APP_NAME, &self.username).ok();
        if let Some(entry) = entry {
            entry.get_password().is_ok()
        } else {
            false
        }
    }

    /// Check if a refresh token exists in the keyring
    pub fn has_refresh_token(&self) -> bool {
        let entry = Entry::new(format!("{}_refresh", APP_NAME).as_str(), &self.username).ok();
        if let Some(entry) = entry {
            entry.get_password().is_ok()
        } else {
            false
        }
    }
}

/// Get the default configuration path
pub fn get_default_config_path() -> Result<PathBuf, ConfigurationError> {
    let mut path = dirs::config_dir().ok_or_else(|| {
        ConfigurationError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine config directory",
        ))
    })?;

    path.push(APP_NAME);
    path.push("config.toml");

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = TimedConfig::default();
        assert_eq!(config.timed_url, DEFAULT_TIMED_URL);
        assert_eq!(config.sso_discovery_url, DEFAULT_SSO_DISCOVERY_URL);
        assert_eq!(config.sso_client_id, DEFAULT_SSO_CLIENT_ID);
    }

    #[test]
    fn test_create_and_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        // Create default config
        TimedConfig::create_default_config(&config_path).unwrap();
        assert!(config_path.exists());

        // Load the config
        let config = TimedConfig::load(Some(&config_path)).unwrap_err();
        assert!(matches!(config, ConfigurationError::MissingConfig(_)));

        // Create a valid config
        let valid_config = TimedConfig {
            username: "testuser".to_string(),
            ..TimedConfig::default()
        };

        let toml = toml::to_string_pretty(&valid_config).unwrap();
        fs::write(&config_path, toml).unwrap();

        // Load the valid config
        let loaded_config = TimedConfig::load(Some(&config_path)).unwrap();
        assert_eq!(loaded_config.username, "testuser");
    }
}
