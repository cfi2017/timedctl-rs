#[cfg(test)]
mod integration {
    #[cfg(test)]
    mod models_test;
    
    #[cfg(test)]
    mod client_test {
        use libtimed::{models::*, TimedClient, Result};
        use mockito::{self, Server};
        use serde_json::json;
        
        #[test]
        fn test_client_get_users() -> Result<()> {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut server = Server::new();
            
            // Mock the users endpoint
            let mock = server.mock("GET", "/api/v1/users/me")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
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
                }"#)
                .create();
            
            // Create client with the mock server URL
            let client = TimedClient::new(
                &server.url(),
                "api/v1",
                Some("mock-token".to_string())
            );
            
            // Call the API
            let response = rt.block_on(client.get::<serde_json::Value>("users/me", None)).unwrap();
            
            // Verify the response
            assert_eq!(response["data"]["id"], "123");
            assert_eq!(response["data"]["attributes"]["username"], "testuser");
            
            // Verify the mock was called
            mock.assert();
            
            Ok(())
        }
        
        #[test]
        fn test_client_get_activities() -> Result<()> {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut server = Server::new();
            
            // Mock the activities endpoint
            let mock = server.mock("GET", "/api/v1/activities")
                .match_query(mockito::Matcher::Any)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
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
                }"#)
                .create();
            
            // Create client with the mock server URL
            let client = TimedClient::new(
                &server.url(),
                "api/v1",
                Some("mock-token".to_string())
            );
            
            // Create filter params
            let mut filter = FilterParams::default();
            filter.date = Some("2023-07-15".to_string());
            filter.include = Some("task,user".to_string());
            
            // Call the API
            let response = rt.block_on(client.get::<ResourcesResponse<Activity>>("activities", Some(&filter))).unwrap();
            
            // Verify the response
            assert_eq!(response.data.len(), 1);
            assert_eq!(response.data[0].id, Some("123".to_string()));
            assert_eq!(response.data[0].attributes.comment, "Working on something");
            assert_eq!(response.data[0].attributes.date, "2023-07-15");
            
            // Verify the mock was called
            mock.assert();
            
            Ok(())
        }
    }
}