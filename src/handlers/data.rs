use anyhow::Result;
use tracing::debug;

use libtimed::{models::FilterParams, TimedClient};

/// Get customers data
pub async fn get_customers(client: &TimedClient, output_format: &str) -> Result<()> {
    debug!("Getting customers data");

    let filter = FilterParams::default();

    let response = client
        .get::<serde_json::Value>("customers", Some(&filter))
        .await?;

    if let Some(customers) = response["data"].as_array() {
        match output_format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&customers)?);
            }
            "csv" => {
                println!("id,name,archived");
                for customer in customers {
                    let id = customer["id"].as_str().unwrap_or("");
                    let name = customer["attributes"]["name"].as_str().unwrap_or("");
                    let archived = customer["attributes"]["archived"]
                        .as_bool()
                        .unwrap_or(false);
                    println!("{id},{name},{archived}");
                }
            }
            "text" => {
                println!("Customers:");
                for customer in customers {
                    let id = customer["id"].as_str().unwrap_or("");
                    let name = customer["attributes"]["name"].as_str().unwrap_or("");
                    let archived = customer["attributes"]["archived"]
                        .as_bool()
                        .unwrap_or(false);
                    println!("{id}: {name} (archived: {archived})");
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid output format: {}", output_format));
            }
        }

        return Ok(());
    }

    Err(anyhow::anyhow!("No customers found"))
}

/// Get projects data
pub async fn get_projects(
    client: &TimedClient,
    customer_id: Option<i32>,
    customer_name: Option<&str>,
    archived: bool,
    output_format: &str,
) -> Result<()> {
    debug!("Getting projects data");

    let mut filter = FilterParams::default();

    // Handle archived flag
    if !archived {
        filter
            .custom
            .insert("archived".to_string(), "0".to_string());
    }

    // Handle customer selection
    if let Some(id) = customer_id {
        filter.custom.insert("customer".to_string(), id.to_string());
    } else if let Some(name) = customer_name {
        // Find customer ID by name
        let customer_response = client
            .get::<serde_json::Value>("customers", Some(&FilterParams::default()))
            .await?;

        if let Some(customers) = customer_response["data"].as_array() {
            let customer = customers
                .iter()
                .find(|c| c["attributes"]["name"].as_str().unwrap_or("") == name)
                .ok_or_else(|| anyhow::anyhow!("Customer not found: {}", name))?;

            let id = customer["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid customer ID"))?;

            filter.custom.insert("customer".to_string(), id.to_string());
        }
    }

    let response = client
        .get::<serde_json::Value>("projects", Some(&filter))
        .await?;

    if let Some(projects) = response["data"].as_array() {
        match output_format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            }
            "csv" => {
                println!("id,name,customer_id,archived");
                for project in projects {
                    let id = project["id"].as_str().unwrap_or("");
                    let name = project["attributes"]["name"].as_str().unwrap_or("");
                    let archived = project["attributes"]["archived"].as_bool().unwrap_or(false);
                    let customer_id = project["relationships"]["customer"]["data"]["id"]
                        .as_str()
                        .unwrap_or("");
                    println!("{id},{name},{customer_id},{archived}");
                }
            }
            "text" => {
                println!("Projects:");
                for project in projects {
                    let id = project["id"].as_str().unwrap_or("");
                    let name = project["attributes"]["name"].as_str().unwrap_or("");
                    let archived = project["attributes"]["archived"].as_bool().unwrap_or(false);
                    let customer_id = project["relationships"]["customer"]["data"]["id"]
                        .as_str()
                        .unwrap_or("");
                    println!("{id}: {name} (customer: {customer_id}, archived: {archived})");
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid output format: {}", output_format));
            }
        }

        return Ok(());
    }

    Err(anyhow::anyhow!("No projects found"))
}

/// Get tasks data
pub async fn get_tasks(
    client: &TimedClient,
    customer_id: Option<i32>,
    customer_name: Option<&str>,
    project_id: Option<i32>,
    project_name: Option<&str>,
    archived: bool,
    output_format: &str,
) -> Result<()> {
    debug!("Getting tasks data");

    let mut filter = FilterParams::default();

    // Handle archived flag
    if !archived {
        filter
            .custom
            .insert("archived".to_string(), "0".to_string());
    }

    // Handle project selection
    if let Some(id) = project_id {
        filter.custom.insert("project".to_string(), id.to_string());
    } else if let Some(name) = project_name {
        // Find project ID by name
        let mut project_filter = FilterParams::default();

        // If customer is specified, use it to filter projects
        if let Some(c_id) = customer_id {
            project_filter
                .custom
                .insert("customer".to_string(), c_id.to_string());
        } else if let Some(c_name) = customer_name {
            // Find customer ID by name
            let customer_response = client
                .get::<serde_json::Value>("customers", Some(&FilterParams::default()))
                .await?;

            if let Some(customers) = customer_response["data"].as_array() {
                let customer = customers
                    .iter()
                    .find(|c| c["attributes"]["name"].as_str().unwrap_or("") == c_name)
                    .ok_or_else(|| anyhow::anyhow!("Customer not found: {}", c_name))?;

                let id = customer["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid customer ID"))?;

                project_filter
                    .custom
                    .insert("customer".to_string(), id.to_string());
            }
        }

        let project_response = client
            .get::<serde_json::Value>("projects", Some(&project_filter))
            .await?;

        if let Some(projects) = project_response["data"].as_array() {
            let project = projects
                .iter()
                .find(|p| p["attributes"]["name"].as_str().unwrap_or("") == name)
                .ok_or_else(|| anyhow::anyhow!("Project not found: {}", name))?;

            let id = project["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid project ID"))?;

            filter.custom.insert("project".to_string(), id.to_string());
        }
    } else if let Some(c_id) = customer_id {
        // If only customer ID is specified, find all projects for this customer
        let mut project_filter = FilterParams::default();
        project_filter
            .custom
            .insert("customer".to_string(), c_id.to_string());

        let project_response = client
            .get::<serde_json::Value>("projects", Some(&project_filter))
            .await?;

        if let Some(projects) = project_response["data"].as_array() {
            if !projects.is_empty() {
                let project_id = projects[0]["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid project ID"))?;

                filter
                    .custom
                    .insert("project".to_string(), project_id.to_string());
            }
        }
    } else if let Some(c_name) = customer_name {
        // Find customer ID by name
        let customer_response = client
            .get::<serde_json::Value>("customers", Some(&FilterParams::default()))
            .await?;

        if let Some(customers) = customer_response["data"].as_array() {
            let customer = customers
                .iter()
                .find(|c| c["attributes"]["name"].as_str().unwrap_or("") == c_name)
                .ok_or_else(|| anyhow::anyhow!("Customer not found: {}", c_name))?;

            let customer_id = customer["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid customer ID"))?;

            let mut project_filter = FilterParams::default();
            project_filter
                .custom
                .insert("customer".to_string(), customer_id.to_string());

            let project_response = client
                .get::<serde_json::Value>("projects", Some(&project_filter))
                .await?;

            if let Some(projects) = project_response["data"].as_array() {
                if !projects.is_empty() {
                    let project_id = projects[0]["id"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Invalid project ID"))?;

                    filter
                        .custom
                        .insert("project".to_string(), project_id.to_string());
                }
            }
        }
    }

    let response = client
        .get::<serde_json::Value>("tasks", Some(&filter))
        .await?;

    if let Some(tasks) = response["data"].as_array() {
        match output_format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&tasks)?);
            }
            "csv" => {
                println!("id,name,project_id,archived");
                for task in tasks {
                    let id = task["id"].as_str().unwrap_or("");
                    let name = task["attributes"]["name"].as_str().unwrap_or("");
                    let archived = task["attributes"]["archived"].as_bool().unwrap_or(false);
                    let project_id = task["relationships"]["project"]["data"]["id"]
                        .as_str()
                        .unwrap_or("");
                    println!("{id},{name},{project_id},{archived}");
                }
            }
            "text" => {
                println!("Tasks:");
                for task in tasks {
                    let id = task["id"].as_str().unwrap_or("");
                    let name = task["attributes"]["name"].as_str().unwrap_or("");
                    let archived = task["attributes"]["archived"].as_bool().unwrap_or(false);
                    let project_id = task["relationships"]["project"]["data"]["id"]
                        .as_str()
                        .unwrap_or("");
                    println!("{id}: {name} (project: {project_id}, archived: {archived})");
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid output format: {}", output_format));
            }
        }

        return Ok(());
    }

    Err(anyhow::anyhow!("No tasks found"))
}
