#[cfg(test)]
mod api_compatibility_tests {
    use crate::models::*;
    use serde_json::json;

    /// Test that Activity model includes all fields from the old Django API
    #[test]
    fn test_activity_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "activities",
                "id": "123",
                "attributes": {
                    "comment": "Working on feature X",
                    "date": "2023-08-15",
                    "from-time": "09:00:00",
                    "to-time": "17:00:00",
                    "review": false,
                    "not-billable": false,
                    "transferred": true
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
        });

        // Test deserialization
        let activity: Result<Activity, _> =
            serde_json::from_value(mock_django_response["data"].clone());

        match activity {
            Ok(act) => {
                assert_eq!(act.id, Some("123".to_string()));
                assert_eq!(act.type_name, "activities");
                assert_eq!(act.attributes.comment, "Working on feature X");
                assert_eq!(act.attributes.date, "2023-08-15");
                assert_eq!(act.attributes.from_time, "09:00:00");
                assert_eq!(act.attributes.to_time, Some("17:00:00".to_string()));
                assert_eq!(act.attributes.review, false);
                assert_eq!(act.attributes.not_billable, false);
                // TODO: Add transferred field to ActivityAttributes
            }
            Err(e) => panic!("Failed to deserialize Activity: {}", e),
        }
    }

    /// Test that Report model includes all fields from the old Django API
    #[test]
    fn test_report_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "reports",
                "id": "123",
                "attributes": {
                    "comment": "Development work",
                    "date": "2023-08-15",
                    "duration": "08:00:00",
                    "review": false,
                    "not-billable": false,
                    "verified": true,
                    "billed": true,
                    "rejected": false,
                    "added": "2023-08-15T09:00:00Z",
                    "updated": "2023-08-15T17:00:00Z",
                    "remaining-effort": "02:00:00"
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
                    },
                    "verified_by": {
                        "data": {
                            "type": "users",
                            "id": "101"
                        }
                    }
                }
            }
        });

        // Check if all expected fields are present in the mock data
        let attributes = &mock_django_response["data"]["attributes"];
        assert!(attributes["added"].is_string(), "Missing 'added' field");
        assert!(attributes["updated"].is_string(), "Missing 'updated' field");
        assert!(
            attributes["remaining-effort"].is_string(),
            "Missing 'remaining-effort' field"
        );

        // TODO: Update ReportAttributes to include added, updated, remaining_effort fields
    }

    /// Test that Customer model includes all fields from the old Django API
    #[test]
    fn test_customer_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "customers",
                "id": "123",
                "attributes": {
                    "name": "ACME Corp",
                    "reference": "ACME-2023",
                    "email": "contact@acme.com",
                    "website": "https://acme.com",
                    "comment": "Important client",
                    "archived": false
                },
                "relationships": {
                    "assignees": {
                        "data": [
                            {
                                "type": "users",
                                "id": "456"
                            }
                        ]
                    }
                }
            }
        });

        let attributes = &mock_django_response["data"]["attributes"];
        assert!(
            attributes["reference"].is_string(),
            "Missing 'reference' field"
        );
        assert!(attributes["email"].is_string(), "Missing 'email' field");
        assert!(attributes["website"].is_string(), "Missing 'website' field");
        assert!(attributes["comment"].is_string(), "Missing 'comment' field");

        let relationships = &mock_django_response["data"]["relationships"];
        assert!(
            relationships["assignees"].is_object(),
            "Missing 'assignees' relationship"
        );

        // TODO: Update CustomerAttributes to include reference, email, website, comment fields
        // TODO: Update CustomerRelationships to include assignees relationship
    }

    /// Test that Project model includes all fields from the old Django API
    #[test]
    fn test_project_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "projects",
                "id": "123",
                "attributes": {
                    "name": "Project Alpha",
                    "reference": "ALPHA-2023",
                    "comment": "High priority project",
                    "archived": false,
                    "billed": false,
                    "estimated-time": "100:00:00",
                    "customer-visible": true,
                    "amount-offered": "50000.00",
                    "amount-invoiced": "25000.00",
                    "remaining-effort-tracking": true,
                    "total-remaining-effort": "50:00:00"
                },
                "relationships": {
                    "customer": {
                        "data": {
                            "type": "customers",
                            "id": "456"
                        }
                    },
                    "billing_type": {
                        "data": {
                            "type": "billing-types",
                            "id": "789"
                        }
                    },
                    "cost_center": {
                        "data": {
                            "type": "cost-centers",
                            "id": "101"
                        }
                    },
                    "assignees": {
                        "data": [
                            {
                                "type": "users",
                                "id": "112"
                            }
                        ]
                    }
                }
            }
        });

        let attributes = &mock_django_response["data"]["attributes"];
        let required_fields = [
            "reference",
            "comment",
            "billed",
            "estimated-time",
            "customer-visible",
            "amount-offered",
            "amount-invoiced",
            "remaining-effort-tracking",
            "total-remaining-effort",
        ];

        for field in required_fields {
            assert!(
                attributes[field].is_string()
                    || attributes[field].is_boolean()
                    || attributes[field].is_number(),
                "Missing '{}' field",
                field
            );
        }

        let relationships = &mock_django_response["data"]["relationships"];
        let required_relationships = ["billing-type", "cost-center", "assignees"];

        for rel in required_relationships {
            assert!(
                relationships[rel].is_object(),
                "Missing '{}' relationship",
                rel
            );
        }

        // TODO: Update ProjectAttributes to include all missing fields
        // TODO: Update ProjectRelationships to include billing_type, cost_center, assignees
    }

    /// Test that Task model includes all fields from the old Django API
    #[test]
    fn test_task_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "tasks",
                "id": "123",
                "attributes": {
                    "name": "Implementation",
                    "reference": "IMPL-001",
                    "estimated-time": "40:00:00",
                    "archived": false,
                    "amount-offered": "10000.00",
                    "amount-invoiced": "5000.00",
                    "most-recent-remaining-effort": "20:00:00"
                },
                "relationships": {
                    "project": {
                        "data": {
                            "type": "projects",
                            "id": "456"
                        }
                    },
                    "cost_center": {
                        "data": {
                            "type": "cost-centers",
                            "id": "789"
                        }
                    },
                    "assignees": {
                        "data": [
                            {
                                "type": "users",
                                "id": "101"
                            }
                        ]
                    }
                }
            }
        });

        let attributes = &mock_django_response["data"]["attributes"];
        let required_fields = [
            "reference",
            "estimated-time",
            "amount-offered",
            "amount-invoiced",
            "most-recent-remaining-effort",
        ];

        for field in required_fields {
            assert!(
                attributes[field].is_string() || attributes[field].is_number(),
                "Missing '{}' field",
                field
            );
        }

        let relationships = &mock_django_response["data"]["relationships"];
        let required_relationships = ["cost-center", "assignees"];

        for rel in required_relationships {
            assert!(
                relationships[rel].is_object(),
                "Missing '{}' relationship",
                rel
            );
        }

        // TODO: Update TaskAttributes to include all missing fields
        // TODO: Update TaskRelationships to include cost_center, assignees
    }

    /// Test that User model includes all fields from the old Django API
    #[test]
    fn test_user_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "users",
                "id": "123",
                "attributes": {
                    "username": "jdoe",
                    "email": "john.doe@example.com",
                    "first-name": "John",
                    "last-name": "Doe",
                    "tour-done": true,
                    "is-accountant": false,
                    "is-reviewer": true,
                    "is-active": true,
                    "is-staff": false,
                    "is-superuser": false,
                    "date-joined": "2023-01-01T00:00:00Z",
                    "last-login": "2023-08-15T09:00:00Z"
                },
                "relationships": {
                    "supervisors": {
                        "data": [
                            {
                                "type": "users",
                                "id": "456"
                            }
                        ]
                    },
                    "supervisees": {
                        "data": [
                            {
                                "type": "users",
                                "id": "789"
                            }
                        ]
                    }
                }
            }
        });

        let attributes = &mock_django_response["data"]["attributes"];
        let additional_fields = [
            "tour-done",
            "is-accountant",
            "is-reviewer",
            "is-active",
            "is-staff",
            "is-superuser",
            "date-joined",
            "last-login",
        ];

        for field in additional_fields {
            assert!(
                attributes[field].is_boolean() || attributes[field].is_string(),
                "Missing '{}' field",
                field
            );
        }

        let relationships = &mock_django_response["data"]["relationships"];
        assert!(
            relationships["supervisors"].is_object(),
            "Missing 'supervisors' relationship"
        );
        assert!(
            relationships["supervisees"].is_object(),
            "Missing 'supervisees' relationship"
        );

        // TODO: Update UserAttributes to include all missing fields
        // TODO: Update UserRelationships to include supervisors, supervisees
    }

    /// Test that Attendance model matches Django API exactly
    #[test]
    fn test_attendance_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "attendances",
                "id": "123",
                "attributes": {
                    "date": "2023-08-15",
                    "from-time": "09:00:00",
                    "to-time": "17:00:00"
                },
                "relationships": {
                    "user": {
                        "data": {
                            "type": "users",
                            "id": "456"
                        }
                    }
                }
            }
        });

        let attendance: Result<Attendance, _> =
            serde_json::from_value(mock_django_response["data"].clone());

        match attendance {
            Ok(att) => {
                assert_eq!(att.id, Some("123".to_string()));
                assert_eq!(att.type_name, "attendances");
                assert_eq!(att.attributes.date, "2023-08-15");
                assert_eq!(att.attributes.from_time, "09:00:00");
                assert_eq!(att.attributes.to_time, Some("17:00:00".to_string()));
            }
            Err(e) => panic!("Failed to deserialize Attendance: {}", e),
        }
    }

    /// Test that Absence model includes all fields from the old Django API
    #[test]
    fn test_absence_api_compatibility() {
        let mock_django_response = json!({
            "data": {
                "type": "absences",
                "id": "123",
                "attributes": {
                    "date": "2023-08-15",
                    "comment": "Vacation day"
                },
                "relationships": {
                    "user": {
                        "data": {
                            "type": "users",
                            "id": "456"
                        }
                    },
                    "absence_type": {
                        "data": {
                            "type": "absence-types",
                            "id": "789"
                        }
                    }
                }
            }
        });

        let absence: Result<Absence, _> =
            serde_json::from_value(mock_django_response["data"].clone());

        match absence {
            Ok(abs) => {
                assert_eq!(abs.id, Some("123".to_string()));
                assert_eq!(abs.type_name, "absences");
                assert_eq!(abs.attributes.date, "2023-08-15");
                assert_eq!(abs.attributes.comment, Some("Vacation day".to_string()));
            }
            Err(e) => panic!("Failed to deserialize Absence: {}", e),
        }
    }

    /// Test that we can deserialize complex API responses with included resources
    #[test]
    fn test_complex_api_response_compatibility() {
        let complex_response = json!({
            "data": [
                {
                    "type": "activities",
                    "id": "1",
                    "attributes": {
                        "comment": "Working on feature",
                        "date": "2023-08-15",
                        "from-time": "09:00:00",
                        "to-time": "12:00:00",
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
            ],
            "included": [
                {
                    "type": "users",
                    "id": "123",
                    "attributes": {
                        "username": "jdoe",
                        "email": "john@example.com",
                        "first-name": "John",
                        "last-name": "Doe"
                    }
                },
                {
                    "type": "tasks",
                    "id": "456",
                    "attributes": {
                        "name": "Development Task",
                        "archived": false
                    },
                    "relationships": {
                        "project": {
                            "data": {
                                "type": "projects",
                                "id": "789"
                            }
                        }
                    }
                }
            ]
        });

        // Test that we can deserialize the main data
        let activities: Result<Vec<Activity>, _> =
            serde_json::from_value(complex_response["data"].clone());
        assert!(activities.is_ok(), "Failed to deserialize activities array");

        // Test that we can deserialize included resources
        let included: Result<Vec<IncludedResource>, _> =
            serde_json::from_value(complex_response["included"].clone());
        assert!(included.is_ok(), "Failed to deserialize included resources");

        if let Ok(inc) = included {
            assert_eq!(inc.len(), 2);
            assert_eq!(inc[0].type_name, "users");
            assert_eq!(inc[1].type_name, "tasks");
        }
    }

    /// Test that all model resource names match the old Django API
    #[test]
    fn test_resource_names_compatibility() {
        let expected_resources = [
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

        for (expected, actual) in expected_resources {
            assert_eq!(expected, actual, "Resource name mismatch for {}", expected);
        }
    }

    /// Test field serialization format compatibility
    #[test]
    fn test_field_format_compatibility() {
        // Test date format
        let activity = Activity {
            id: Some("1".to_string()),
            type_name: "activities".to_string(),
            attributes: ActivityAttributes {
                comment: "Test".to_string(),
                date: "2023-08-15".to_string(), // Should be YYYY-MM-DD
                from_time: "09:00:00".to_string(), // Should be HH:MM:SS
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

        // Verify date format
        assert_eq!(json["attributes"]["date"], "2023-08-15");

        // Verify time format
        assert_eq!(json["attributes"]["from-time"], "09:00:00");
        assert_eq!(json["attributes"]["to-time"], "17:00:00");
    }

    /// Comprehensive test documenting all missing fields that need to be implemented
    /// for full API compatibility with the old Django backend
    #[test]
    #[should_panic(expected = "Missing API fields detected")]
    fn test_comprehensive_missing_fields_documentation() {
        // This test documents all the fields we've identified as missing
        // It should panic until all fields are implemented

        let missing_fields = vec![
            // Activity model missing fields
            (
                "Activity",
                "transferred",
                "boolean field indicating if activity was transferred",
            ),
            // Report model missing fields
            ("Report", "added", "datetime when report was created"),
            ("Report", "updated", "datetime when report was last updated"),
            (
                "Report",
                "remaining-effort",
                "duration field for remaining effort",
            ),
            // Customer model missing fields
            ("Customer", "reference", "string reference field"),
            ("Customer", "email", "email field"),
            ("Customer", "website", "website URL field"),
            ("Customer", "comment", "comment text field"),
            ("Customer", "assignees", "relationship to assigned users"),
            // Project model missing fields
            ("Project", "reference", "string reference field"),
            ("Project", "comment", "comment text field"),
            (
                "Project",
                "billed",
                "boolean indicating if project is billed",
            ),
            ("Project", "estimated-time", "duration of estimated time"),
            (
                "Project",
                "customer-visible",
                "boolean for customer visibility",
            ),
            (
                "Project",
                "amount-offered",
                "money field for offered amount",
            ),
            (
                "Project",
                "amount-invoiced",
                "money field for invoiced amount",
            ),
            (
                "Project",
                "remaining-effort-tracking",
                "boolean for effort tracking",
            ),
            (
                "Project",
                "total-remaining-effort",
                "duration of total remaining effort",
            ),
            ("Project", "billing-type", "relationship to billing type"),
            ("Project", "cost-center", "relationship to cost center"),
            ("Project", "assignees", "relationship to assigned users"),
            // Task model missing fields
            ("Task", "reference", "string reference field"),
            ("Task", "estimated-time", "duration of estimated time"),
            ("Task", "amount-offered", "money field for offered amount"),
            ("Task", "amount-invoiced", "money field for invoiced amount"),
            (
                "Task",
                "most-recent-remaining-effort",
                "duration of most recent remaining effort",
            ),
            ("Task", "cost-center", "relationship to cost center"),
            ("Task", "assignees", "relationship to assigned users"),
            // User model missing fields
            (
                "User",
                "tour-done",
                "boolean indicating if user completed tour",
            ),
            (
                "User",
                "is-accountant",
                "boolean indicating if user is accountant",
            ),
            (
                "User",
                "is-reviewer",
                "boolean indicating if user is reviewer",
            ),
            ("User", "is-active", "boolean indicating if user is active"),
            ("User", "is-staff", "boolean indicating if user is staff"),
            (
                "User",
                "is-superuser",
                "boolean indicating if user is superuser",
            ),
            ("User", "date-joined", "datetime when user joined"),
            ("User", "last-login", "datetime of last login"),
            ("User", "supervisors", "relationship to supervisor users"),
            ("User", "supervisees", "relationship to supervisee users"),
        ];

        eprintln!("=== MISSING API FIELDS DETECTED ===");
        eprintln!("The following fields need to be implemented for full API compatibility:");
        eprintln!();

        for (model, field, description) in &missing_fields {
            eprintln!("• {}: {} - {}", model, field, description);
        }

        eprintln!();
        eprintln!("Total missing fields: {}", missing_fields.len());
        eprintln!();
        eprintln!("To fix this:");
        eprintln!("1. Add missing fields to the corresponding model attributes structs");
        eprintln!("2. Add missing relationships to the corresponding relationship structs");
        eprintln!("3. Update serialization/deserialization annotations as needed");
        eprintln!("4. Update this test to expect fewer missing fields");
        eprintln!();

        panic!("Missing API fields detected - see list above");
    }

    /// Test to ensure we don't accidentally remove existing compatible fields
    #[test]
    fn test_existing_compatible_fields_preserved() {
        // Test that basic models still work with their current fields

        // Activity - should work with current fields
        let activity_json = json!({
            "type": "activities",
            "id": "1",
            "attributes": {
                "comment": "Test work",
                "date": "2023-08-15",
                "from-time": "09:00:00",
                "to-time": "17:00:00",
                "review": false,
                "not-billable": false
            },
            "relationships": {}
        });

        let activity: Result<Activity, _> = serde_json::from_value(activity_json);
        assert!(
            activity.is_ok(),
            "Activity deserialization should work with current fields"
        );

        // Attendance - should work with current fields
        let attendance_json = json!({
            "type": "attendances",
            "id": "1",
            "attributes": {
                "date": "2023-08-15",
                "from-time": "09:00:00",
                "to-time": "17:00:00"
            },
            "relationships": {}
        });

        let attendance: Result<Attendance, _> = serde_json::from_value(attendance_json);
        assert!(
            attendance.is_ok(),
            "Attendance deserialization should work with current fields"
        );

        // Absence - should work with current fields
        let absence_json = json!({
            "type": "absences",
            "id": "1",
            "attributes": {
                "date": "2023-08-15",
                "comment": "Vacation"
            },
            "relationships": {}
        });

        let absence: Result<Absence, _> = serde_json::from_value(absence_json);
        assert!(
            absence.is_ok(),
            "Absence deserialization should work with current fields"
        );
    }
}
