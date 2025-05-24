use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info};

use crate::config::{get_default_config_path, TimedConfig};

/// View the current configuration
pub fn view_config(config: &TimedConfig, config_path: &Path) -> Result<()> {
    println!("Configuration (from {})", config_path.display());
    println!("----------------------------------------");
    println!("Username: {}", config.username);
    println!("Timed URL: {}", config.timed_url);
    println!("SSO Discovery URL: {}", config.sso_discovery_url);
    println!("SSO Client ID: {}", config.sso_client_id);
    println!("----------------------------------------");

    // Check if token exists
    println!(
        "Authentication Token: {}",
        if config.has_token() {
            "Present"
        } else {
            "Not present"
        }
    );

    Ok(())
}

/// View the current configuration without loading the config first
pub fn view_config_without_loading(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file does not exist at {}",
            config_path.display()
        ));
    }

    let config_text =
        std::fs::read_to_string(config_path).context("Failed to read configuration file")?;

    let config: TimedConfig =
        toml::from_str(&config_text).context("Failed to parse configuration file")?;

    view_config(&config, config_path)
}

/// Set a configuration value
pub fn set_config(
    config: &mut TimedConfig,
    config_path: &Path,
    key: &str,
    value: &str,
) -> Result<()> {
    debug!("Setting configuration {} = {}", key, value);

    match key {
        "username" => config.username = value.to_string(),
        "timed_url" | "timedurl" => config.timed_url = value.to_string(),
        "sso_discovery_url" | "ssodiscoveryurl" => config.sso_discovery_url = value.to_string(),
        "sso_client_id" | "ssoclientid" => config.sso_client_id = value.to_string(),
        _ => return Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
    }

    // Validate config
    if config.username.is_empty() {
        return Err(anyhow::anyhow!("Username cannot be empty"));
    }

    // Save config
    let toml = toml::to_string_pretty(&config).context("Failed to serialize configuration")?;

    std::fs::write(config_path, toml).context("Failed to write configuration file")?;

    info!("Configuration updated successfully");
    println!("{} = {}", key, value);

    Ok(())
}

/// Set a configuration value without loading the config first
pub fn set_config_without_loading(config_path: &Path, key: &str, value: &str) -> Result<()> {
    debug!(
        "Setting configuration {} = {} (without loading)",
        key, value
    );

    // Make sure parent directories exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
    }

    // If the file doesn't exist, create a default config first
    let mut config = if config_path.exists() {
        let config_text =
            std::fs::read_to_string(config_path).context("Failed to read configuration file")?;

        toml::from_str(&config_text).context("Failed to parse configuration file")?
    } else {
        debug!("Configuration file doesn't exist, creating default");
        TimedConfig::default()
    };

    // Update the config
    set_config(&mut config, config_path, key, value)
}

/// Reset the configuration to defaults
pub fn reset_config(config_path: &Path) -> Result<()> {
    debug!("Resetting configuration to defaults");

    // Make sure parent directories exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create default config
    let _default_config = TimedConfig::default();

    // Save to file
    TimedConfig::create_default_config(config_path)?;

    info!("Configuration reset to defaults");
    println!("Configuration has been reset to defaults.");
    println!("Please update your username and other settings before using timedctl-rs.");

    Ok(())
}

/// Initialize the configuration if it doesn't exist
pub fn init_config(config_path: &Path) -> Result<()> {
    debug!("Initializing configuration if needed");

    if config_path.exists() {
        println!("Configuration already exists at {}", config_path.display());
        println!("To reset, use 'timedctl config reset'");
        return Ok(());
    }

    // Make sure parent directories exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create default config
    TimedConfig::create_default_config(config_path)?;

    info!("Created new configuration file");
    println!(
        "Created new configuration file at {}",
        config_path.display()
    );
    println!("Please update your settings with 'timedctl config set <key> <value>'");
    println!("Required keys:");
    println!("  - username");
    println!("  - timed_url");
    println!("  - sso_discovery_url");
    println!("  - sso_client_id");

    Ok(())
}

/// Show the location of the config file
pub fn config_path() -> Result<()> {
    let path = get_default_config_path()?;
    println!("Config file path: {}", path.display());

    if path.exists() {
        println!("File exists: Yes");
    } else {
        println!("File exists: No");
    }

    Ok(())
}
