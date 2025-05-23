use anyhow::Result;
use chrono::{Local, Datelike};
use tracing::debug;

use libtimed::{
    models::{
        CustomerStatistic, FilterParams, MonthStatistic, ProjectStatistic, 
        ResourceResponse, ResourcesResponse, TaskStatistic, UserStatistic, 
        WorkReport, YearStatistic
    },
    TimedClient,
};

/// Get year statistics for the current user or a specific user
pub async fn get_year_statistics(
    client: &TimedClient, 
    year: Option<i32>,
    user_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Use current year if not specified
    let year_value = year.unwrap_or_else(|| Local::now().year());
    filter.custom.insert("year".to_string(), year_value.to_string());
    
    // Add user filter if specified
    if let Some(id) = user_id {
        filter.user = Some(id.to_string());
    }
    
    // Include user data
    filter.include = Some("user".to_string());
    
    let response = client
        .get::<ResourcesResponse<YearStatistic>>("year-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No year statistics found for year {}", year_value);
        return Ok(());
    }
    
    println!("Year Statistics for {}", year_value);
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        let total_attendance = stat.attributes.total_attendance;
        
        // Get user info
        let mut user_name = "Current User".to_string();
        if let Some(user_rel) = &stat.relationships.user {
            if let Some(user_res) = &user_rel.data {
                let user_id = &user_res.id;
                
                // Find user in included data
                if let Some(user) = included.iter().find(|inc| {
                    inc.type_name == "users" && inc.id == *user_id
                }) {
                    if let Some(username) = user.attributes.get("username").and_then(|n| n.as_str()) {
                        user_name = username.to_string();
                    }
                }
            }
        }
        
        println!("User: {} | Total Time: {} | Total Attendance: {}", 
                 user_name, duration, total_attendance);
    }
    
    Ok(())
}

/// Get month statistics for the current user or a specific user
pub async fn get_month_statistics(
    client: &TimedClient, 
    year: Option<i32>,
    month: Option<i32>,
    user_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Use current year/month if not specified
    let now = Local::now();
    let year_value = year.unwrap_or_else(|| now.year());
    let month_value = month.unwrap_or_else(|| now.month() as i32);
    
    filter.custom.insert("year".to_string(), year_value.to_string());
    filter.custom.insert("month".to_string(), month_value.to_string());
    
    // Add user filter if specified
    if let Some(id) = user_id {
        filter.user = Some(id.to_string());
    }
    
    // Include user data
    filter.include = Some("user".to_string());
    
    let response = client
        .get::<ResourcesResponse<MonthStatistic>>("month-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No month statistics found for {}/{}", year_value, month_value);
        return Ok(());
    }
    
    println!("Month Statistics for {}/{}", year_value, month_value);
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        let total_attendance = stat.attributes.total_attendance;
        
        // Get user info
        let mut user_name = "Current User".to_string();
        if let Some(user_rel) = &stat.relationships.user {
            if let Some(user_res) = &user_rel.data {
                let user_id = &user_res.id;
                
                // Find user in included data
                if let Some(user) = included.iter().find(|inc| {
                    inc.type_name == "users" && inc.id == *user_id
                }) {
                    if let Some(username) = user.attributes.get("username").and_then(|n| n.as_str()) {
                        user_name = username.to_string();
                    }
                }
            }
        }
        
        println!("User: {} | Total Time: {} | Total Attendance: {}", 
                 user_name, duration, total_attendance);
    }
    
    Ok(())
}

/// Get task statistics for a user and date range
pub async fn get_task_statistics(
    client: &TimedClient, 
    from_date: Option<&str>,
    to_date: Option<&str>,
    user_id: Option<&str>,
    task_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Add date range if specified
    if let Some(from) = from_date {
        filter.from_date = Some(from.to_string());
    }
    
    if let Some(to) = to_date {
        filter.to_date = Some(to.to_string());
    }
    
    // Add user filter if specified
    if let Some(id) = user_id {
        filter.user = Some(id.to_string());
    }
    
    // Add task filter if specified
    if let Some(id) = task_id {
        filter.task = Some(id.to_string());
    }
    
    // Include task and user data
    filter.include = Some("task,task.project,task.project.customer,user".to_string());
    
    let response = client
        .get::<ResourcesResponse<TaskStatistic>>("task-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No task statistics found for the specified criteria");
        return Ok(());
    }
    
    println!("Task Statistics");
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        
        // Get task info
        let mut task_name = "Unknown Task".to_string();
        let mut project_name = "Unknown Project".to_string();
        let mut customer_name = "Unknown Customer".to_string();
        
        if let Some(task_rel) = &stat.relationships.task {
            if let Some(task_res) = &task_rel.data {
                let task_id = &task_res.id;
                
                // Find task in included data
                if let Some(task) = included.iter().find(|inc| {
                    inc.type_name == "tasks" && inc.id == *task_id
                }) {
                    if let Some(name) = task.attributes.get("name").and_then(|n| n.as_str()) {
                        task_name = name.to_string();
                    }
                    
                    // Find project
                    if let Some(project_id) = task.relationships.as_ref()
                        .and_then(|r| r.get("project"))
                        .and_then(|p| p.get("data"))
                        .and_then(|d| d.get("id"))
                        .and_then(|id| id.as_str()) {
                        
                        if let Some(project) = included.iter().find(|inc| {
                            inc.type_name == "projects" && inc.id == project_id
                        }) {
                            if let Some(name) = project.attributes.get("name").and_then(|n| n.as_str()) {
                                project_name = name.to_string();
                            }
                            
                            // Find customer
                            if let Some(customer_id) = project.relationships.as_ref()
                                .and_then(|r| r.get("customer"))
                                .and_then(|p| p.get("data"))
                                .and_then(|d| d.get("id"))
                                .and_then(|id| id.as_str()) {
                                
                                if let Some(customer) = included.iter().find(|inc| {
                                    inc.type_name == "customers" && inc.id == customer_id
                                }) {
                                    if let Some(name) = customer.attributes.get("name").and_then(|n| n.as_str()) {
                                        customer_name = name.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Get user info
        let mut user_name = "Unknown User".to_string();
        if let Some(user_rel) = &stat.relationships.user {
            if let Some(user_res) = &user_rel.data {
                let user_id = &user_res.id;
                
                // Find user in included data
                if let Some(user) = included.iter().find(|inc| {
                    inc.type_name == "users" && inc.id == *user_id
                }) {
                    if let Some(username) = user.attributes.get("username").and_then(|n| n.as_str()) {
                        user_name = username.to_string();
                    }
                }
            }
        }
        
        println!("User: {} | Customer: {} | Project: {} | Task: {} | Duration: {}", 
                 user_name, customer_name, project_name, task_name, duration);
    }
    
    Ok(())
}

/// Get user statistics
pub async fn get_user_statistics(
    client: &TimedClient, 
    from_date: Option<&str>,
    to_date: Option<&str>,
    user_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Add date range if specified
    if let Some(from) = from_date {
        filter.from_date = Some(from.to_string());
    }
    
    if let Some(to) = to_date {
        filter.to_date = Some(to.to_string());
    }
    
    // Add user filter if specified
    if let Some(id) = user_id {
        filter.user = Some(id.to_string());
    }
    
    // Include user data
    filter.include = Some("user".to_string());
    
    let response = client
        .get::<ResourcesResponse<UserStatistic>>("user-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No user statistics found for the specified criteria");
        return Ok(());
    }
    
    println!("User Statistics");
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        
        // Get user info
        let mut user_name = "Unknown User".to_string();
        if let Some(user_rel) = &stat.relationships.user {
            if let Some(user_res) = &user_rel.data {
                let user_id = &user_res.id;
                
                // Find user in included data
                if let Some(user) = included.iter().find(|inc| {
                    inc.type_name == "users" && inc.id == *user_id
                }) {
                    if let Some(username) = user.attributes.get("username").and_then(|n| n.as_str()) {
                        user_name = username.to_string();
                    }
                }
            }
        }
        
        println!("User: {} | Total Time: {}", user_name, duration);
    }
    
    Ok(())
}

/// Get customer statistics
pub async fn get_customer_statistics(
    client: &TimedClient, 
    from_date: Option<&str>,
    to_date: Option<&str>,
    customer_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Add date range if specified
    if let Some(from) = from_date {
        filter.from_date = Some(from.to_string());
    }
    
    if let Some(to) = to_date {
        filter.to_date = Some(to.to_string());
    }
    
    // Add customer filter if specified
    if let Some(id) = customer_id {
        filter.customer = Some(id.to_string());
    }
    
    // Include customer data
    filter.include = Some("customer".to_string());
    
    let response = client
        .get::<ResourcesResponse<CustomerStatistic>>("customer-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No customer statistics found for the specified criteria");
        return Ok(());
    }
    
    println!("Customer Statistics");
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        
        // Get customer info
        let mut customer_name = "Unknown Customer".to_string();
        if let Some(customer_rel) = &stat.relationships.customer {
            if let Some(customer_res) = &customer_rel.data {
                let customer_id = &customer_res.id;
                
                // Find customer in included data
                if let Some(customer) = included.iter().find(|inc| {
                    inc.type_name == "customers" && inc.id == *customer_id
                }) {
                    if let Some(name) = customer.attributes.get("name").and_then(|n| n.as_str()) {
                        customer_name = name.to_string();
                    }
                }
            }
        }
        
        println!("Customer: {} | Total Time: {}", customer_name, duration);
    }
    
    Ok(())
}

/// Get project statistics
pub async fn get_project_statistics(
    client: &TimedClient, 
    from_date: Option<&str>,
    to_date: Option<&str>,
    customer_id: Option<&str>,
    project_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Add date range if specified
    if let Some(from) = from_date {
        filter.from_date = Some(from.to_string());
    }
    
    if let Some(to) = to_date {
        filter.to_date = Some(to.to_string());
    }
    
    // Add customer filter if specified
    if let Some(id) = customer_id {
        filter.customer = Some(id.to_string());
    }
    
    // Add project filter if specified
    if let Some(id) = project_id {
        filter.project = Some(id.to_string());
    }
    
    // Include project and customer data
    filter.include = Some("project,project.customer".to_string());
    
    let response = client
        .get::<ResourcesResponse<ProjectStatistic>>("project-statistics", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No project statistics found for the specified criteria");
        return Ok(());
    }
    
    println!("Project Statistics");
    println!("----------------------------------------");
    
    // Get included data
    let included = response.included.unwrap_or_default();
    
    // Display each statistic
    for stat in response.data {
        let duration = stat.attributes.duration;
        
        // Get project info
        let mut project_name = "Unknown Project".to_string();
        let mut customer_name = "Unknown Customer".to_string();
        
        if let Some(project_rel) = &stat.relationships.project {
            if let Some(project_res) = &project_rel.data {
                let project_id = &project_res.id;
                
                // Find project in included data
                if let Some(project) = included.iter().find(|inc| {
                    inc.type_name == "projects" && inc.id == *project_id
                }) {
                    if let Some(name) = project.attributes.get("name").and_then(|n| n.as_str()) {
                        project_name = name.to_string();
                    }
                    
                    // Find customer
                    if let Some(customer_id) = project.relationships.as_ref()
                        .and_then(|r| r.get("customer"))
                        .and_then(|p| p.get("data"))
                        .and_then(|d| d.get("id"))
                        .and_then(|id| id.as_str()) {
                        
                        if let Some(customer) = included.iter().find(|inc| {
                            inc.type_name == "customers" && inc.id == customer_id
                        }) {
                            if let Some(name) = customer.attributes.get("name").and_then(|n| n.as_str()) {
                                customer_name = name.to_string();
                            }
                        }
                    }
                }
            }
        }
        
        println!("Customer: {} | Project: {} | Total Time: {}", 
                 customer_name, project_name, duration);
    }
    
    Ok(())
}

/// Get work report
pub async fn get_work_report(
    client: &TimedClient, 
    from_date: &str,
    to_date: &str,
    user_id: Option<&str>,
) -> Result<()> {
    let mut filter = FilterParams::default();
    
    // Add required date range
    filter.from_date = Some(from_date.to_string());
    filter.to_date = Some(to_date.to_string());
    
    // Add user filter if specified
    if let Some(id) = user_id {
        filter.user = Some(id.to_string());
    }
    
    let response = client
        .get::<ResourcesResponse<WorkReport>>("work-reports", Some(&filter))
        .await?;
    
    if response.data.is_empty() {
        println!("No work report data found for the specified date range");
        return Ok(());
    }
    
    println!("Work Report from {} to {}", from_date, to_date);
    println!("----------------------------------------");
    
    // Display report data
    for report in response.data {
        // Work report data is a complex JSON structure
        // Here we just print it as formatted JSON
        let data_str = serde_json::to_string_pretty(&report.attributes.data)?;
        println!("{}", data_str);
    }
    
    Ok(())
}