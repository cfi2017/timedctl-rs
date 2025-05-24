use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::config::TimedConfig;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to decode token: {0}")]
    TokenDecode(String),

    #[error("Failed to open browser: {0}")]
    #[allow(dead_code)]
    Browser(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Token expired")]
    TokenExpired,
}

#[derive(Debug, Deserialize)]
struct OpenIdConfiguration {
    device_authorization_endpoint: String,
    token_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    #[allow(dead_code)]
    id_token: Option<String>,
    #[allow(dead_code)]
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Debug, serde::Deserialize)]
struct TokenClaims {
    exp: u64,
    #[allow(dead_code)]
    sub: String,
    #[allow(dead_code)]
    aud: String,
}

pub struct AuthClient {
    client: Client,
    config: TimedConfig,
}

impl AuthClient {
    pub fn new(config: TimedConfig) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    /// Get the OpenID Connect configuration from the discovery URL
    async fn get_openid_configuration(&self) -> Result<OpenIdConfiguration, AuthError> {
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.config.sso_discovery_url
        );
        debug!("Fetching OpenID configuration from: {}", discovery_url);

        let response = self.client.get(&discovery_url).send().await.map_err(|e| {
            error!("Failed to fetch OpenID configuration: {}", e);
            AuthError::Http(e)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                "Failed to fetch OpenID configuration, status: {}, body: {}",
                status, text
            );
            return Err(AuthError::AuthFailed(format!(
                "Failed to fetch OpenID configuration: HTTP {}: {}",
                status, text
            )));
        }

        response.json::<OpenIdConfiguration>().await.map_err(|e| {
            error!("Failed to parse OpenID configuration: {}", e);
            AuthError::TokenDecode(format!("Failed to parse OpenID configuration: {}", e))
        })
    }

    /// Start the device flow authentication process
    async fn start_device_flow(
        &self,
        oidc_config: &OpenIdConfiguration,
    ) -> Result<DeviceAuthResponse, AuthError> {
        let device_auth_url = &oidc_config.device_authorization_endpoint;
        debug!(
            "Starting device flow authentication at: {}",
            device_auth_url
        );

        let response = self
            .client
            .post(device_auth_url)
            .form(&[
                ("client_id", self.config.sso_client_id.as_str()),
                ("scope", "openid profile email"),
            ])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to start device flow: {}", e);
                AuthError::Http(e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                "Device flow request failed, status: {}, body: {}",
                status, text
            );
            return Err(AuthError::AuthFailed(format!(
                "Device flow request failed: HTTP {}: {}",
                status, text
            )));
        }

        response.json::<DeviceAuthResponse>().await.map_err(|e| {
            error!("Failed to parse device auth response: {}", e);
            AuthError::TokenDecode(format!("Failed to parse device auth response: {}", e))
        })
    }

    /// Poll for a token after starting the device flow
    async fn poll_for_token(
        &self,
        oidc_config: &OpenIdConfiguration,
        device_auth: &DeviceAuthResponse,
    ) -> Result<TokenResponse, AuthError> {
        let token_url = &oidc_config.token_endpoint;
        debug!("Polling for token at: {}", token_url);

        let interval = device_auth.interval.unwrap_or(5);
        let mut current_interval = interval;

        let start_time = SystemTime::now();
        let expires_at = start_time + Duration::from_secs(device_auth.expires_in);

        // Define a max number of retries for transient errors
        let mut retries = 5;
        let mut backoff_delay = 1;

        loop {
            // Check if the device code has expired
            if SystemTime::now() > expires_at {
                return Err(AuthError::AuthFailed("Device code expired".to_string()));
            }

            // Wait before polling
            time::sleep(Duration::from_secs(current_interval)).await;

            debug!("Polling for token...");
            let response = match self
                .client
                .post(token_url)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", &device_auth.device_code),
                    ("client_id", &self.config.sso_client_id),
                ])
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    if retries > 0 {
                        retries -= 1;
                        warn!(
                            "Transient error during token polling, retrying: {} ({} retries left)",
                            e, retries
                        );
                        // Use exponential backoff
                        time::sleep(Duration::from_secs(backoff_delay)).await;
                        backoff_delay *= 2;
                        continue;
                    } else {
                        error!("Failed to poll for token: {}", e);
                        return Err(AuthError::Http(e));
                    }
                }
            };

            if response.status().is_success() {
                debug!("Successfully received token");
                return response.json::<TokenResponse>().await.map_err(|e| {
                    error!("Failed to parse token response: {}", e);
                    AuthError::TokenDecode(format!("Failed to parse token response: {}", e))
                });
            }

            let error_response: serde_json::Value = match response.json().await {
                Ok(val) => val,
                Err(e) => {
                    warn!("Failed to parse error response: {}", e);
                    // Continue polling as if we got authorization_pending
                    continue;
                }
            };

            let error = error_response["error"].as_str().unwrap_or("unknown");

            match error {
                "authorization_pending" => {
                    debug!("Authorization pending, continuing to poll");
                    // Reset interval to default for next attempt
                    current_interval = interval;
                    continue;
                }
                "slow_down" => {
                    // If we get a slow_down error, increase the interval
                    current_interval = interval + 5;
                    debug!(
                        "Received slow_down error, increasing polling interval to {}",
                        current_interval
                    );
                    continue;
                }
                "expired_token" => {
                    return Err(AuthError::TokenExpired);
                }
                "access_denied" => {
                    return Err(AuthError::AuthFailed("Access denied by user".to_string()));
                }
                _ => {
                    warn!("Unexpected error during token polling: {}", error);
                    if retries > 0 {
                        retries -= 1;
                        warn!("Retrying token polling ({} retries left)", retries);
                        time::sleep(Duration::from_secs(backoff_delay)).await;
                        backoff_delay *= 2;
                        continue;
                    } else {
                        return Err(AuthError::AuthFailed(format!(
                            "Failed to obtain token: {}",
                            error
                        )));
                    }
                }
            }
        }
    }

    /// Decode a JWT token to extract claims
    fn decode_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        // JWT is in the format header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::TokenDecode("Invalid token format".to_string()));
        }

        // Decode the payload (second part)
        let payload = URL_SAFE_NO_PAD.decode(parts[1]).map_err(|e| {
            AuthError::TokenDecode(format!("Failed to decode token payload: {}", e))
        })?;

        // Parse the JSON claims
        let claims: TokenClaims =
            serde_json::from_slice(&payload).map_err(|e| AuthError::TokenDecode(e.to_string()))?;

        Ok(claims)
    }

    /// Check if the token is expired
    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.decode_token(token) {
            Ok(claims) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Add a buffer time of 1 hour (3600 seconds) to prevent frequent re-authentication
                claims.exp <= now + 3600
            }
            Err(e) => {
                warn!("Failed to decode token for expiration check: {}", e);
                true // Assume expired if we can't decode
            }
        }
    }

    /// Authenticate with the OpenID Connect provider using device flow
    pub async fn authenticate(&self) -> Result<String, AuthError> {
        info!("Starting authentication with OpenID Connect device flow");

        // Get OpenID configuration
        let oidc_config = match self.get_openid_configuration().await {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to get OpenID configuration: {}", e);
                return Err(e);
            }
        };

        // Start device flow
        let device_auth = match self.start_device_flow(&oidc_config).await {
            Ok(auth) => auth,
            Err(e) => {
                error!("Failed to start device flow: {}", e);
                return Err(e);
            }
        };

        // Display authentication information
        info!("Authentication code: {}", device_auth.user_code);
        info!("Please visit: {}", device_auth.verification_uri);
        println!("\n🔐 Authentication Required");
        println!("-------------------------");
        println!("Visit:  {}", device_auth.verification_uri);
        println!("Enter code: {}", device_auth.user_code);
        println!("-------------------------\n");

        // Try to open the browser
        match webbrowser::open(&device_auth.verification_uri_complete) {
            Ok(_) => info!("Opened browser for authentication"),
            Err(e) => {
                warn!("Failed to open browser: {}", e);
                warn!("Please manually visit the URL and enter the code");
            }
        }

        // Poll for the token
        info!("Waiting for authentication to complete...");
        let token_response = match self.poll_for_token(&oidc_config, &device_auth).await {
            Ok(response) => response,
            Err(e) => {
                error!("Authentication failed during token polling: {}", e);
                return Err(e);
            }
        };

        info!("Authentication successful");

        // Store the token in the config
        match self.config.store_token(&token_response.access_token) {
            Ok(_) => info!("Access token stored securely"),
            Err(e) => {
                error!("Failed to store token: {}", e);
                warn!("Authentication succeeded but token could not be stored");
                warn!("You may need to authenticate again in the future");
                // Continue anyway since we have a valid token
            }
        }

        // Also store refresh token if available
        if let Some(refresh_token) = &token_response.refresh_token {
            match self.config.store_refresh_token(refresh_token) {
                Ok(_) => info!("Refresh token stored securely"),
                Err(e) => {
                    warn!("Failed to store refresh token: {}", e);
                    warn!("Token refresh may not work, requiring re-authentication");
                    // Continue anyway since we have a valid access token
                }
            }
        }

        Ok(token_response.access_token)
    }

    /// Try to refresh an access token using the refresh token
    async fn refresh_token(&self, refresh_token: &str) -> Result<String, AuthError> {
        debug!("Attempting to refresh token");

        // Get OpenID configuration to get token endpoint
        let oidc_config = self.get_openid_configuration().await?;
        let token_url = oidc_config.token_endpoint;

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.config.sso_client_id),
            ("refresh_token", refresh_token),
        ];

        let client = reqwest::Client::new();
        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(AuthError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            warn!("Token refresh failed with status {}: {}", status, text);
            return Err(AuthError::AuthFailed(format!(
                "Token refresh failed: {}",
                text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenDecode(e.to_string()))?;

        // Store the new token
        self.config
            .store_token(&token_response.access_token)
            .map_err(|e| AuthError::AuthFailed(format!("Failed to store token: {}", e)))?;

        // Also store the refresh token if we got a new one
        if let Some(refresh) = &token_response.refresh_token {
            self.config.store_refresh_token(refresh).map_err(|e| {
                AuthError::AuthFailed(format!("Failed to store refresh token: {}", e))
            })?;
        }

        info!("Successfully refreshed access token");
        Ok(token_response.access_token)
    }

    /// Ensure we have a valid token, refreshing or renewing if necessary
    pub async fn ensure_valid_token(&self) -> Result<String, AuthError> {
        // Check if we have a stored token
        if !self.config.has_token() {
            info!("No authentication token found, starting authentication flow...");
            return self.authenticate().await;
        }

        // Try to get the token from secure storage
        let token = match self.config.get_token() {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to retrieve stored token: {}", e);
                info!("Starting new authentication flow...");
                return self.authenticate().await;
            }
        };

        // Check if the token is expired
        if self.is_token_expired(&token) {
            info!("Authentication token has expired");

            // Try to use refresh token if available
            if let Ok(refresh_token) = self.config.get_refresh_token() {
                match self.refresh_token(&refresh_token).await {
                    Ok(new_token) => {
                        debug!("Successfully refreshed token");
                        return Ok(new_token);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to refresh token: {}, starting new authentication flow",
                            e
                        );
                        return self.authenticate().await;
                    }
                }
            } else {
                info!("No refresh token available, starting new authentication flow...");
                return self.authenticate().await;
            }
        }

        debug!("Using existing valid authentication token");
        Ok(token)
    }

    /// Force renewal of the authentication token
    pub async fn force_renew_token(&self) -> Result<String, AuthError> {
        info!("Forcing token renewal");

        // Delete existing token if any
        let _ = self.config.delete_token();

        // Start a new authentication flow
        self.authenticate().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TimedConfig;

    #[test]
    fn test_is_token_expired() {
        // Create a test config
        let config = TimedConfig {
            username: "test".to_string(),
            ..TimedConfig::default()
        };

        let auth_client = AuthClient::new(config);

        // This is an invalid token, so it should be considered expired
        let invalid_token = "invalid.token.format";
        assert!(auth_client.is_token_expired(invalid_token));

        // We can't easily test a valid token without mocking JWT libraries
    }
}
