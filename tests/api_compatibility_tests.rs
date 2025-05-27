use libtimed::models::*;
use serde_json::json;

/// Integration test to ensure our models maintain compatibility with the old Django API
/// This test should be run in CI to catch any accidental field removals
#[test]
fn test_api_model_compatibility_integration() {
    // Test Activity model compatibility
    let activity_response = json!({
        "data": {
            "type": "activities",
            "id": "1",
            "attributes": {
                "comment": "Working on implementation",
                "date": "2023-08-15",
                "from-time": "09:00:00",
                "to-time": "17:00:00",
                "review": false,
                "not-billable": false
            },
            "relationships": {
                "user": {
                    "data": {
                        "type": "users",
                        "id": "123"
                    }
                },
                "task": {
                    "data": {
                        "type": "tasks",
                        "id": "456"
                    }
                }
            }
        }
    });

    let activity: Activity = serde_json::from_value(activity_response["data"].clone())
        .expect("Activity model should deserialize successfully");

    assert_eq!(activity.type_name, "activities");
    assert_eq!(activity.attributes.date, "2023-08-15");

    // Test Attendance model compatibility
    let attendance_response = json!({
        "data": {
            "type": "attendances",
            "id": "1",
            "attributes": {
                "date": "2023-08-15",
                "from-time": "09:00:00",
                "to-time": "17:00:00"
            },
            "relationships": {
                "user": {
                    "data": {
                        "type": "users",
                        "id": "123"
                    }
                }
            }
        }
    });

    let attendance: Attendance = serde_json::from_value(attendance_response["data"].clone())
        .expect("Attendance model should deserialize successfully");

    assert_eq!(attendance.type_name, "attendances");
    assert_eq!(attendance.attributes.date, "2023-08-15");

    // Test Report model basic compatibility (without missing fields)
    let report_response = json!({
        "data": {
            "type": "reports",
            "id": "1",
            "attributes": {
                "comment": "Development work",
                "date": "2023-08-15",
                "duration": "08:00:00",
                "review": false,
                "not-billable": false,
                "verified": true,
                "billed": true,
                "rejected": false
            },
            "relationships": {
                "user": {
                    "data": {
                        "type": "users",
                        "id": "123"
                    }
                },
                "task": {
                    "data": {
                        "type": "tasks",
                        "id": "456"
                    }
                }
            }
        }
    });

    let report: Report = serde_json::from_value(report_response["data"].clone())
        .expect("Report model should deserialize successfully");

    assert_eq!(report.type_name, "reports");
    assert_eq!(report.attributes.date, "2023-08-15");

    // Test Customer model basic compatibility (without missing fields)
    let customer_response = json!({
        "data": {
            "type": "customers",
            "id": "1",
            "attributes": {
                "name": "ACME Corp",
                "archived": false
            },
            "relationships": {}
        }
    });

    let customer: Customer = serde_json::from_value(customer_response["data"].clone())
        .expect("Customer model should deserialize successfully");

    assert_eq!(customer.type_name, "customers");
    assert_eq!(customer.attributes.name, "ACME Corp");

    // Test Project model basic compatibility (without missing fields)
    let project_response = json!({
        "data": {
            "type": "projects",
            "id": "1",
            "attributes": {
                "name": "Project Alpha",
                "archived": false
            },
            "relationships": {
                "customer": {
                    "data": {
                        "type": "customers",
                        "id": "123"
                    }
                }
            }
        }
    });

    let project: Project = serde_json::from_value(project_response["data"].clone())
        .expect("Project model should deserialize successfully");

    assert_eq!(project.type_name, "projects");
    assert_eq!(project.attributes.name, "Project Alpha");

    // Test Task model basic compatibility (without missing fields)
    let task_response = json!({
        "data": {
            "type": "tasks",
            "id": "1",
            "attributes": {
                "name": "Implementation",
                "archived": false
            },
            "relationships": {
                "project": {
                    "data": {
                        "type": "projects",
                        "id": "123"
                    }
                }
            }
        }
    });

    let task: Task = serde_json::from_value(task_response["data"].clone())
        .expect("Task model should deserialize successfully");

    assert_eq!(task.type_name, "tasks");
    assert_eq!(task.attributes.name, "Implementation");

    // Test User model basic compatibility (without missing fields)
    let user_response = json!({
        "data": {
            "type": "users",
            "id": "1",
            "attributes": {
                "username": "jdoe",
                "email": "john@example.com",
                "first-name": "John",
                "last-name": "Doe"
            },
            "relationships": {}
        }
    });

    let user: User = serde_json::from_value(user_response["data"].clone())
        .expect("User model should deserialize successfully");

    assert_eq!(user.type_name, "users");
    assert_eq!(user.attributes.username, "jdoe");

    // Test Absence model compatibility
    let absence_response = json!({
        "data": {
            "type": "absences",
            "id": "1",
            "attributes": {
                "date": "2023-08-15",
                "comment": "Vacation"
            },
            "relationships": {
                "user": {
                    "data": {
                        "type": "users",
                        "id": "123"
                    }
                },
                "absence-type": {
                    "data": {
                        "type": "absence-types",
                        "id": "456"
                    }
                }
            }
        }
    });

    let absence: Absence = serde_json::from_value(absence_response["data"].clone())
        .expect("Absence model should deserialize successfully");

    assert_eq!(absence.type_name, "absences");
    assert_eq!(absence.attributes.date, "2023-08-15");
}

/// Test to ensure JSON:API collection responses work correctly
#[test]
fn test_collection_response_compatibility() {
    let collection_response = json!({
        "data": [
            {
                "type": "activities",
                "id": "1",
                "attributes": {
                    "comment": "First activity",
                    "date": "2023-08-15",
                    "from-time": "09:00:00",
                    "to-time": "12:00:00",
                    "review": false,
                    "not-billable": false
                },
                "relationships": {}
            },
            {
                "type": "activities",
                "id": "2",
                "attributes": {
                    "comment": "Second activity",
                    "date": "2023-08-15",
                    "from-time": "13:00:00",
                    "to-time": "17:00:00",
                    "review": false,
                    "not-billable": false
                },
                "relationships": {}
            }
        ],
        "included": []
    });

    let activities: Vec<Activity> = serde_json::from_value(collection_response["data"].clone())
        .expect("Activity collection should deserialize successfully");

    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].id, Some("1".to_string()));
    assert_eq!(activities[1].id, Some("2".to_string()));
    assert_eq!(activities[0].attributes.comment, "First activity");
    assert_eq!(activities[1].attributes.comment, "Second activity");
}

/// Test field name consistency with Django API (kebab-case vs snake_case)
#[test]
fn test_field_naming_consistency() {
    // Test that our models use the correct field naming convention (kebab-case)
    // that matches the Django JSON:API output

    let activity = Activity {
        id: Some("1".to_string()),
        type_name: "activities".to_string(),
        attributes: ActivityAttributes {
            comment: "Test".to_string(),
            date: "2023-08-15".to_string(),
            from_time: "09:00:00".to_string(),
            to_time: Some("17:00:00".to_string()),
            review: false,
            not_billable: false,
        },
        relationships: ActivityRelationships {
            user: None,
            task: None,
        },
    };

    let json = serde_json::to_value(&activity).unwrap();

    // Verify that serialization uses kebab-case field names
    assert!(json["attributes"]["from-time"].is_string());
    assert!(json["attributes"]["to-time"].is_string());
    assert!(json["attributes"]["not-billable"].is_boolean());

    // Verify that snake_case fields don't exist
    assert!(json["attributes"]["from_time"].is_null());
    assert!(json["attributes"]["to_time"].is_null());
    assert!(json["attributes"]["not_billable"].is_null());
}

/// Test that model resource names match Django API endpoints
#[test]
fn test_resource_name_consistency() {
    let expected_mappings = [
        ("users", User::resource_name()),
        ("customers", Customer::resource_name()),
        ("projects", Project::resource_name()),
        ("tasks", Task::resource_name()),
        ("activities", Activity::resource_name()),
        ("reports", Report::resource_name()),
        ("attendances", Attendance::resource_name()),
        ("absences", Absence::resource_name()),
        ("absence-types", AbsenceType::resource_name()),
        ("worktime-balances", WorktimeBalance::resource_name()),
        ("year-statistics", YearStatistic::resource_name()),
        ("month-statistics", MonthStatistic::resource_name()),
        ("task-statistics", TaskStatistic::resource_name()),
        ("user-statistics", UserStatistic::resource_name()),
        ("project-statistics", ProjectStatistic::resource_name()),
        ("customer-statistics", CustomerStatistic::resource_name()),
        ("work-reports", WorkReport::resource_name()),
    ];

    for (expected, actual) in expected_mappings {
        assert_eq!(
            expected, actual,
            "Resource name mismatch: expected '{}', got '{}'",
            expected, actual
        );
    }
}
