use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::TimedClient;
use serde_json::Value;

/// Base trait for all API models
pub trait Model {
    /// Get the resource name for this model
    fn resource_name() -> &'static str;

    /// Get the endpoint URL for this model
    fn endpoint_url(client: &TimedClient) -> String {
        format!("{}{}", client.base_url(), Self::resource_name())
    }
}

/// Common parameters for API filtering
#[derive(Debug, Clone, Default, Serialize)]
pub struct FilterParams {
    /// Filter by specific date (field name depends on the endpoint: 'date' or 'day')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Filter by start date - inclusive (greater than or equal)
    #[serde(rename = "from_date", skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,

    /// Filter by end date - inclusive (less than or equal)
    #[serde(rename = "to_date", skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,

    /// Filter by archived status (0 = not archived, 1 = archived, null = both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<i32>,

    /// Filter by active status (for activities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<i32>,

    /// Filter by user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Filter by review status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<i32>,

    /// Filter by billable status
    #[serde(rename = "not_billable", skip_serializing_if = "Option::is_none")]
    pub not_billable: Option<i32>,

    /// Filter by task ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,

    /// Filter by project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,

    /// Filter by customer ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,

    /// Include related resources in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,

    /// Additional custom parameters that aren't covered by the common ones
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

/// API response wrapper for a single resource
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceResponse<T> {
    pub data: T,
    pub included: Option<Vec<IncludedResource>>,
}

/// API response wrapper for multiple resources
#[derive(Debug, Clone, Deserialize)]
pub struct ResourcesResponse<T> {
    pub data: Vec<T>,
    pub included: Option<Vec<IncludedResource>>,
}

/// A generic included resource in an API response
#[derive(Debug, Clone, Deserialize)]
pub struct IncludedResource {
    #[serde(rename = "type")]
    pub type_name: String,
    pub id: String,
    pub attributes: serde_json::Value,
    pub relationships: Option<HashMap<String, serde_json::Value>>,
}

/// User model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: UserAttributes,
    pub relationships: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAttributes {
    pub username: String,
    pub email: String,
    #[serde(rename = "first-name")]
    pub first_name: String,
    #[serde(rename = "last-name")]
    pub last_name: String,
}

impl Model for User {
    fn resource_name() -> &'static str {
        "users"
    }
}

/// Customer model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: CustomerAttributes,
    pub relationships: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAttributes {
    pub name: String,
    pub archived: bool,
}

impl Model for Customer {
    fn resource_name() -> &'static str {
        "customers"
    }
}

/// Project model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: ProjectAttributes,
    pub relationships: ProjectRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAttributes {
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRelationships {
    pub customer: Option<RelationshipData>,
}

impl Model for Project {
    fn resource_name() -> &'static str {
        "projects"
    }
}

/// Task model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: TaskAttributes,
    pub relationships: TaskRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttributes {
    pub name: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRelationships {
    pub project: Option<RelationshipData>,
}

impl Model for Task {
    fn resource_name() -> &'static str {
        "tasks"
    }
}

/// Activity model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: ActivityAttributes,
    pub relationships: ActivityRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityAttributes {
    pub comment: String,
    pub date: String,
    #[serde(rename = "from-time")]
    pub from_time: String,
    #[serde(rename = "to-time")]
    pub to_time: Option<String>,
    pub review: bool,
    #[serde(rename = "not-billable")]
    pub not_billable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRelationships {
    pub user: Option<RelationshipData>,
    pub task: Option<RelationshipData>,
}

impl Model for Activity {
    fn resource_name() -> &'static str {
        "activities"
    }
}

/// Report model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: ReportAttributes,
    pub relationships: ReportRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAttributes {
    pub comment: String,
    pub date: String,
    pub duration: String,
    pub review: bool,
    #[serde(rename = "not-billable")]
    pub not_billable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRelationships {
    pub user: Option<RelationshipData>,
    pub task: Option<RelationshipData>,
    #[serde(rename = "verified-by", skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<RelationshipData>,
}

impl Model for Report {
    fn resource_name() -> &'static str {
        "reports"
    }
}

/// WorktimeBalance model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktimeBalance {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: WorktimeBalanceAttributes,
    pub relationships: WorktimeBalanceRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktimeBalanceAttributes {
    pub date: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktimeBalanceRelationships {
    pub user: Option<RelationshipData>,
}

impl Model for WorktimeBalance {
    fn resource_name() -> &'static str {
        "worktime-balances"
    }
}

/// Common structure for relationship data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    pub data: Option<RelationshipResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipResource {
    #[serde(rename = "type")]
    pub type_name: String,
    pub id: String,
}

/// Attendance model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendance {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: AttendanceAttributes,
    pub relationships: AttendanceRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceAttributes {
    pub date: String,
    #[serde(rename = "from-time")]
    pub from_time: String,
    #[serde(rename = "to-time")]
    pub to_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRelationships {
    pub user: Option<RelationshipData>,
}

impl Model for Attendance {
    fn resource_name() -> &'static str {
        "attendances"
    }
}

/// Absence model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Absence {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: AbsenceAttributes,
    pub relationships: AbsenceRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsenceAttributes {
    pub date: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsenceRelationships {
    pub user: Option<RelationshipData>,
    #[serde(rename = "absence-type")]
    pub absence_type: Option<RelationshipData>,
}

impl Model for Absence {
    fn resource_name() -> &'static str {
        "absences"
    }
}

/// AbsenceType model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsenceType {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: AbsenceTypeAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsenceTypeAttributes {
    pub name: String,
    #[serde(rename = "fill-worktime")]
    pub fill_worktime: bool,
}

impl Model for AbsenceType {
    fn resource_name() -> &'static str {
        "absence-types"
    }
}

/// YearStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: YearStatisticAttributes,
    pub relationships: YearStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearStatisticAttributes {
    pub year: i32,
    pub duration: String,
    #[serde(rename = "total-attendance")]
    pub total_attendance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearStatisticRelationships {
    pub user: Option<RelationshipData>,
}

impl Model for YearStatistic {
    fn resource_name() -> &'static str {
        "year-statistics"
    }
}

/// MonthStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: MonthStatisticAttributes,
    pub relationships: MonthStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthStatisticAttributes {
    pub year: i32,
    pub month: i32,
    pub duration: String,
    #[serde(rename = "total-attendance")]
    pub total_attendance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthStatisticRelationships {
    pub user: Option<RelationshipData>,
}

impl Model for MonthStatistic {
    fn resource_name() -> &'static str {
        "month-statistics"
    }
}

/// TaskStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: TaskStatisticAttributes,
    pub relationships: TaskStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatisticAttributes {
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatisticRelationships {
    pub task: Option<RelationshipData>,
    pub user: Option<RelationshipData>,
}

impl Model for TaskStatistic {
    fn resource_name() -> &'static str {
        "task-statistics"
    }
}

/// UserStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: UserStatisticAttributes,
    pub relationships: UserStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatisticAttributes {
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatisticRelationships {
    pub user: Option<RelationshipData>,
}

impl Model for UserStatistic {
    fn resource_name() -> &'static str {
        "user-statistics"
    }
}

/// ProjectStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: ProjectStatisticAttributes,
    pub relationships: ProjectStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatisticAttributes {
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatisticRelationships {
    pub project: Option<RelationshipData>,
}

impl Model for ProjectStatistic {
    fn resource_name() -> &'static str {
        "project-statistics"
    }
}

/// CustomerStatistic model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerStatistic {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: CustomerStatisticAttributes,
    pub relationships: CustomerStatisticRelationships,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerStatisticAttributes {
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerStatisticRelationships {
    pub customer: Option<RelationshipData>,
}

impl Model for CustomerStatistic {
    fn resource_name() -> &'static str {
        "customer-statistics"
    }
}

/// WorkReport model for Timed API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkReport {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attributes: WorkReportAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkReportAttributes {
    pub data: Value,
}

impl Model for WorkReport {
    fn resource_name() -> &'static str {
        "work-reports"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_resource_names() {
        assert_eq!(User::resource_name(), "users");
        assert_eq!(Customer::resource_name(), "customers");
        assert_eq!(Project::resource_name(), "projects");
        assert_eq!(Task::resource_name(), "tasks");
        assert_eq!(Activity::resource_name(), "activities");
        assert_eq!(Report::resource_name(), "reports");
        assert_eq!(WorktimeBalance::resource_name(), "worktime-balances");
        assert_eq!(Attendance::resource_name(), "attendances");
        assert_eq!(Absence::resource_name(), "absences");
        assert_eq!(AbsenceType::resource_name(), "absence-types");
        assert_eq!(YearStatistic::resource_name(), "year-statistics");
        assert_eq!(MonthStatistic::resource_name(), "month-statistics");
        assert_eq!(TaskStatistic::resource_name(), "task-statistics");
        assert_eq!(UserStatistic::resource_name(), "user-statistics");
        assert_eq!(ProjectStatistic::resource_name(), "project-statistics");
        assert_eq!(CustomerStatistic::resource_name(), "customer-statistics");
        assert_eq!(WorkReport::resource_name(), "work-reports");
    }

    #[test]
    fn test_serialize_activity() {
        let activity = Activity {
            id: Some("123".to_string()),
            type_name: "activities".to_string(),
            attributes: ActivityAttributes {
                comment: "Working on something".to_string(),
                date: "2023-07-15".to_string(),
                from_time: "09:00:00".to_string(),
                to_time: Some("17:00:00".to_string()),
                review: false,
                not_billable: false,
            },
            relationships: ActivityRelationships {
                user: Some(RelationshipData {
                    data: Some(RelationshipResource {
                        type_name: "users".to_string(),
                        id: "456".to_string(),
                    }),
                }),
                task: Some(RelationshipData {
                    data: Some(RelationshipResource {
                        type_name: "tasks".to_string(),
                        id: "789".to_string(),
                    }),
                }),
            },
        };

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("activities"));
        assert!(json.contains("Working on something"));
    }
}
