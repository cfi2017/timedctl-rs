use anyhow::Result;
use chrono::Local;
use tracing::{debug, info};

use libtimed::{
    models::{Absence, AbsenceAttributes, AbsenceRelationships, AbsenceType, FilterParams, RelationshipData, RelationshipResource, ResourceResponse, ResourcesResponse},
    TimedClient,
};

/// List absences for the current user or all users
pub async fn list_absences(
    client: &TimedClient, 
    date_str: Option<&str>,
    from_str: Option<&str>,
    to_str: Option<&str>,
    all_users: bool,
) -> Result<()> {
    // Create filter for absences
    let mut filter = FilterParams::default();
    
    // Handle date filtering with priority: specific date > date range > today
    if let Some(date) = date_str {
        // Specific date has highest priority
        debug!("Getting absences for specific date: {}", date);
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
        debug!("Getting absences for today: {}", today);
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
    filter.include = Some("absence-type,user".to_string());
    
    let response = client
        .get::<ResourcesResponse<Absence>>("absences", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        if date_str.is_some() {
            println!("No absences found for date: {}", date_str.unwrap());
        } else if from_str.is_some() || to_str.is_some() {
            let from_msg = from_str.map_or("today", |d| d);
            let to_msg = to_str.map_or("today", |d| d);
            println!("No absences found from {} to {}", from_msg, to_msg);
        } else {
            println!("No absences found for today");
        }
        return Ok(());
    }
    
    // Display date range in the header
    if date_str.is_some() {
        println!("Absences for {}", date_str.unwrap());
    } else if from_str.is_some() || to_str.is_some() {
        let from_msg = from_str.map_or("today", |d| d);
        let to_msg = to_str.map_or("today", |d| d);
        println!("Absences from {} to {}", from_msg, to_msg);
    } else {
        let today = Local::now().date_naive().format("%Y-%m-%d");
        println!("Absences for {}", today);
    }
    
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each absence
    for absence in response.data {
        let date = absence.attributes.date;
        let comment = absence.attributes.comment.unwrap_or_else(|| "-".to_string());
        
        // Get absence type info
        let mut absence_type_name = "Unknown";
        if let Some(absence_type_rel) = &absence.relationships.absence_type {
            if let Some(absence_type_res) = &absence_type_rel.data {
                let absence_type_id = &absence_type_res.id;
                
                // Find absence type in included data
                if let Some(absence_type) = included.iter().find(|inc| {
                    inc.type_name == "absence-types" && inc.id == *absence_type_id
                }) {
                    if let Some(name) = absence_type.attributes.get("name").and_then(|n| n.as_str()) {
                        absence_type_name = name;
                    }
                }
            }
        }
        
        // Get user info if we're showing all users
        let mut user_prefix = "".to_string();
        if all_users {
            if let Some(user_rel) = &absence.relationships.user {
                if let Some(user_res) = &user_rel.data {
                    let user_id = &user_res.id;
                    
                    // Find user in included data
                    if let Some(user) = included.iter().find(|inc| {
                        inc.type_name == "users" && inc.id == *user_id
                    }) {
                        if let Some(username) = user.attributes.get("username").and_then(|n| n.as_str()) {
                            user_prefix = format!("[{}] ", username);
                        }
                    }
                }
            }
        }
        
        println!("{}Date: {} | Type: {} | Comment: {}", 
                 user_prefix, date, absence_type_name, comment);
    }
    
    Ok(())
}

/// Create a new absence for the current user
pub async fn create_absence(
    client: &TimedClient,
    date_str: &str,
    absence_type_id: &str,
    comment: Option<&str>,
) -> Result<()> {
    // Get current user info
    let current_user_response = client.get::<serde_json::Value>("users/me", None).await?;
    let user_id = current_user_response["data"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get user ID"))?;
    
    // Create absence
    let absence = Absence {
        id: None,
        type_name: "absences".to_string(),
        attributes: AbsenceAttributes {
            date: date_str.to_string(),
            comment: comment.map(|s| s.to_string()),
        },
        relationships: AbsenceRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: user_id.to_string(),
                }),
            }),
            absence_type: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "absence-types".to_string(),
                    id: absence_type_id.to_string(),
                }),
            }),
        },
    };
    
    // Post to API
    let request_body = serde_json::json!({
        "data": absence
    });
    
    let response = client
        .post::<_, ResourceResponse<Absence>>("absences", &request_body)
        .await?;
    
    info!("Created absence for {} with ID: {:?}", date_str, response.data.id);
    println!("Created absence for {}", date_str);
    
    Ok(())
}

/// Delete an absence by ID
pub async fn delete_absence(client: &TimedClient, absence_id: &str) -> Result<()> {
    let endpoint = format!("absences/{}", absence_id);
    client.delete(&endpoint).await?;
    
    info!("Deleted absence with ID: {}", absence_id);
    println!("Deleted absence");
    
    Ok(())
}

/// List available absence types
pub async fn list_absence_types(client: &TimedClient) -> Result<()> {
    let response = client
        .get::<ResourcesResponse<AbsenceType>>("absence-types", None)
        .await?;
    
    if response.data.is_empty() {
        println!("No absence types found");
        return Ok(());
    }
    
    println!("Available absence types:");
    println!("----------------------------------------");
    
    for absence_type in response.data {
        println!("ID: {} | Name: {} | Fill worktime: {}", 
                 absence_type.id.unwrap_or_else(|| "N/A".to_string()), 
                 absence_type.attributes.name,
                 if absence_type.attributes.fill_worktime { "Yes" } else { "No" });
    }
    
    Ok(())
}