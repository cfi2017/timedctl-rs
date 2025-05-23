//! libtimed-rs: A Rust client library for the Timed API
//!
//! This library provides a type-safe interface to interact with the Timed API.
//! It handles serialization/deserialization of models, API calls, and data transformation.

use std::time::Duration;

use reqwest::{Client, ClientBuilder, header};
use serde::{Deserialize, Serialize};

use thiserror::Error;
use tracing::{debug, error};

pub mod models;
pub mod transforms;

use models::FilterParams;

/// Error types for the libtimed library
#[derive(Error, Debug)]
pub enum TimedError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),
}

/// Result type for libtimed operations
pub type Result<T> = std::result::Result<T, TimedError>;

/// Client for interacting with the Timed API
pub struct TimedClient {
    http_client: Client,
    base_url: String,
    token: Option<String>,
}

impl TimedClient {
    /// Create a new Timed API client
    pub fn new(base_url: &str, api_namespace: &str, token: Option<String>) -> Self {
        let http_client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
            
        let base_url = format!("{}/{}/", base_url, api_namespace);
        
        Self {
            http_client,
            base_url,
            token,
        }
    }
    
    /// Set the authentication token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }
    
    /// Check if the client has an authentication token
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }
    
    /// Get the authentication token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
    
    /// Get a reference to the underlying HTTP client
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }
    
    /// Get a reference to the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Make a GET request to the API
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self, 
        endpoint: &str, 
        params: Option<&FilterParams>
    ) -> Result<T> {
        if self.token.is_none() {
            return Err(TimedError::AuthenticationRequired);
        }
        
        let mut req = self.http_client.get(format!("{}{}", self.base_url, endpoint));
        
        // Add authentication header
        req = req.header(
            header::AUTHORIZATION,
            format!("Bearer {}", self.token.as_ref().unwrap())
        );
        
        // Add content type header
        req = req.header(
            header::CONTENT_TYPE,
            "application/vnd.api+json"
        );
        
        // Add query parameters if provided
        if let Some(p) = params {
            req = req.query(&p);
            debug!(
                "Making GET request to {} with params: {:?}", 
                endpoint, 
                serde_json::to_string(p).unwrap_or_else(|_| format!("{:?}", p))
            );
        } else {
            debug!("Making GET request to {}", endpoint);
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("API error ({}): {}", status, text);
            
            return match status.as_u16() {
                404 => Err(TimedError::NotFound(endpoint.to_string())),
                401 | 403 => Err(TimedError::AuthenticationRequired),
                _ => Err(TimedError::InvalidResponse(format!("HTTP {}: {}", status, text))),
            };
        }
        
        let data = response.json::<T>().await
        .map_err(|e| TimedError::Serialization(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;
        
        Ok(data)
    }
    
    /// Make a POST request to the API
    pub async fn post<T: Serialize + std::fmt::Debug, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        data: &T
    ) -> Result<R> {
        if self.token.is_none() {
            return Err(TimedError::AuthenticationRequired);
        }
        
        let req = self.http_client
            .post(format!("{}{}", self.base_url, endpoint))
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token.as_ref().unwrap())
            )
            .header(
                header::CONTENT_TYPE,
                "application/vnd.api+json"
            )
            .json(data);
        
        debug!(
            "Making POST request to {} with data: {}", 
            endpoint, 
            serde_json::to_string(data).unwrap_or_else(|_| format!("{:?}", data))
        );
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("API error ({}): {}", status, text);
            
            return match status.as_u16() {
                404 => Err(TimedError::NotFound(endpoint.to_string())),
                401 | 403 => Err(TimedError::AuthenticationRequired),
                _ => Err(TimedError::InvalidResponse(format!("HTTP {}: {}", status, text))),
            };
        }
        
        let result = response.json::<R>().await
            .map_err(|e| TimedError::Serialization(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;
        
        Ok(result)
    }
    
    /// Make a PATCH request to the API
    pub async fn patch<T: Serialize + std::fmt::Debug, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        data: &T
    ) -> Result<R> {
        if self.token.is_none() {
            return Err(TimedError::AuthenticationRequired);
        }
        
        let req = self.http_client
            .patch(format!("{}{}", self.base_url, endpoint))
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token.as_ref().unwrap())
            )
            .header(
                header::CONTENT_TYPE,
                "application/vnd.api+json"
            )
            .json(data);
        
        debug!(
            "Making PATCH request to {} with data: {}", 
            endpoint, 
            serde_json::to_string(data).unwrap_or_else(|_| format!("{:?}", data))
        );
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("API error ({}): {}", status, text);
            
            return match status.as_u16() {
                404 => Err(TimedError::NotFound(endpoint.to_string())),
                401 | 403 => Err(TimedError::AuthenticationRequired),
                _ => Err(TimedError::InvalidResponse(format!("HTTP {}: {}", status, text))),
            };
        }
        
        let result = response.json::<R>().await
            .map_err(|e| TimedError::Serialization(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;
        
        Ok(result)
    }
    
    /// Make a DELETE request to the API
    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        if self.token.is_none() {
            return Err(TimedError::AuthenticationRequired);
        }
        
        let req = self.http_client
            .delete(format!("{}{}", self.base_url, endpoint))
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token.as_ref().unwrap())
            );
        
        debug!("Making DELETE request to {}", endpoint);
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("API error ({}): {}", status, text);
            
            return match status.as_u16() {
                404 => Err(TimedError::NotFound(endpoint.to_string())),
                401 | 403 => Err(TimedError::AuthenticationRequired),
                _ => Err(TimedError::InvalidResponse(format!("HTTP {}: {}", status, text))),
            };
        }
        
        Ok(())
    }
}

/// API resource endpoints
#[derive(Debug, Clone, Copy)]
pub enum ApiResource {
    Users,
    Reports,
    Activities,
    WorktimeBalances,
    Customers,
    Projects,
    Tasks,
    Attendances,
    Absences,
    AbsenceTypes,
    YearStatistics,
    MonthStatistics,
    TaskStatistics,
    UserStatistics,
    ProjectStatistics,
    CustomerStatistics,
    WorkReports,
}

impl ApiResource {
    /// Convert the resource to its API path
    pub fn as_path(&self) -> &'static str {
        match self {
            Self::Users => "users",
            Self::Reports => "reports",
            Self::Activities => "activities",
            Self::WorktimeBalances => "worktime-balances",
            Self::Customers => "customers",
            Self::Projects => "projects",
            Self::Tasks => "tasks",
            Self::Attendances => "attendances",
            Self::Absences => "absences",
            Self::AbsenceTypes => "absence-types",
            Self::YearStatistics => "year-statistics",
            Self::MonthStatistics => "month-statistics",
            Self::TaskStatistics => "task-statistics",
            Self::UserStatistics => "user-statistics",
            Self::ProjectStatistics => "project-statistics",
            Self::CustomerStatistics => "customer-statistics",
            Self::WorkReports => "work-reports",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = TimedClient::new("https://example.com", "api/v1", None);
        assert_eq!(client.base_url, "https://example.com/api/v1/");
        assert!(!client.has_token());
    }
    
    #[test]
    fn test_client_with_token() {
        let client = TimedClient::new("https://example.com", "api/v1", Some("token123".to_string()));
        assert!(client.has_token());
    }
    
    #[test]
    fn test_api_resource_paths() {
        assert_eq!(ApiResource::Users.as_path(), "users");
        assert_eq!(ApiResource::Reports.as_path(), "reports");
        assert_eq!(ApiResource::Activities.as_path(), "activities");
        assert_eq!(ApiResource::WorktimeBalances.as_path(), "worktime-balances");
        assert_eq!(ApiResource::Customers.as_path(), "customers");
        assert_eq!(ApiResource::Projects.as_path(), "projects");
        assert_eq!(ApiResource::Tasks.as_path(), "tasks");
        assert_eq!(ApiResource::Attendances.as_path(), "attendances");
        assert_eq!(ApiResource::Absences.as_path(), "absences");
        assert_eq!(ApiResource::AbsenceTypes.as_path(), "absence-types");
        assert_eq!(ApiResource::YearStatistics.as_path(), "year-statistics");
        assert_eq!(ApiResource::MonthStatistics.as_path(), "month-statistics");
        assert_eq!(ApiResource::TaskStatistics.as_path(), "task-statistics");
        assert_eq!(ApiResource::UserStatistics.as_path(), "user-statistics");
        assert_eq!(ApiResource::ProjectStatistics.as_path(), "project-statistics");
        assert_eq!(ApiResource::CustomerStatistics.as_path(), "customer-statistics");
        assert_eq!(ApiResource::WorkReports.as_path(), "work-reports");
    }
    
    #[tokio::test]
    async fn test_get_unauthorized() {
        let client = TimedClient::new("https://example.com", "api/v1", None);
        let result: Result<serde_json::Value> = client.get("test", None).await;
        assert!(matches!(result, Err(TimedError::AuthenticationRequired)));
    }
    
    #[tokio::test]
    async fn test_post_unauthorized() {
        let client = TimedClient::new("https://example.com", "api/v1", None);
        let data = serde_json::json!({});
        let result: Result<serde_json::Value> = client.post("test", &data).await;
        assert!(matches!(result, Err(TimedError::AuthenticationRequired)));
    }
    
    #[tokio::test]
    async fn test_delete_unauthorized() {
        let client = TimedClient::new("https://example.com", "api/v1", None);
        let result = client.delete("test").await;
        assert!(matches!(result, Err(TimedError::AuthenticationRequired)));
    }
    
    // Simplified test that doesn't require mockito
    #[test]
    fn test_get_request_url_formation() {
        let client = TimedClient::new("https://example.com", "api/v1", Some("token123".to_string()));
        let url = format!("{}{}", client.base_url(), "users");
        assert_eq!(url, "https://example.com/api/v1/users");
    }
    
    // Simplified test that doesn't require mockito
    #[test]
    fn test_post_request_url_formation() {
        let client = TimedClient::new("https://example.com", "api/v1", Some("token123".to_string()));
        let url = format!("{}{}", client.base_url(), "reports");
        assert_eq!(url, "https://example.com/api/v1/reports");
    }
}