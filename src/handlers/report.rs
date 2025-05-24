use anyhow::Result;
use chrono::{Local, NaiveDate};
use dialoguer::{theme::ColorfulTheme, Input};
use serde_json;
use tracing::{debug, info};

use libtimed::{models::FilterParams, TimedClient};

use super::parse_date;

/// Options for getting reports
#[derive(Debug, Clone)]
pub struct GetReportsOptions {
    pub date: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub all_users: bool,
    pub interactive: bool,
}

/// Options for adding a new report
#[derive(Debug, Clone)]
pub struct AddReportOptions {
    pub customer: Option<String>,
    pub project: Option<String>,
    pub task: Option<String>,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub show_archived: bool,
    pub review: bool,
    pub not_billable: bool,
    pub interactive: bool,
}

/// Get reports for a specified date or date range
pub async fn get_reports(
    client: &TimedClient,
    date_str: Option<&str>,
    from_str: Option<&str>,
    to_str: Option<&str>,
    all_users: bool,
    interactive: bool,
) -> Result<()> {
    let options = GetReportsOptions {
        date: date_str.map(String::from),
        from_date: from_str.map(String::from),
        to_date: to_str.map(String::from),
        all_users,
        interactive,
    };

    get_reports_with_options(client, options).await
}

async fn get_reports_with_options(client: &TimedClient, options: GetReportsOptions) -> Result<()> {
    // Determine date input method
    let (date_param, from_param, to_param) = determine_date_parameters(&options).await?;

    // Parse date for display purposes
    let date = if let Some(date) = date_param.as_deref() {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().naive_local().date())
    } else {
        Local::now().naive_local().date()
    };

    // Get reports for the date
    let mut filter = FilterParams {
        include: Some("task,task.project,task.project.customer,user".to_string()),
        ..Default::default()
    };

    // Handle date filtering with priority: specific date > date range > today
    if let Some(ref date) = date_param {
        // Specific date has highest priority
        debug!("Getting reports for specific date: {}", date);
        filter.date = Some(date.clone());
    } else if from_param.is_some() || to_param.is_some() {
        // Date range has second priority
        if let Some(ref from) = from_param {
            debug!("Setting from_date: {}", from);
            filter.from_date = Some(from.clone());
        }
        if let Some(ref to) = to_param {
            debug!("Setting to_date: {}", to);
            filter.to_date = Some(to.clone());
        }
    } else {
        // Default to today if no date parameters provided
        filter.date = Some(date.format("%Y-%m-%d").to_string());
    }

    // Add user filter unless all_users flag is set
    if !options.all_users {
        // Get current user ID from /users/me endpoint
        let current_user_response = client.get::<serde_json::Value>("users/me", None).await?;

        if let Some(user_id) = current_user_response["data"]["id"].as_str() {
            debug!("Filtering for current user: {}", user_id);
            filter.user = Some(user_id.to_string());
        }
    }

    let response = client
        .get::<serde_json::Value>("reports", Some(&filter))
        .await?;

    if let Some(reports) = response["data"].as_array() {
        if reports.is_empty() {
            if let Some(ref date) = date_param {
                println!("No reports found for date: {}", date);
            } else if from_param.is_some() || to_param.is_some() {
                let from_msg = from_param.as_deref().unwrap_or("today");
                let to_msg = to_param.as_deref().unwrap_or("today");
                println!("No reports found from {} to {}", from_msg, to_msg);
            } else {
                println!("No reports found for today");
            }
            return Ok(());
        }

        // Display date range in the header
        if let Some(ref date) = date_param {
            println!("Reports for {}", date);
        } else if from_param.is_some() || to_param.is_some() {
            let from_msg = from_param.as_deref().unwrap_or("today");
            let to_msg = to_param.as_deref().unwrap_or("today");
            println!("Reports from {} to {}", from_msg, to_msg);
        } else {
            let today = Local::now().date_naive().format("%Y-%m-%d");
            println!("Reports for {}", today);
        }
        println!("----------------------------------------");

        let mut total_duration = 0.0;

        for report in reports {
            let comment = report["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            let duration = report["attributes"]["duration"]
                .as_str()
                .unwrap_or("00:00:00");
            let review = report["attributes"]["review"].as_bool().unwrap_or(false);
            let not_billable = report["attributes"]["not-billable"]
                .as_bool()
                .unwrap_or(false);
            let verified = report["attributes"]["verified"].as_bool().unwrap_or(false);
            let rejected = report["attributes"]["rejected"].as_bool().unwrap_or(false);

            // Parse duration
            let parts: Vec<&str> = duration.split(':').collect();
            let hours: f64 = if parts.len() >= 2 {
                parts[0].parse::<f64>().unwrap_or(0.0)
                    + parts[1].parse::<f64>().unwrap_or(0.0) / 60.0
            } else {
                0.0
            };

            total_duration += hours;

            // Get task/project/customer info
            let task_id = report["relationships"]["task"]["data"]["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid task reference"))?;

            let mut task_name = "Unknown Task";
            let mut project_name = "Unknown Project";
            let mut customer_name = "Unknown Customer";

            if let Some(included) = response["included"].as_array() {
                // Find task
                if let Some(task) = included
                    .iter()
                    .find(|item| item["type"] == "tasks" && item["id"] == task_id)
                {
                    task_name = task["attributes"]["name"].as_str().unwrap_or(task_name);

                    // Find project
                    if let Some(project_id) =
                        task["relationships"]["project"]["data"]["id"].as_str()
                    {
                        if let Some(project) = included
                            .iter()
                            .find(|item| item["type"] == "projects" && item["id"] == project_id)
                        {
                            project_name = project["attributes"]["name"]
                                .as_str()
                                .unwrap_or(project_name);

                            // Find customer
                            if let Some(customer_id) =
                                project["relationships"]["customer"]["data"]["id"].as_str()
                            {
                                if let Some(customer) = included.iter().find(|item| {
                                    item["type"] == "customers" && item["id"] == customer_id
                                }) {
                                    customer_name = customer["attributes"]["name"]
                                        .as_str()
                                        .unwrap_or(customer_name);
                                }
                            }
                        }
                    }
                }
            }

            let mut prefix = "".to_string();
            let mut flags = Vec::new();

            if review {
                flags.push("REVIEW");
            }
            if not_billable {
                flags.push("NOT-BILLABLE");
            }
            if verified {
                flags.push("VERIFIED");
            }
            if rejected {
                flags.push("REJECTED");
            }

            let _flags_str = if !flags.is_empty() {
                format!(" [{}]", flags.join(", "))
            } else {
                "".to_string()
            };
            let mut flags = Vec::new();

            if review {
                flags.push("REVIEW");
            }
            if not_billable {
                flags.push("NOT-BILLABLE");
            }
            if verified {
                flags.push("VERIFIED");
            }
            if rejected {
                flags.push("REJECTED");
            }

            let _flags_str = if !flags.is_empty() {
                format!(" [{}]", flags.join(", "))
            } else {
                "".to_string()
            };

            // Show username if all_users flag is set
            if options.all_users {
                // Get user info
                let user_id = report["relationships"]["user"]["data"]["id"]
                    .as_str()
                    .unwrap_or("");
                let mut username = "Unknown";

                if let Some(included) = response["included"].as_array() {
                    if let Some(user) = included
                        .iter()
                        .find(|i| i["type"] == "users" && i["id"] == user_id)
                    {
                        username = user["attributes"]["username"].as_str().unwrap_or(username);
                    }
                }
                prefix = format!("[{}] ", username);
            }

            println!(
                "{}{} - {} / {} / {} - {}",
                prefix, duration, customer_name, project_name, task_name, comment
            );
        }

        println!("----------------------------------------");
        println!("Total: {:.2} hours", total_duration);
        return Ok(());
    }

    // If we get here, no reports were found
    if let Some(ref date) = date_param {
        println!("No reports found for date: {}", date);
    } else if from_param.is_some() || to_param.is_some() {
        let from_msg = from_param.as_deref().unwrap_or("today");
        let to_msg = to_param.as_deref().unwrap_or("today");
        println!("No reports found from {} to {}", from_msg, to_msg);
    } else {
        println!("No reports found for today");
    }
    Ok(())
}

/// Helper function to determine date parameters
async fn determine_date_parameters(
    options: &GetReportsOptions,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
    if options.interactive
        && options.date.is_none()
        && options.from_date.is_none()
        && options.to_date.is_none()
    {
        interactive_date_selection().await
    } else {
        Ok((
            options.date.clone(),
            options.from_date.clone(),
            options.to_date.clone(),
        ))
    }
}

/// Interactive date selection dialog
async fn interactive_date_selection() -> Result<(Option<String>, Option<String>, Option<String>)> {
    println!("Select date mode:");
    println!("1: Specific date");
    println!("2: Date range");
    println!("3: Today (default)");

    let mode: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Selection")
        .default("3".to_string())
        .interact_text()?;

    match mode.as_str() {
        "1" => {
            let date_input = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Date (YYYY-MM-DD)")
                .default(Local::now().format("%Y-%m-%d").to_string())
                .interact_text()?;
            Ok((Some(date_input), None, None))
        }
        "2" => {
            let from_date = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("From date (YYYY-MM-DD)")
                .default(Local::now().format("%Y-%m-%d").to_string())
                .interact_text()?;

            let to_date = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("To date (YYYY-MM-DD)")
                .default(from_date.clone())
                .interact_text()?;

            Ok((None, Some(from_date), Some(to_date)))
        }
        _ => Ok((None, None, None)),
    }
}

/// Get intersection of common values among reports that match a filter
#[allow(dead_code)]
pub async fn get_report_intersection(
    client: &TimedClient,
    filter_params: FilterParams,
) -> Result<()> {
    // Add the intersection endpoint to the filter
    let endpoint = "reports/intersection";

    // Add relationship includes if not already set
    let mut filter_with_includes = filter_params.clone();
    if filter_with_includes.include.is_none() {
        filter_with_includes.include =
            Some("task,task.project,task.project.customer,user".to_string());
    }

    // Make the request
    let response = client
        .get::<serde_json::Value>(endpoint, Some(&filter_with_includes))
        .await?;

    // Parse and display the intersection data
    if let Some(data) = response.get("data") {
        let map = serde_json::Map::new();
        let default_value = serde_json::Value::Object(map);
        let attributes = data.get("attributes").unwrap_or(&default_value);

        println!("Common values across matching reports:");
        println!("----------------------------------------");

        // Check for task
        if let Some(task) = attributes.get("task") {
            if !task.is_null() {
                if let Some(task_id) = task.get("id").and_then(|id| id.as_str()) {
                    println!("Task: {}", task_id);
                }
            } else {
                println!("Task: <varies>");
            }
        }

        // Check for review flag
        if let Some(review) = attributes.get("review") {
            if !review.is_null() {
                println!("Review: {}", review.as_bool().unwrap_or(false));
            } else {
                println!("Review: <varies>");
            }
        }

        // Check for not_billable flag
        if let Some(not_billable) = attributes.get("not-billable") {
            if !not_billable.is_null() {
                println!("Not Billable: {}", not_billable.as_bool().unwrap_or(false));
            } else {
                println!("Not Billable: <varies>");
            }
        }

        // Check for comment
        if let Some(comment) = attributes.get("comment") {
            if !comment.is_null() {
                println!("Comment: {}", comment.as_str().unwrap_or("<empty>"));
            } else {
                println!("Comment: <varies>");
            }
        }
    } else {
        println!("No intersection data found");
    }

    Ok(())
}

/// Bulk update reports based on a filter
#[allow(dead_code)]
pub async fn bulk_update_reports(
    client: &TimedClient,
    filter_params: FilterParams,
    review: Option<bool>,
    not_billable: Option<bool>,
    verified: Option<bool>,
    comment: Option<String>,
) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Confirm};
    // Add relationship includes if not already set
    let mut filter_with_includes = filter_params.clone();
    if filter_with_includes.include.is_none() {
        filter_with_includes.include =
            Some("task,task.project,task.project.customer,user".to_string());
    }

    // First, get the reports that match the filter
    let response = client
        .get::<serde_json::Value>("reports", Some(&filter_with_includes))
        .await?;

    let report_count = response["data"].as_array().map_or(0, |a| a.len());

    if report_count == 0 {
        println!("No reports found matching the filter criteria");
        return Ok(());
    }

    println!(
        "Found {} reports matching the filter criteria",
        report_count
    );

    // Confirm the bulk update
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to update {} reports?",
            report_count
        ))
        .default(false)
        .interact()?;

    if !confirm {
        println!("Bulk update canceled");
        return Ok(());
    }

    // Create the bulk update request
    let bulk_request = serde_json::json!({
        "review": review,
        "not-billable": not_billable,
        "verified": verified,
        "comment": comment
    });

    // Make the request
    let _response = client
        .post::<_, serde_json::Value>("reports/bulk", &bulk_request)
        .await?;

    println!("Successfully updated reports");

    Ok(())
}

/// Export reports based on a filter
#[allow(dead_code)]
pub async fn export_reports(
    client: &TimedClient,
    filter_params: FilterParams,
    file_type: &str,
    output_path: &std::path::Path,
) -> Result<()> {
    // Validate file type
    if !["csv", "xlsx", "ods"].contains(&file_type) {
        return Err(anyhow::anyhow!(
            "Invalid file type. Must be one of: csv, xlsx, ods"
        ));
    }

    // Add file_type to the filter
    let mut params = filter_params.clone();
    params
        .custom
        .insert("file_type".to_string(), file_type.to_string());

    // Use a custom call to get the raw bytes
    let endpoint = "reports/export";
    debug!("Exporting reports to {} format", file_type);

    // Special handling for export endpoint - custom headers and response type
    let url = format!("{}{}", client.base_url(), endpoint);

    let token = match client.token() {
        Some(t) => t,
        None => return Err(anyhow::anyhow!("No authentication token")),
    };

    let response = client
        .http_client()
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .query(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        return Err(anyhow::anyhow!("API error ({}): {}", status, text));
    }

    let bytes = response.bytes().await?;

    // Write to file
    std::fs::write(output_path, bytes)?;

    println!("Exported reports to {}", output_path.display());

    Ok(())
}

/// Add a new report
#[allow(clippy::too_many_arguments)]
pub async fn add_report(
    client: &TimedClient,
    customer: Option<&str>,
    project: Option<&str>,
    task: Option<&str>,
    description: Option<&str>,
    duration: Option<&str>,
    show_archived: bool,
    review: bool,
    not_billable: bool,
    interactive: bool,
) -> Result<()> {
    let options = AddReportOptions {
        customer: customer.map(String::from),
        project: project.map(String::from),
        task: task.map(String::from),
        description: description.map(String::from),
        duration: duration.map(String::from),
        show_archived,
        review,
        not_billable,
        interactive,
    };

    add_report_with_options(client, options).await
}

async fn add_report_with_options(client: &TimedClient, options: AddReportOptions) -> Result<()> {
    let task_id = get_task_for_report(client, &options).await?;
    let comment = get_description_for_report(&options)?;
    let duration_str = get_duration_for_report(&options)?;

    let report_data = serde_json::json!({
        "data": {
            "type": "reports",
            "attributes": {
                "comment": comment,
                "duration": duration_str,
                "review": options.review,
                "not-billable": options.not_billable
            },
            "relationships": {
                "task": {
                    "data": {
                        "type": "tasks",
                        "id": task_id
                    }
                }
            }
        }
    });

    let response = client
        .post::<_, serde_json::Value>("reports", &report_data)
        .await?;

    if response.get("data").is_some() {
        println!("Report added successfully");
        println!("Duration: {}", duration_str);
        println!("Description: {}", comment);
    } else {
        return Err(anyhow::anyhow!("Failed to create report"));
    }

    Ok(())
}

async fn get_task_for_report(client: &TimedClient, options: &AddReportOptions) -> Result<String> {
    if let (Some(customer), Some(project), Some(task)) =
        (&options.customer, &options.project, &options.task)
    {
        get_task_id(client, customer, project, task, options.show_archived).await
    } else if options.interactive {
        interactive_select_task(client, options.show_archived).await
    } else {
        Err(anyhow::anyhow!(
            "Task information required. Provide --customer, --project, and --task parameters or use interactive mode"
        ))
    }
}

fn get_description_for_report(options: &AddReportOptions) -> Result<String> {
    if let Some(desc) = &options.description {
        Ok(desc.clone())
    } else if options.interactive {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Report description")
            .interact_text()
            .map_err(Into::into)
    } else {
        Err(anyhow::anyhow!(
            "Description required. Provide --description parameter or use interactive mode"
        ))
    }
}

fn get_duration_for_report(options: &AddReportOptions) -> Result<String> {
    if let Some(dur) = &options.duration {
        round_duration_to_15min(dur)
    } else if options.interactive {
        interactive_duration_input()
    } else {
        Err(anyhow::anyhow!(
            "Duration required. Provide --duration parameter or use interactive mode"
        ))
    }
}

fn interactive_duration_input() -> Result<String> {
    let input_duration = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Duration (format: HH:MM or decimal hours)")
        .default("01:00".to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.contains(':') {
                let parts: Vec<&str> = input.split(':').collect();
                if parts.len() != 2 {
                    return Err("Duration must be in HH:MM format");
                }
                if let Ok(mins) = parts[1].parse::<u32>() {
                    if mins % 15 != 0 {
                        return Err("Minutes must be in 15-minute increments (00, 15, 30, 45)");
                    }
                } else {
                    return Err("Invalid minutes value");
                }
            }
            Ok(())
        })
        .interact_text()?;

    round_duration_to_15min(&input_duration)
}

fn round_duration_to_15min(duration_str: &str) -> Result<String> {
    if duration_str.contains(':') {
        let parts: Vec<&str> = duration_str.split(':').collect();

        let hours = parts[0]
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid hours value"))?;

        let minutes = if parts.len() > 1 {
            parts[1]
                .parse::<u32>()
                .map_err(|_| anyhow::anyhow!("Invalid minutes value"))?
        } else {
            0
        };

        let rounded_minutes = ((minutes as f32 / 15.0).round() * 15.0) as u32;
        let adjusted_hours = hours + (rounded_minutes / 60);
        let adjusted_minutes = rounded_minutes % 60;

        Ok(format!("{:02}:{:02}:00", adjusted_hours, adjusted_minutes))
    } else if let Ok(decimal_hours) = duration_str.parse::<f32>() {
        let total_minutes = (decimal_hours * 60.0).round() as u32;
        let rounded_minutes = ((total_minutes as f32 / 15.0).round() * 15.0) as u32;
        let hours = rounded_minutes / 60;
        let minutes = rounded_minutes % 60;

        Ok(format!("{:02}:{:02}:00", hours, minutes))
    } else {
        Err(anyhow::anyhow!("Invalid duration format"))
    }
}

/// Delete a report
pub async fn delete_report(
    client: &TimedClient,
    date_str: Option<&str>,
    all_users: bool,
    interactive: bool,
) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};

    // Determine date - either from parameter or interactive selection (default)
    let date = if let Some(date_str) = date_str {
        parse_date(Some(date_str))?
    } else if interactive {
        // Interactive date selection
        println!("Enter date to delete reports for (YYYY-MM-DD):");
        let date_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Date")
            .default(Local::now().format("%Y-%m-%d").to_string())
            .interact_text()?;

        NaiveDate::parse_from_str(&date_input, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().naive_local().date())
    } else {
        // Default to today if no date is provided and not interactive
        Local::now().naive_local().date()
    };

    // Get reports for the date
    let mut filter = FilterParams {
        date: Some(date.format("%Y-%m-%d").to_string()),
        include: Some("task,task.project,task.project.customer,user".to_string()),
        ..Default::default()
    };

    // Add user filter unless all_users flag is set
    if !all_users {
        // Get current user ID from /users/me endpoint
        let current_user_response = client.get::<serde_json::Value>("users/me", None).await?;

        if let Some(user_id) = current_user_response["data"]["id"].as_str() {
            filter.user = Some(user_id.to_string());
        }
    }

    let response = client
        .get::<serde_json::Value>("reports", Some(&filter))
        .await?;

    if let Some(reports) = response["data"].as_array() {
        if reports.is_empty() {
            return Err(anyhow::anyhow!(
                "No reports found for {}",
                date.format("%Y-%m-%d")
            ));
        }

        // Display list of reports with details
        println!("Reports for {}", date.format("%Y-%m-%d"));
        println!("----------------------------------------");

        for (i, report) in reports.iter().enumerate() {
            let comment = report["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            let duration = report["attributes"]["duration"]
                .as_str()
                .unwrap_or("00:00:00");

            // Get task/project/customer info
            let task_id = report["relationships"]["task"]["data"]["id"]
                .as_str()
                .unwrap_or("");

            let mut task_name = "Unknown Task";
            let mut project_name = "Unknown Project";
            let mut customer_name = "Unknown Customer";

            if let Some(included) = response["included"].as_array() {
                // Find task
                if let Some(task) = included
                    .iter()
                    .find(|item| item["type"] == "tasks" && item["id"] == task_id)
                {
                    task_name = task["attributes"]["name"].as_str().unwrap_or(task_name);

                    // Find project
                    if let Some(project_id) =
                        task["relationships"]["project"]["data"]["id"].as_str()
                    {
                        if let Some(project) = included
                            .iter()
                            .find(|item| item["type"] == "projects" && item["id"] == project_id)
                        {
                            project_name = project["attributes"]["name"]
                                .as_str()
                                .unwrap_or(project_name);

                            // Find customer
                            if let Some(customer_id) =
                                project["relationships"]["customer"]["data"]["id"].as_str()
                            {
                                if let Some(customer) = included.iter().find(|item| {
                                    item["type"] == "customers" && item["id"] == customer_id
                                }) {
                                    customer_name = customer["attributes"]["name"]
                                        .as_str()
                                        .unwrap_or(customer_name);
                                }
                            }
                        }
                    }
                }
            }

            let mut prefix = "".to_string();

            // Show username if all_users flag is set
            if all_users {
                // Get user info
                let user_id = report["relationships"]["user"]["data"]["id"]
                    .as_str()
                    .unwrap_or("");
                let mut username = "Unknown";

                if let Some(included) = response["included"].as_array() {
                    if let Some(user) = included
                        .iter()
                        .find(|i| i["type"] == "users" && i["id"] == user_id)
                    {
                        username = user["attributes"]["username"].as_str().unwrap_or(username);
                    }
                }
                prefix = format!("[{}] ", username);
            }

            println!(
                "{}. {}{} - {} / {} / {} - {}",
                i + 1,
                prefix,
                duration,
                customer_name,
                project_name,
                task_name,
                comment
            );
        }

        println!("----------------------------------------");

        // Interactive selection of report to delete
        let selection = if interactive {
            // Prepare report options for selection
            let mut report_options = Vec::new();
            for report in reports.iter() {
                let comment = report["attributes"]["comment"]
                    .as_str()
                    .unwrap_or("No comment");
                let duration = report["attributes"]["duration"]
                    .as_str()
                    .unwrap_or("00:00:00");
                let task_id = report["relationships"]["task"]["data"]["id"]
                    .as_str()
                    .unwrap_or("");

                let mut task_name = "Unknown Task";
                let mut project_name = "Unknown Project";
                let mut customer_name = "Unknown Customer";

                if let Some(included) = response["included"].as_array() {
                    // Find task
                    if let Some(task) = included
                        .iter()
                        .find(|item| item["type"] == "tasks" && item["id"] == task_id)
                    {
                        task_name = task["attributes"]["name"].as_str().unwrap_or(task_name);

                        // Find project
                        if let Some(project_id) =
                            task["relationships"]["project"]["data"]["id"].as_str()
                        {
                            if let Some(project) = included
                                .iter()
                                .find(|item| item["type"] == "projects" && item["id"] == project_id)
                            {
                                project_name = project["attributes"]["name"]
                                    .as_str()
                                    .unwrap_or(project_name);

                                // Find customer
                                if let Some(customer_id) =
                                    project["relationships"]["customer"]["data"]["id"].as_str()
                                {
                                    if let Some(customer) = included.iter().find(|item| {
                                        item["type"] == "customers" && item["id"] == customer_id
                                    }) {
                                        customer_name = customer["attributes"]["name"]
                                            .as_str()
                                            .unwrap_or(customer_name);
                                    }
                                }
                            }
                        }
                    }
                }

                report_options.push(format!(
                    "{} - {} / {} / {} - {}",
                    duration, customer_name, project_name, task_name, comment
                ));
            }

            // Add a cancel option
            report_options.push("Cancel".to_string());

            // Interactive selection
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a report to delete")
                .items(&report_options)
                .default(0)
                .interact()?;

            // Check if the user selected the cancel option
            if selection == reports.len() {
                info!("Deletion cancelled");
                return Ok(());
            }
            selection
        } else {
            // In non-interactive mode, we need to confirm deletion
            if reports.len() > 1 {
                println!("Multiple reports found for {}. Remove --non-interactive flag for interactive selection.", date.format("%Y-%m-%d"));
                if !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to delete the first report?")
                    .default(false)
                    .interact()?
                {
                    info!("Deletion cancelled");
                    return Ok(());
                }
            }
            0 // Use the first report in non-interactive mode
        };

        let selected = &reports[selection];
        let id = selected["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid report ID"))?;

        // Confirm deletion
        println!("Are you sure you want to delete this report? (y/N)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            client.delete(&format!("reports/{}", id)).await?;
            info!("Report deleted");
            return Ok(());
        } else {
            info!("Deletion cancelled");
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "No reports found for {}",
        date.format("%Y-%m-%d")
    ))
}

/// Edit a report
pub async fn edit_report(
    client: &TimedClient,
    date_str: Option<&str>,
    interactive: bool,
) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};

    // Determine date - either from parameter or interactive selection (default)
    let date = if let Some(date_str) = date_str {
        parse_date(Some(date_str))?
    } else if interactive {
        // Interactive date selection
        println!("Enter date to edit reports for (YYYY-MM-DD):");
        let date_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Date")
            .default(Local::now().format("%Y-%m-%d").to_string())
            .interact_text()?;

        NaiveDate::parse_from_str(&date_input, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().naive_local().date())
    } else {
        // Default to today if no date is provided and not interactive
        Local::now().naive_local().date()
    };

    // Get reports for the date
    let filter = FilterParams {
        date: Some(date.format("%Y-%m-%d").to_string()),
        include: Some("task,task.project,task.project.customer".to_string()),
        ..Default::default()
    };

    let response = client
        .get::<serde_json::Value>("reports", Some(&filter))
        .await?;

    if let Some(reports) = response["data"].as_array() {
        if reports.is_empty() {
            return Err(anyhow::anyhow!(
                "No reports found for {}",
                date.format("%Y-%m-%d")
            ));
        }

        let (selection, comment, duration, review, not_billable) = if interactive {
            // Prepare report options for selection
            let mut report_options = Vec::new();
            for report in reports.iter() {
                let comment = report["attributes"]["comment"]
                    .as_str()
                    .unwrap_or("No comment");
                let duration = report["attributes"]["duration"]
                    .as_str()
                    .unwrap_or("00:00:00");
                let task_id = report["relationships"]["task"]["data"]["id"]
                    .as_str()
                    .unwrap_or("");

                let mut task_name = "Unknown Task";
                let mut project_name = "Unknown Project";
                let mut customer_name = "Unknown Customer";

                if let Some(included) = response["included"].as_array() {
                    // Find task
                    if let Some(task) = included
                        .iter()
                        .find(|item| item["type"] == "tasks" && item["id"] == task_id)
                    {
                        task_name = task["attributes"]["name"].as_str().unwrap_or(task_name);

                        // Find project
                        if let Some(project_id) =
                            task["relationships"]["project"]["data"]["id"].as_str()
                        {
                            if let Some(project) = included
                                .iter()
                                .find(|item| item["type"] == "projects" && item["id"] == project_id)
                            {
                                project_name = project["attributes"]["name"]
                                    .as_str()
                                    .unwrap_or(project_name);

                                // Find customer
                                if let Some(customer_id) =
                                    project["relationships"]["customer"]["data"]["id"].as_str()
                                {
                                    if let Some(customer) = included.iter().find(|item| {
                                        item["type"] == "customers" && item["id"] == customer_id
                                    }) {
                                        customer_name = customer["attributes"]["name"]
                                            .as_str()
                                            .unwrap_or(customer_name);
                                    }
                                }
                            }
                        }
                    }
                }

                report_options.push(format!(
                    "{} - {} / {} / {} - {}",
                    duration, customer_name, project_name, task_name, comment
                ));
            }

            // Interactive selection of report to edit
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a report to edit")
                .items(&report_options)
                .default(0)
                .interact()?;
            let selected = &reports[selection];

            // Get current values
            let current_comment = selected["attributes"]["comment"].as_str().unwrap_or("");
            let current_duration = selected["attributes"]["duration"]
                .as_str()
                .unwrap_or("00:00:00");
            let current_review = selected["attributes"]["review"].as_bool().unwrap_or(false);
            let current_not_billable = selected["attributes"]["not-billable"]
                .as_bool()
                .unwrap_or(false);

            // Get new comment using dialoguer
            let comment = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Comment")
                .default(current_comment.to_string())
                .interact_text()?;

            // Get new duration using dialoguer
            let duration = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Duration (HH:MM:SS)")
                .default(current_duration.to_string())
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.matches(':').count() == 2 {
                        Ok(())
                    } else {
                        Err("Duration must be in HH:MM:SS format")
                    }
                })
                .interact_text()?;

            // Get review status
            let review = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Mark for review?")
                .default(current_review)
                .interact()?;

            // Get billable status
            let not_billable = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Mark as not billable?")
                .default(current_not_billable)
                .interact()?;

            (selection, comment, duration, review, not_billable)
        } else {
            // In non-interactive mode, just edit the first report with minimal changes
            if reports.len() > 1 {
                println!("Multiple reports found. Remove --non-interactive flag to select a specific report.");
                println!("Editing the first report.");
            }

            let selected = &reports[0];
            let current_comment = selected["attributes"]["comment"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let current_duration = selected["attributes"]["duration"]
                .as_str()
                .unwrap_or("00:00:00")
                .to_string();
            let current_review = selected["attributes"]["review"].as_bool().unwrap_or(false);
            let current_not_billable = selected["attributes"]["not-billable"]
                .as_bool()
                .unwrap_or(false);

            (
                0,
                current_comment,
                current_duration,
                current_review,
                current_not_billable,
            )
        };

        let selected = &reports[selection];
        let id = selected["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid report ID"))?;

        // Update report
        let report = serde_json::json!({
            "data": {
                "type": "reports",
                "id": id,
                "attributes": {
                    "comment": comment,
                    "duration": duration,
                    "review": review,
                    "not-billable": not_billable
                }
            }
        });

        client
            .patch::<_, serde_json::Value>(&format!("reports/{}", id), &report)
            .await?;

        println!("Report updated successfully");
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "No reports found for {}",
        date.format("%Y-%m-%d")
    ))
}

/// Interactive function to select task
async fn interactive_select_task(client: &TimedClient, show_archived: bool) -> Result<String> {
    use dialoguer::{theme::ColorfulTheme, FuzzySelect};

    // Get customers
    let mut filter = FilterParams::default();
    if !show_archived {
        filter
            .custom
            .insert("archived".to_string(), "0".to_string());
    }

    let customers_response = client
        .get::<serde_json::Value>("customers", Some(&filter))
        .await?;

    if let Some(customers) = customers_response["data"].as_array() {
        if customers.is_empty() {
            return Err(anyhow::anyhow!("No customers found"));
        }

        // Prepare customer options for selection
        let mut customer_names = Vec::new();
        for customer in customers.iter() {
            let name = customer["attributes"]["name"].as_str().unwrap_or("Unknown");
            customer_names.push(name.to_string());
        }

        // Interactive selection of customer
        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a customer")
            .items(&customer_names)
            .default(0)
            .interact()?;

        let customer_id = customers[selection]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid customer ID"))?;

        // Get projects for the selected customer
        filter
            .custom
            .insert("customer".to_string(), customer_id.to_string());

        let projects_response = client
            .get::<serde_json::Value>("projects", Some(&filter))
            .await?;

        if let Some(projects) = projects_response["data"].as_array() {
            if projects.is_empty() {
                return Err(anyhow::anyhow!("No projects found for this customer"));
            }

            // Prepare project options for selection
            let mut project_names = Vec::new();
            for project in projects.iter() {
                let name = project["attributes"]["name"].as_str().unwrap_or("Unknown");
                project_names.push(name.to_string());
            }

            // Interactive selection of project
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a project")
                .items(&project_names)
                .default(0)
                .interact()?;

            let project_id = projects[selection]["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid project ID"))?;

            // Get tasks for the selected project
            filter.custom.remove("customer");
            filter
                .custom
                .insert("project".to_string(), project_id.to_string());

            let tasks_response = client
                .get::<serde_json::Value>("tasks", Some(&filter))
                .await?;

            if let Some(tasks) = tasks_response["data"].as_array() {
                if tasks.is_empty() {
                    return Err(anyhow::anyhow!("No tasks found for this project"));
                }

                // Prepare task options for selection
                let mut task_names = Vec::new();
                for task in tasks.iter() {
                    let name = task["attributes"]["name"].as_str().unwrap_or("Unknown");
                    task_names.push(name.to_string());
                }

                // Interactive selection of task
                let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a task")
                    .items(&task_names)
                    .default(0)
                    .interact()?;

                let task_id = tasks[selection]["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid task ID"))?;

                return Ok(task_id.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Failed to select task"))
}

/// Get task ID from customer, project, and task names
async fn get_task_id(
    client: &TimedClient,
    customer_name: &str,
    project_name: &str,
    task_name: &str,
    show_archived: bool,
) -> Result<String> {
    // Get customer ID
    let mut filter = FilterParams::default();
    if !show_archived {
        filter
            .custom
            .insert("archived".to_string(), "0".to_string());
    }

    let customers_response = client
        .get::<serde_json::Value>("customers", Some(&filter))
        .await?;

    if let Some(customers) = customers_response["data"].as_array() {
        let customer = customers
            .iter()
            .find(|c| c["attributes"]["name"].as_str().unwrap_or("") == customer_name)
            .ok_or_else(|| anyhow::anyhow!("Customer not found: {}", customer_name))?;

        let customer_id = customer["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid customer ID"))?;

        // Get project ID
        filter
            .custom
            .insert("customer".to_string(), customer_id.to_string());

        let projects_response = client
            .get::<serde_json::Value>("projects", Some(&filter))
            .await?;

        if let Some(projects) = projects_response["data"].as_array() {
            let project = projects
                .iter()
                .find(|p| p["attributes"]["name"].as_str().unwrap_or("") == project_name)
                .ok_or_else(|| anyhow::anyhow!("Project not found: {}", project_name))?;

            let project_id = project["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid project ID"))?;

            // Get task ID
            filter.custom.remove("customer");
            filter
                .custom
                .insert("project".to_string(), project_id.to_string());

            let tasks_response = client
                .get::<serde_json::Value>("tasks", Some(&filter))
                .await?;

            if let Some(tasks) = tasks_response["data"].as_array() {
                let task = tasks
                    .iter()
                    .find(|t| t["attributes"]["name"].as_str().unwrap_or("") == task_name)
                    .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_name))?;

                let task_id = task["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid task ID"))?;

                return Ok(task_id.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Failed to get task ID"))
}
