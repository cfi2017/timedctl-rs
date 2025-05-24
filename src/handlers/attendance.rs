use anyhow::Result;
use chrono::Local;
use tracing::{debug, info};

use libtimed::{
    models::{
        Attendance, AttendanceAttributes, AttendanceRelationships, FilterParams, RelationshipData,
        RelationshipResource, ResourceResponse, ResourcesResponse,
    },
    TimedClient,
};

/// List attendances for the current user or all users
#[allow(dead_code)]
pub async fn list_attendances(
    client: &TimedClient,
    date_str: Option<&str>,
    from_str: Option<&str>,
    to_str: Option<&str>,
    all_users: bool,
) -> Result<()> {
    // Create filter for attendances
    let mut filter = FilterParams::default();

    // Handle date filtering with priority: specific date > date range > today
    if let Some(date) = date_str {
        // Specific date has highest priority
        debug!("Getting attendances for specific date: {}", date);
        filter.date = Some(date.to_string());
    } else if from_str.is_some() || to_str.is_some() {
        // Date range has second priority
        if let Some(from) = from_str {
            debug!("Setting from_date: {}", from);
            filter.from_date = Some(from.to_string());
        }
        if let Some(to) = to_str {
            debug!("Setting to_date: {}", to);
            filter.to_date = Some(to.to_string());
        }
    } else {
        // Default to today if no date parameters provided
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        debug!("Getting attendances for today: {}", today);
        filter.date = Some(today);
    }

    // Add user filter unless all_users flag is set
    if !all_users {
        // Get current user ID from /users/me endpoint
        let current_user_response = client.get::<serde_json::Value>("users/me", None).await?;

        if let Some(user_id) = current_user_response["data"]["id"].as_str() {
            debug!("Filtering for current user: {}", user_id);
            filter.user = Some(user_id.to_string());
        }
    }

    // Include related entities for better display
    filter.include = Some("user".to_string());

    let response = client
        .get::<ResourcesResponse<Attendance>>("attendances", Some(&filter))
        .await?;

    // Display results
    if response.data.is_empty() {
        if date_str.is_some() {
            println!("No attendances found for date: {}", date_str.unwrap());
        } else if from_str.is_some() || to_str.is_some() {
            let from_msg = from_str.map_or("today", |d| d);
            let to_msg = to_str.map_or("today", |d| d);
            println!("No attendances found from {} to {}", from_msg, to_msg);
        } else {
            println!("No attendances found for today");
        }
        return Ok(());
    }

    // Display date range in the header
    if date_str.is_some() {
        println!("Attendances for {}", date_str.unwrap());
    } else if from_str.is_some() || to_str.is_some() {
        let from_msg = from_str.map_or("today", |d| d);
        let to_msg = to_str.map_or("today", |d| d);
        println!("Attendances from {} to {}", from_msg, to_msg);
    } else {
        let today = Local::now().date_naive().format("%Y-%m-%d");
        println!("Attendances for {}", today);
    }

    println!("----------------------------------------");

    // Get included data
    let included = response.included.unwrap_or_default();

    // Display each attendance
    for attendance in response.data {
        let date = attendance.attributes.date;
        let from_time = attendance.attributes.from_time;
        let to_time = attendance
            .attributes
            .to_time
            .unwrap_or_else(|| "-".to_string());

        // Get user info if we're showing all users
        let mut user_prefix = "".to_string();
        if all_users {
            if let Some(user_rel) = &attendance.relationships.user {
                if let Some(user_res) = &user_rel.data {
                    let user_id = &user_res.id;

                    // Find user in included data
                    if let Some(user) = included
                        .iter()
                        .find(|inc| inc.type_name == "users" && inc.id == *user_id)
                    {
                        if let Some(username) =
                            user.attributes.get("username").and_then(|n| n.as_str())
                        {
                            user_prefix = format!("[{}] ", username);
                        }
                    }
                }
            }
        }

        println!(
            "{}Date: {} | From: {} | To: {}",
            user_prefix, date, from_time, to_time
        );
    }

    Ok(())
}

/// Create a new attendance for the current user
#[allow(dead_code)]
pub async fn create_attendance(
    client: &TimedClient,
    date_str: &str,
    from_time: &str,
    to_time: Option<&str>,
) -> Result<()> {
    // Get current user info
    let current_user_response = client.get::<serde_json::Value>("users/me", None).await?;
    let user_id = current_user_response["data"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get user ID"))?;

    // Create attendance
    let attendance = Attendance {
        id: None,
        type_name: "attendances".to_string(),
        attributes: AttendanceAttributes {
            date: date_str.to_string(),
            from_time: from_time.to_string(),
            to_time: to_time.map(|s| s.to_string()),
        },
        relationships: AttendanceRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: user_id.to_string(),
                }),
            }),
        },
    };

    // Post to API
    let request_body = serde_json::json!({
        "data": attendance
    });

    let response = client
        .post::<_, ResourceResponse<Attendance>>("attendances", &request_body)
        .await?;

    info!(
        "Created attendance for {} with ID: {:?}",
        date_str, response.data.id
    );
    println!(
        "Created attendance for {} from {} to {}",
        date_str,
        from_time,
        to_time.unwrap_or("now")
    );

    Ok(())
}

/// Update an existing attendance record
#[allow(dead_code)]
pub async fn update_attendance(
    client: &TimedClient,
    attendance_id: &str,
    date_str: Option<&str>,
    from_time: Option<&str>,
    to_time: Option<&str>,
) -> Result<()> {
    // Fetch current attendance data
    let endpoint = format!("attendances/{}", attendance_id);
    let current = client
        .get::<ResourceResponse<Attendance>>(&endpoint, None)
        .await?;

    // Update with new values or keep existing ones
    let attendance = Attendance {
        id: Some(attendance_id.to_string()),
        type_name: "attendances".to_string(),
        attributes: AttendanceAttributes {
            date: date_str
                .map(|s| s.to_string())
                .unwrap_or(current.data.attributes.date),
            from_time: from_time
                .map(|s| s.to_string())
                .unwrap_or(current.data.attributes.from_time),
            to_time: to_time
                .map(|s| s.to_string())
                .or(current.data.attributes.to_time),
        },
        relationships: current.data.relationships,
    };

    // Patch to API
    let request_body = serde_json::json!({
        "data": attendance
    });

    let _response = client
        .patch::<_, ResourceResponse<Attendance>>(&endpoint, &request_body)
        .await?;

    info!("Updated attendance with ID: {}", attendance_id);
    println!("Updated attendance");

    Ok(())
}

/// Delete an attendance record by ID
#[allow(dead_code)]
pub async fn delete_attendance(client: &TimedClient, attendance_id: &str) -> Result<()> {
    let endpoint = format!("attendances/{}", attendance_id);
    client.delete(&endpoint).await?;

    info!("Deleted attendance with ID: {}", attendance_id);
    println!("Deleted attendance");

    Ok(())
}
