use runar_macros::{action, service};
use runar_node::services::{ResponseStatus};
use runar_node::services::abstract_service::AbstractService;
use runar_node::{Node};
use anyhow::{Result, Error};
use std::collections::HashMap;
use std::path::PathBuf;
use runar_common::types::ValueType;
use runar_common::{vmap};
use tempfile::tempdir;

// Define a simple test service
#[derive(Clone)]
#[service(
    name = "test",
    description = "A simple test service",
    version = "1.0.0"
)]
struct TestService {}

impl TestService {
    fn new() -> Self {
        Self {}
    }

    // Action with the action macro that uses direct parameters instead of ServiceRequest
    #[action(name = "add")]
    async fn add(&self, a: i32, b: i32) -> Result<i32> {
        // Directly use the parameters
        Ok(a + b)
    }

    // Action with default name using direct parameters
    #[action]
    async fn multiply(&self, a: i32, b: i32) -> Result<i32> {
        // Directly use the parameters
        Ok(a * b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_action_macro() -> Result<(), Error> {
        // Create a test service instance
        let service = TestService::new();
        
        // Test the add action generated handler method directly
        let add_params = vmap!{
            "a" => 5,
            "b" => 3
        };
        
        // Create a request for the add action
        let add_request = runar_node::services::ServiceRequest {
            path: "test".to_string(),
            action: "add".to_string(),
            data: Some(add_params),
            request_id: None,
            context: std::sync::Arc::new(runar_node::services::RequestContext::default()),
            metadata: None,
            topic_path: None,
        };
        
        // Call handle_request which will route to the handle_add method
        let add_response = service.handle_request(add_request).await?;
        
        // Check add result
        assert_eq!(add_response.status, ResponseStatus::Success);
        
        if let Some(ValueType::Number(result)) = add_response.data {
            assert_eq!(result, 8.0);
            println!("Add test passed: 5 + 3 = {}", result);
        } else {
            panic!("Expected Number result in add response");
        }
        
        // Test the multiply action generated handler method directly
        let multiply_params = vmap!{
            "a" => 4,
            "b" => 7
        };
        
        // Create a request for the multiply action
        let multiply_request = runar_node::services::ServiceRequest {
            path: "test".to_string(),
            action: "multiply".to_string(),
            data: Some(multiply_params),
            request_id: None,
            context: std::sync::Arc::new(runar_node::services::RequestContext::default()),
            metadata: None,
            topic_path: None,
        };
        
        // Call handle_request which will route to the handle_multiply method
        let multiply_response = service.handle_request(multiply_request).await?;
        
        // Check multiply result
        assert_eq!(multiply_response.status, ResponseStatus::Success);
        
        if let Some(ValueType::Number(result)) = multiply_response.data {
            assert_eq!(result, 28.0);
            println!("Multiply test passed: 4 * 7 = {}", result);
        } else {
            panic!("Expected Number result in multiply response");
        }
        
        // TODO: Implement proper Node API test once the Node functionality is available
        // The current approach is a temporary solution until we have access to the proper Node API
        // This will need to be updated according to the Critical issue in the macro_remediation_analysis.md
        
        Ok(())
    }
} 