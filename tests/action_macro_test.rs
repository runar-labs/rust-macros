use runar_macros::{action, service};
use runar_node::services::ServiceResponse;
use runar_node::services::ServiceRequest;
use runar_node::{Node};
use runar_common::types::ValueType;
use std::collections::HashMap;
use std::path::PathBuf;

use runar_node::services::abstract_service::AbstractService;
use tempfile::tempdir;
use runar_common;

// Define a simple test service
#[derive(Clone)]
#[service(
    name = "test",
    description = "A simple test service",
    version = "1.0.0"
)]
struct TestService {
    counter: u32,
}

impl TestService {
    fn new() -> Self {
        Self { counter: 0 }
    }

    // Action with the action macro that uses direct parameters instead of ServiceRequest
    #[action(name = "add")]
    async fn add(&self, a: i32, b: i32) -> anyhow::Result<i32> {
        Ok(a + b)
    }

    // Action with default name using direct parameters
    #[action]
    async fn multiply(&self, a: i32, b: i32) -> anyhow::Result<i32> {
        Ok(a * b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_action_macro() {
        // Create a test service
        let service = TestService::new();

        // Test the add action
        let response = service.add(3, 5).await.unwrap();
        assert_eq!(response, 8);

        // Test the multiply action
        let response = service.multiply(4, 7).await.unwrap();
        assert_eq!(response, 28);

        // This is a temporary test until we have proper Node API testing
        println!("Action macro test passed!");
    }
} 