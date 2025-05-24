use libtimed::{models::*, Result, TimedClient};
use mockito::{self, Server};
use serde_json::json;

fn test_client_get_users() -> Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut server = Server::new();

    // Mock the users endpoint
    let mock = server
        .mock("GET", "/api/v1/users/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "data": {
                "id": "123",
                "type": "users",
                "attributes": {
                    "username": "testuser",
                    "email": "test@example.com",
                    "first-name": "Test",
                    "last-name": "User"
                }
            }
        }"#,
        )
        .create();

    // Create client with the mock server URL
    let client = TimedClient::new(&server.url(), "api/v1", Some("mock-token".to_string()));

    // Call the API
    let response = rt.block_on(client.get::<serde_json::Value>("users/me", None))?;

    // Verify the response
    assert_eq!(response["data"]["id"], "123");
    assert_eq!(response["data"]["attributes"]["username"], "testuser");

    // Verify the mock was called
    mock.assert();

    Ok(())
}

fn test_client_get_activities() -> Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut server = Server::new();

    // Mock the activities endpoint
    let mock = server
        .mock("GET", "/api/v1/activities")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
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
            ]
        }"#,
        )
        .create();

    // Create client with the mock server URL
    let client = TimedClient::new(&server.url(), "api/v1", Some("mock-token".to_string()));

    // Create filter params
    let mut filter = FilterParams::default();
    filter.date = Some("2023-07-15".to_string());
    filter.include = Some("task,user".to_string());

    // Call the API
    let response =
        rt.block_on(client.get::<ResourcesResponse<Activity>>("activities", Some(&filter)))?;

    // Verify the response
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, Some("123".to_string()));
    assert_eq!(response.data[0].attributes.comment, "Working on something");
    assert_eq!(response.data[0].attributes.date, "2023-07-15");

    // Verify the mock was called
    mock.assert();

    Ok(())
}

fn test_client_post_activity() -> Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut server = Server::new();

    // Mock the activities endpoint for POST
    let mock = server
        .mock("POST", "/api/v1/activities")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "data": {
                "id": "999",
                "type": "activities",
                "attributes": {
                    "comment": "New activity",
                    "date": "2023-07-16",
                    "from-time": "10:00:00",
                    "to-time": null,
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
        }"#,
        )
        .create();

    // Create client with the mock server URL
    let client = TimedClient::new(&server.url(), "api/v1", Some("mock-token".to_string()));

    // Create new activity
    let activity = Activity {
        id: None,
        type_name: "activities".to_string(),
        attributes: ActivityAttributes {
            comment: "New activity".to_string(),
            date: "2023-07-16".to_string(),
            from_time: "10:00:00".to_string(),
            to_time: None,
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

    let request_body = json!({
        "data": activity
    });

    // Call the API
    let response =
        rt.block_on(client.post::<_, ResourceResponse<Activity>>("activities", &request_body))?;

    // Verify the response
    assert_eq!(response.data.id, Some("999".to_string()));
    assert_eq!(response.data.attributes.comment, "New activity");

    // Verify the mock was called
    mock.assert();

    Ok(())
}

fn test_client_worktime_balance() -> Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut server = Server::new();

    // Mock the worktime-balances endpoint
    let mock = server
        .mock("GET", "/api/v1/worktime-balances")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "data": [
                {
                    "id": "123",
                    "type": "worktime-balances",
                    "attributes": {
                        "date": "2023-07-15",
                        "balance": "08:30:00"
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
            ]
        }"#,
        )
        .create();

    // Create client with the mock server URL
    let client = TimedClient::new(&server.url(), "api/v1", Some("mock-token".to_string()));

    // Create filter params
    let mut filter = FilterParams::default();
    filter.date = Some("2023-07-15".to_string());

    // Call the API
    let response = rt.block_on(
        client.get::<ResourcesResponse<WorktimeBalance>>("worktime-balances", Some(&filter)),
    )?;

    // Verify the response
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, Some("123".to_string()));
    assert_eq!(response.data[0].attributes.date, "2023-07-15");
    assert_eq!(response.data[0].attributes.balance, "08:30:00");

    // Verify the mock was called
    mock.assert();

    Ok(())
}
