use anyhow::Result;
use chrono::{Local, NaiveDate};

use libtimed::{
    models::{FilterParams, User},
    TimedClient,
};

pub mod absence;
pub mod activity;
pub mod attendance;
pub mod config;
pub mod data;
pub mod report;
pub mod statistics;

/// Parse a date string or return today's date
pub fn parse_date(date_str: Option<&str>) -> Result<NaiveDate> {
    match date_str {
        Some(s) => {
            let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%d.%m.%Y"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%d-%m-%Y"))?;
            Ok(date)
        }
        None => Ok(Local::now().date_naive()),
    }
}

/// Format a duration in hours and minutes
#[allow(dead_code)]
pub fn format_duration(duration_str: &str) -> Result<String> {
    // Check if it's already in HH:MM format
    if duration_str.contains(':') {
        return Ok(duration_str.to_string());
    }

    // Parse as decimal hours
    let hours: f64 = duration_str.parse()?;
    let total_minutes = (hours * 60.0).round() as i64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    Ok(format!("{hours:02}:{minutes:02}:00"))
}

/// Get the current user
#[allow(dead_code)]
pub async fn get_current_user(client: &TimedClient) -> Result<User> {
    let filter = FilterParams::default();
    let response = client
        .get::<serde_json::Value>("users/me", Some(&filter))
        .await?;

    let user = serde_json::from_value::<User>(response["data"].clone())?;
    Ok(user)
}

/// Get overtime for a specific date
pub async fn get_overtime(client: &TimedClient, date_str: Option<&str>) -> Result<String> {
    let date = parse_date(date_str)?;

    let filter = FilterParams {
        date: Some(date.format("%Y-%m-%d").to_string()),
        ..Default::default()
    };

    let response = client
        .get::<serde_json::Value>("worktime-balances", Some(&filter))
        .await?;

    if let Some(data) = response["data"].as_array() {
        if let Some(balance) = data.first() {
            let balance_str = balance["attributes"]["balance"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid balance format"))?;

            return Ok(balance_str.to_string());
        }
    }

    Err(anyhow::anyhow!(
        "No overtime data found for the specified date"
    ))
}
