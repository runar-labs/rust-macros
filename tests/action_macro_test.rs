use runar_macros::{action, service};
use runar_macros::{action, service};
use runar_node::Node;
use runar_common::types::{ArcValueType, hmap};

// Define a simple test service
#[derive(Clone)]
#[service(
    name = "test",
    description = "A simple test service",
    version = "1.0.0"
)]
struct TestService {

}

impl TestService {
    fn new() -> Self {
        Self {}
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

        let node = Node::new("test");
        node.add_service(service);

        node.start().await?;

        let add_params = ArcValueType::new_map(hmap! {
            "a" => 5.0,
            "b" => 3.0
        });
    
        // Use the proper network path format - with network ID for remote actions
        let response = node.request("test/add", add_params).await?;
        if let Some(mut result_value) = response.data {
            let result: f64 = result_value.as_type()?;
            assert_eq!(result, 8.0);
            println!("Add operation succeeded: 5 + 3 = {}", result);
        } else {
            return Err(anyhow::anyhow!("Unexpected response type: {:?}", response.data));
        }
    
        // Test calling math service2 (on node2) from node1
        println!("Testing remote action call from node1 to node2...");
        let multiply_params = ArcValueType::new_map(hmap! {
            "a" => 4.0,
            "b" => 7.0
        });
        
        let response = node.request("test/multiply", multiply_params).await?;
        if let Some(mut result_value) = response.data {
            let result: f64 = result_value.as_type()?;
            assert_eq!(result, 28.0);
            println!("Multiply operation succeeded: 4 * 7 = {}", result);
        } else {
            return Err(anyhow::anyhow!("Unexpected response type: {:?}", response.data));
        }
    }
} 