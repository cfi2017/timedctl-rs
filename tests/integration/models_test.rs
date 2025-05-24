use libtimed::models::*;
use std::collections::HashMap;

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
fn test_attendance_serialization() {
    let attendance = Attendance {
        id: Some("123".to_string()),
        type_name: "attendances".to_string(),
        attributes: AttendanceAttributes {
            date: "2023-08-15".to_string(),
            from_time: "09:00:00".to_string(),
            to_time: Some("17:00:00".to_string()),
        },
        relationships: AttendanceRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: "456".to_string(),
                }),
            }),
        },
    };

    let json = serde_json::to_string(&attendance).unwrap();
    assert!(json.contains("attendances"));
    assert!(json.contains("2023-08-15"));
    assert!(json.contains("09:00:00"));
    assert!(json.contains("17:00:00"));
    assert!(json.contains("users"));
    assert!(json.contains("456"));

    // Test deserialization
    let deserialized: Attendance = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, attendance.id);
    assert_eq!(deserialized.attributes.date, attendance.attributes.date);
    assert_eq!(
        deserialized.attributes.from_time,
        attendance.attributes.from_time
    );
    assert_eq!(
        deserialized.attributes.to_time,
        attendance.attributes.to_time
    );
}

#[test]
fn test_absence_serialization() {
    let absence = Absence {
        id: Some("123".to_string()),
        type_name: "absences".to_string(),
        attributes: AbsenceAttributes {
            date: "2023-08-15".to_string(),
            comment: Some("Vacation".to_string()),
        },
        relationships: AbsenceRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: "456".to_string(),
                }),
            }),
            absence_type: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "absence-types".to_string(),
                    id: "789".to_string(),
                }),
            }),
        },
    };

    let json = serde_json::to_string(&absence).unwrap();
    assert!(json.contains("absences"));
    assert!(json.contains("2023-08-15"));
    assert!(json.contains("Vacation"));
    assert!(json.contains("users"));
    assert!(json.contains("456"));
    assert!(json.contains("absence-types"));
    assert!(json.contains("789"));

    // Test deserialization
    let deserialized: Absence = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, absence.id);
    assert_eq!(deserialized.attributes.date, absence.attributes.date);
    assert_eq!(deserialized.attributes.comment, absence.attributes.comment);
}

#[test]
fn test_statistics_models() {
    let year_statistic = YearStatistic {
        id: Some("123".to_string()),
        type_name: "year-statistics".to_string(),
        attributes: YearStatisticAttributes {
            year: 2023,
            duration: "2000:00:00".to_string(),
            total_attendance: "2100:00:00".to_string(),
        },
        relationships: YearStatisticRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: "456".to_string(),
                }),
            }),
        },
    };

    let json = serde_json::to_string(&year_statistic).unwrap();
    assert!(json.contains("year-statistics"));
    assert!(json.contains("2023"));
    assert!(json.contains("2000:00:00"));
    assert!(json.contains("2100:00:00"));

    // Test month statistics
    let month_statistic = MonthStatistic {
        id: Some("123".to_string()),
        type_name: "month-statistics".to_string(),
        attributes: MonthStatisticAttributes {
            year: 2023,
            month: 8,
            duration: "160:00:00".to_string(),
            total_attendance: "168:00:00".to_string(),
        },
        relationships: MonthStatisticRelationships {
            user: Some(RelationshipData {
                data: Some(RelationshipResource {
                    type_name: "users".to_string(),
                    id: "456".to_string(),
                }),
            }),
        },
    };

    let json = serde_json::to_string(&month_statistic).unwrap();
    assert!(json.contains("month-statistics"));
    assert!(json.contains("2023"));
    assert!(json.contains("8"));
    assert!(json.contains("160:00:00"));
}

#[test]
fn test_filter_params() {
    let filter = FilterParams {
        date: Some("2023-08-15".to_string()),
        from_date: Some("2023-08-01".to_string()),
        to_date: Some("2023-08-31".to_string()),
        archived: Some(0),
        active: Some(1),
        user: Some("123".to_string()),
        review: Some(0),
        not_billable: Some(0),
        task: Some("456".to_string()),
        project: Some("789".to_string()),
        customer: Some("101".to_string()),
        include: Some("user,task".to_string()),
        custom: {
            let mut map = HashMap::new();
            map.insert("custom_param".to_string(), "value".to_string());
            map
        },
    };

    let json = serde_json::to_value(&filter).unwrap();

    assert_eq!(json["date"], "2023-08-15");
    assert_eq!(json["from_date"], "2023-08-01");
    assert_eq!(json["to_date"], "2023-08-31");
    assert_eq!(json["archived"], 0);
    assert_eq!(json["active"], 1);
    assert_eq!(json["user"], "123");
    assert_eq!(json["review"], 0);
    assert_eq!(json["not_billable"], 0);
    assert_eq!(json["task"], "456");
    assert_eq!(json["project"], "789");
    assert_eq!(json["customer"], "101");
    assert_eq!(json["include"], "user,task");
    assert_eq!(json["custom_param"], "value");
}

#[test]
fn test_resources_response() {
    // Test activities response parsing
    let json_str = r#"{
        "data": [
            {
                "id": "123",
                "type": "activities",
                "attributes": {
                    "comment": "Working on something",
                    "date": "2023-07-15",
                    "from-time": "09:00:00",
                    "to-time": "17:00:00",
                    "review": false,
                    "not-billable": false
                },
                "relationships": {
                    "user": {
                        "data": {
                            "type": "users",
                            "id": "456"
                        }
                    },
                    "task": {
                        "data": {
                            "type": "tasks",
                            "id": "789"
                        }
                    }
                }
            }
        ],
        "included": [
            {
                "id": "456",
                "type": "users",
                "attributes": {
                    "username": "testuser",
                    "email": "test@example.com",
                    "first-name": "Test",
                    "last-name": "User"
                }
            },
            {
                "id": "789",
                "type": "tasks",
                "attributes": {
                    "name": "Test Task",
                    "archived": false
                },
                "relationships": {
                    "project": {
                        "data": {
                            "type": "projects",
                            "id": "101"
                        }
                    }
                }
            }
        ]
    }"#;

    let response: ResourcesResponse<Activity> = serde_json::from_str(json_str).unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, Some("123".to_string()));
    assert_eq!(response.data[0].attributes.comment, "Working on something");
    assert_eq!(response.included.as_ref().unwrap().len(), 2);
}
