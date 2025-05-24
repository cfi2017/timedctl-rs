#[cfg(test)]
mod tests {
    use crate::models::*;
    use serde_json::json;

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
    }

    #[test]
    fn test_absence_type_serialization() {
        let absence_type = AbsenceType {
            id: Some("123".to_string()),
            type_name: "absence-types".to_string(),
            attributes: AbsenceTypeAttributes {
                name: "Vacation".to_string(),
                fill_worktime: true,
            },
        };

        let json = serde_json::to_string(&absence_type).unwrap();
        assert!(json.contains("absence-types"));
        assert!(json.contains("Vacation"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_year_statistic_serialization() {
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
        assert!(json.contains("users"));
        assert!(json.contains("456"));
    }

    #[test]
    fn test_month_statistic_serialization() {
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
        assert!(json.contains("168:00:00"));
        assert!(json.contains("users"));
        assert!(json.contains("456"));
    }

    #[test]
    fn test_enhanced_filter_params() {
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
}
