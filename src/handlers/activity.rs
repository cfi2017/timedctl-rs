use anyhow::Result;
use chrono::{Local, NaiveTime, Timelike};
use tracing::{debug, info};

use libtimed::{models::FilterParams, TimedClient};

use super::parse_date;

/// Start a new activity
#[allow(clippy::too_many_arguments)]
pub async fn start_activity(
    client: &TimedClient,
    comment: &str,
    customer: Option<&str>,
    project: Option<&str>,
    task: Option<&str>,
    show_archived: bool,
    start_time: Option<&str>,
    interactive: bool,
) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};

    // If there's an active activity, stop it first
    stop_activity(client).await?;

    // Get task ID based on selection or parameters
    let task_id = if let (Some(customer_name), Some(project_name), Some(task_name)) =
        (customer, project, task)
    {
        // Get task ID directly from parameters
        get_task_id(
            client,
            customer_name,
            project_name,
            task_name,
            show_archived,
        )
        .await?
    } else if interactive {
        // Interactive selection
        interactive_select_task(client, show_archived).await?
    } else {
        // Non-interactive mode requires task information
        return Err(anyhow::anyhow!("Task information required. Provide --customer, --project, and --task parameters or remove --non-interactive flag"));
    };

    // Get comment if not provided
    let activity_comment = if comment.is_empty() {
        if interactive {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Activity description")
                .interact_text()?
        } else {
            return Err(anyhow::anyhow!("Comment is required in non-interactive mode. Provide a comment or remove --non-interactive flag"));
        }
    } else {
        comment.to_string()
    };

    // Parse start time if provided, or ask for it interactively
    let start = if let Some(time_str) = start_time {
        // Try various time formats
        if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
            time
        } else if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
            time
        } else {
            return Err(anyhow::anyhow!(
                "Invalid time format. Use HH:MM or HH:MM:SS"
            ));
        }
    } else if interactive {
        // Default to current time
        let now = Local::now().time();

        // Ask if user wants to use current time or specify a different time
        let options = vec!["Current time", "Specify start time"];
        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Start time")
            .default(0)
            .items(&options)
            .interact()?;

        if selection == 0 {
            now
        } else {
            // Parse user input time
            let time_input = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter start time (HH:MM)")
                .default(format!("{:02}:{:02}", now.hour(), now.minute()))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if NaiveTime::parse_from_str(input, "%H:%M").is_ok() {
                        Ok(())
                    } else {
                        Err("Please use HH:MM format")
                    }
                })
                .interact_text()?;

            NaiveTime::parse_from_str(&time_input, "%H:%M")?
        }
    } else {
        // In non-interactive mode, default to current time
        Local::now().time()
    };

    // Create activity
    let activity = serde_json::json!({
        "data": {
            "type": "activities",
            "attributes": {
                "comment": activity_comment,
                "date": Local::now().date_naive().format("%Y-%m-%d").to_string(),
                "from-time": format!("{:02}:{:02}:00", start.hour(), start.minute()),
                "to-time": null,
                "review": false,
                "not-billable": false
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

    client
        .post::<_, serde_json::Value>("activities", &activity)
        .await?;

    info!(
        "Activity started: {} at {:02}:{:02}",
        activity_comment,
        start.hour(),
        start.minute()
    );
    Ok(())
}

/// Stop the currently active activity
pub async fn stop_activity(client: &TimedClient) -> Result<()> {
    debug!("Checking for active activity");

    // Get current activity
    let mut filter = FilterParams::default();
    filter
        .custom
        .insert("active".to_string(), "true".to_string());

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if let Some(activity) = activities.first() {
            let id = activity["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid activity ID"))?;

            // Stop the activity by setting to-time
            let now = Local::now().time();
            let update = serde_json::json!({
                "data": {
                    "type": "activities",
                    "id": id,
                    "attributes": {
                        "to-time": format!("{:02}:{:02}:00", now.hour(), now.minute())
                    }
                }
            });

            client
                .patch::<_, serde_json::Value>(&format!("activities/{}", id), &update)
                .await?;
            info!("Activity stopped");
            return Ok(());
        }
    }

    debug!("No active activity found");
    Ok(())
}

/// Show information about activities within a date range
pub async fn show_activity(
    client: &TimedClient,
    _short: bool,
    date_str: Option<&str>,
    from_str: Option<&str>,
    to_str: Option<&str>,
    all_users: bool,
) -> Result<()> {
    // Create filter for activities
    let mut filter = FilterParams::default();

    // Handle date filtering with priority: specific date > date range > today
    if let Some(date) = date_str {
        // Specific date has highest priority - use 'day' parameter which is what the backend uses
        debug!("Getting activities for specific date: {}", date);
        filter.custom.insert("day".to_string(), date.to_string());
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
        debug!("Getting activities for today: {}", today);
        filter.custom.insert("day".to_string(), today);
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
    filter.include = Some("task,task.project,task.project.customer,user".to_string());

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if activities.is_empty() {
            if date_str.is_some() {
                println!("No activities found for date: {}", date_str.unwrap());
            } else if from_str.is_some() || to_str.is_some() {
                let from_msg = from_str.map_or("today", |d| d);
                let to_msg = to_str.map_or("today", |d| d);
                println!("No activities found from {} to {}", from_msg, to_msg);
            } else {
                println!("No activities found for today");
            }
            return Ok(());
        }

        // Show the activities as a list
        if date_str.is_some() {
            println!("Activities for {}", date_str.unwrap());
        } else if from_str.is_some() || to_str.is_some() {
            let from_msg = from_str.map_or("today", |d| d);
            let to_msg = to_str.map_or("today", |d| d);
            println!("Activities from {} to {}", from_msg, to_msg);
        } else {
            let today = Local::now().date_naive().format("%Y-%m-%d");
            println!("Activities for {}", today);
        }
        println!("----------------------------------------");

        for activity in activities {
            let comment = activity["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            let from_time = activity["attributes"]["from-time"]
                .as_str()
                .unwrap_or("00:00:00");
            let to_time = activity["attributes"]["to-time"]
                .as_str()
                .unwrap_or("(active)");

            // Get task info (and related project/customer) if available
            let mut task_name = "Unknown Task";
            let mut project_name = "Unknown Project";
            let mut customer_name = "Unknown Customer";
            let mut username = "Unknown User";

            if let Some(task_data) = activity["relationships"]["task"]["data"].as_object() {
                if let (Some(task_type), Some(task_id)) =
                    (task_data["type"].as_str(), task_data["id"].as_str())
                {
                    if task_type == "tasks" {
                        // Find included task
                        if let Some(included) = response["included"].as_array() {
                            if let Some(task) = included
                                .iter()
                                .find(|i| i["type"] == "tasks" && i["id"] == task_id)
                            {
                                task_name =
                                    task["attributes"]["name"].as_str().unwrap_or(task_name);

                                // Find project
                                if let Some(project_id) =
                                    task["relationships"]["project"]["data"]["id"].as_str()
                                {
                                    if let Some(project) = included
                                        .iter()
                                        .find(|i| i["type"] == "projects" && i["id"] == project_id)
                                    {
                                        project_name = project["attributes"]["name"]
                                            .as_str()
                                            .unwrap_or(project_name);

                                        // Find customer
                                        if let Some(customer_id) = project["relationships"]
                                            ["customer"]["data"]["id"]
                                            .as_str()
                                        {
                                            if let Some(customer) = included.iter().find(|i| {
                                                i["type"] == "customers" && i["id"] == customer_id
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
                    }
                }
            }

            // Get user info if all_users is true
            if all_users {
                if let Some(user_data) = activity["relationships"]["user"]["data"].as_object() {
                    if let (Some(user_type), Some(user_id)) =
                        (user_data["type"].as_str(), user_data["id"].as_str())
                    {
                        if user_type == "users" {
                            if let Some(included) = response["included"].as_array() {
                                if let Some(user) = included
                                    .iter()
                                    .find(|i| i["type"] == "users" && i["id"] == user_id)
                                {
                                    username =
                                        user["attributes"]["username"].as_str().unwrap_or(username);
                                }
                            }
                        }
                    }
                }
            }

            // Calculate duration if there's a to_time
            let duration_str = if to_time != "(active)" {
                let start = NaiveTime::parse_from_str(from_time, "%H:%M:%S").unwrap_or_default();
                let end = NaiveTime::parse_from_str(to_time, "%H:%M:%S").unwrap_or_default();
                let duration_mins = end.signed_duration_since(start).num_minutes();
                format!("{:.2}h", duration_mins as f64 / 60.0)
            } else {
                "active".to_string()
            };

            // Format output
            if all_users {
                println!(
                    "[{}] {} - {} - {}/{}/{} - {}",
                    username,
                    from_time,
                    duration_str,
                    customer_name,
                    project_name,
                    task_name,
                    comment
                );
            } else {
                println!(
                    "{} - {} - {}/{}/{} - {}",
                    from_time, duration_str, customer_name, project_name, task_name, comment
                );
            }
        }

        println!("----------------------------------------");
        println!("Total: {} activities", activities.len());

        return Ok(());
    }

    if date_str.is_some() {
        println!("No activities found for date: {}", date_str.unwrap());
    } else if from_str.is_some() || to_str.is_some() {
        let from_msg = from_str.map_or("today", |d| d);
        let to_msg = to_str.map_or("today", |d| d);
        println!("No activities found from {} to {}", from_msg, to_msg);
    } else {
        println!("No activities found for today");
    }

    Ok(())
}

/// Get the currently active activity
pub async fn get_active_activity(client: &TimedClient, short: bool) -> Result<()> {
    debug!("Getting active activity");

    // Create filter for active activities
    let mut filter = FilterParams::default();
    filter
        .custom
        .insert("active".to_string(), "true".to_string());
    filter.include = Some("task,task.project,task.project.customer,user".to_string());

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if activities.is_empty() {
            println!("No active activity found");
            return Ok(());
        }

        // If short flag is set, just show the comment
        if short {
            let activity = &activities[0];
            let comment = activity["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            println!("{}", comment);
            return Ok(());
        }

        // Get the first active activity
        let activity = &activities[0];
        let comment = activity["attributes"]["comment"]
            .as_str()
            .unwrap_or("No comment");
        let from_time = activity["attributes"]["from-time"]
            .as_str()
            .unwrap_or("00:00:00");

        // Find task/project/customer info
        if let Some(task_data) = activity["relationships"]["task"]["data"].as_object() {
            if let (Some(_task_type), Some(task_id)) =
                (task_data["type"].as_str(), task_data["id"].as_str())
            {
                if let Some(included) = response["included"].as_array() {
                    if let Some(task) = included
                        .iter()
                        .find(|i| i["type"] == "tasks" && i["id"] == task_id)
                    {
                        let task_name = task["attributes"]["name"]
                            .as_str()
                            .unwrap_or("Unknown Task");

                        if let Some(project_id) =
                            task["relationships"]["project"]["data"]["id"].as_str()
                        {
                            if let Some(project) = included
                                .iter()
                                .find(|i| i["type"] == "projects" && i["id"] == project_id)
                            {
                                let project_name = project["attributes"]["name"]
                                    .as_str()
                                    .unwrap_or("Unknown Project");

                                if let Some(customer_id) =
                                    project["relationships"]["customer"]["data"]["id"].as_str()
                                {
                                    if let Some(customer) = included.iter().find(|i| {
                                        i["type"] == "customers" && i["id"] == customer_id
                                    }) {
                                        let customer_name = customer["attributes"]["name"]
                                            .as_str()
                                            .unwrap_or("Unknown Customer");

                                        // Calculate elapsed time
                                        let start_time =
                                            NaiveTime::parse_from_str(from_time, "%H:%M:%S")
                                                .unwrap_or_default();
                                        let now = Local::now().time();
                                        let hours_elapsed =
                                            now.signed_duration_since(start_time).num_minutes()
                                                as f64
                                                / 60.0;

                                        println!("Active Activity");
                                        println!("----------------------------------------");
                                        println!("Activity: {}", comment);
                                        println!("Customer: {}", customer_name);
                                        println!("Project: {}", project_name);
                                        println!("Task: {}", task_name);
                                        println!("Started at: {}", from_time);
                                        println!("Elapsed time: {:.2} hours", hours_elapsed);

                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback if we couldn't get all the details
        println!("Active Activity: {}", comment);
        println!("Started at: {}", from_time);
    } else {
        println!("No active activity found");
    }

    Ok(())
}

/// Restart a previously tracked activity
pub async fn restart_activity(client: &TimedClient, date_str: Option<&str>) -> Result<()> {
    let date = parse_date(date_str)?;

    // Get activities for the date
    let filter = FilterParams {
        date: Some(date.format("%Y-%m-%d").to_string()),
        include: Some("task".to_string()),
        ..Default::default()
    };

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if activities.is_empty() {
            return Err(anyhow::anyhow!(
                "No activities found for the specified date"
            ));
        }

        // Interactive selection of activity to restart
        println!("Select an activity to restart:");
        for (i, activity) in activities.iter().enumerate() {
            let comment = activity["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            println!("{}: {}", i + 1, comment);
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let selection: usize = input.trim().parse()?;

        if selection < 1 || selection > activities.len() {
            return Err(anyhow::anyhow!("Invalid selection"));
        }

        let selected = &activities[selection - 1];
        let comment = selected["attributes"]["comment"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid activity comment"))?;

        let task_id = selected["relationships"]["task"]["data"]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid task reference"))?;

        // Stop current activity if any
        stop_activity(client).await?;

        // Create new activity with same task and comment
        let activity = serde_json::json!({
            "data": {
                "type": "activities",
                "attributes": {
                    "comment": comment,
                    "date": Local::now().date_naive().format("%Y-%m-%d").to_string(),
                    "from-time": format!("{:02}:{:02}:00", Local::now().hour(), Local::now().minute()),
                    "to-time": null,
                    "review": false,
                    "not-billable": false
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

        client
            .post::<_, serde_json::Value>("activities", &activity)
            .await?;

        info!("Activity restarted: {}", comment);
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "No activities found for the specified date"
    ))
}

/// Delete an activity
pub async fn delete_activity(client: &TimedClient, date_str: Option<&str>) -> Result<()> {
    let date = parse_date(date_str)?;

    // Get activities for the date
    let filter = FilterParams {
        date: Some(date.format("%Y-%m-%d").to_string()),
        ..Default::default()
    };

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if activities.is_empty() {
            return Err(anyhow::anyhow!(
                "No activities found for the specified date"
            ));
        }

        // Interactive selection of activity to delete
        println!("Select an activity to delete:");
        for (i, activity) in activities.iter().enumerate() {
            let comment = activity["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            println!("{}: {}", i + 1, comment);
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let selection: usize = input.trim().parse()?;

        if selection < 1 || selection > activities.len() {
            return Err(anyhow::anyhow!("Invalid selection"));
        }

        let selected = &activities[selection - 1];
        let id = selected["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid activity ID"))?;

        // Confirm deletion
        println!("Are you sure you want to delete this activity? (y/N)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            client.delete(&format!("activities/{}", id)).await?;
            info!("Activity deleted");
            return Ok(());
        } else {
            info!("Deletion cancelled");
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "No activities found for the specified date"
    ))
}

/// Generate a timesheet from the current activities
pub async fn generate_timesheet(client: &TimedClient) -> Result<()> {
    let today = Local::now().date_naive();

    // Get activities for today
    let filter = FilterParams {
        date: Some(today.format("%Y-%m-%d").to_string()),
        include: Some("task,task.project,task.project.customer".to_string()),
        ..Default::default()
    };

    let response = client
        .get::<serde_json::Value>("activities", Some(&filter))
        .await?;

    if let Some(activities) = response["data"].as_array() {
        if activities.is_empty() {
            return Err(anyhow::anyhow!("No activities found for today"));
        }

        println!("Timesheet for {}", today.format("%Y-%m-%d"));
        println!("----------------------------------------");

        let mut total_duration = 0.0;

        for activity in activities {
            let comment = activity["attributes"]["comment"]
                .as_str()
                .unwrap_or("No comment");
            let from_time = activity["attributes"]["from-time"]
                .as_str()
                .unwrap_or("00:00:00");
            let to_time = activity["attributes"]["to-time"].as_str();

            // Skip activities without an end time
            if to_time.is_none() {
                continue;
            }

            let to_time = to_time.unwrap();

            // Calculate duration
            let start = NaiveTime::parse_from_str(from_time, "%H:%M:%S")?;
            let end = NaiveTime::parse_from_str(to_time, "%H:%M:%S")?;

            let duration_minutes = end.signed_duration_since(start).num_minutes() as f64;
            let duration_hours = duration_minutes / 60.0;
            total_duration += duration_hours;

            // Get task/project/customer info
            let task_id = activity["relationships"]["task"]["data"]["id"]
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

            println!(
                "{:.2}h - {} / {} / {} - {}",
                duration_hours, customer_name, project_name, task_name, comment
            );
        }

        println!("----------------------------------------");
        println!("Total: {:.2} hours", total_duration);

        return Ok(());
    }

    Err(anyhow::anyhow!("No activities found for today"))
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
