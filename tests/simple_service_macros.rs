// Test for the service and action macros
//
// This test demonstrates how to use the service and action macros
// to create a simple service with actions.

use anyhow::Result;
use runar_common::types::ArcValueType;
use runar_macros::{action, service};
use runar_node::services::RequestContext;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MyData {
    id: i32,
    text_field: String,
    number_field: i32,
    boolean_field: bool,
    float_field: f64,
    vector_field: Vec<i32>,
    map_field: HashMap<String, i32>,
}

// Define a simple math service
pub struct TestService {
    // Empty struct for testing
}

// Implement Clone manually for TestMathService
impl Clone for TestService {
    fn clone(&self) -> Self {
        Self {}
    }
}

#[service(name="Test Service Name", path="math", description="Test Service Description", version="0.0.1")]
impl TestService {

    #[action(path="my_data")]
    async fn get_my_data(&self, id: i32, ctx: &RequestContext) -> Result<MyData> {
        // Log using the context
        ctx.debug(format!("get_my_data id: {}", id));

        let total_res = ctx.request("math/add", ArcValueType::new_map(HashMap::from([("a_param".to_string(), 10.0), ("b_param".to_string(), 5.0)]))).await?;
        let mut data = total_res.data.unwrap();
        let total = data.as_type::<f64>()?;

        // Return the result
        Ok(MyData {
            id,
            text_field: "test".to_string(),
            number_field: id,
            boolean_field: true,
            float_field: total,
            vector_field: vec![1, 2, 3],
            map_field: HashMap::new(),
        })
    }

    // Define an action using the action macro
    #[action]
    async fn add(&self, a_param: f64, b_param: f64, ctx: &RequestContext) -> Result<f64> {
 

        // Log using the context
        ctx.debug(format!("Adding {} + {}", a_param, b_param));

        // Return the result
        Ok(a_param + b_param)
    }

    // Define another action
    #[action]
    async fn subtract(&self, a_param: f64, b_param: f64, ctx: &RequestContext) -> Result<f64> {
        
        // Log using the context
        ctx.debug(format!("Subtracting {} - {}", a_param, b_param));

        // Return the result
        Ok(a_param - b_param)
    }

    // Define an action with a custom name
    #[action("multiply_numbers")]
    async fn multiply(&self, a_param: f64, b_param: f64, ctx: &RequestContext) -> Result<f64> {
 
        // Log using the context
        ctx.debug(format!("Multiplying {} * {}", a_param, b_param));

        // Return the result
        Ok(a_param * b_param)
    }

    // Define an action that can fail
    #[action]
    async fn divide(&self, a_param: f64, b_param: f64, ctx: &RequestContext) -> Result<f64> {
 
        // Log using the context
        ctx.debug(format!("Dividing {} / {}", a_param, b_param));

        // Check for division by zero
        if b_param == 0.0 {
            ctx.error("Division by zero".to_string());
            return Err(anyhow::anyhow!("Division by zero"));
        }

        // Return the result
        Ok(a_param / b_param)
    }
 
}

#[cfg(test)]
mod tests {
    use super::*;
    use runar_node::Node;
    use runar_node::NodeConfig;

    #[tokio::test]
    async fn test_math_service() {
        // Create a node with a test network ID
        let mut config = NodeConfig::new("test-node", "test_network");
        // Disable networking
        config.network_config = None;
        let mut node = Node::new(config).await.unwrap();

        // Create a test math service
        let service = TestService{};

        // Add the service to the node
        node.add_service(service).await.unwrap();

        // Start the node to initialize all services
        node.start().await.unwrap();

        // Create parameters for the add action
        let mut map = std::collections::HashMap::new();
        map.insert("a_param".to_string(), 10.0);
        map.insert("b_param".to_string(), 5.0);
        let params = ArcValueType::new_map(map);

        // Call the add action
        let response =
            node.request("math/add", params)
        .await.unwrap();

        // Verify the response
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap().as_type::<f64>().unwrap(),
            15.0
        );

        // Make a request to the subtract action
        let mut map = std::collections::HashMap::new();
        map.insert("a_param".to_string(), 10.0);
        map.insert("b_param".to_string(), 5.0);
        let params = ArcValueType::new_map(map);

        let response = node.request("math/subtract", params)
        .await.unwrap();

        // Verify the response
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap().as_type::<f64>().unwrap(),
            5.0
        );

        // Make a request to the multiply action (with custom name)
        let mut map = std::collections::HashMap::new();
        map.insert("a_param".to_string(), 5.0);
        map.insert("b_param".to_string(), 3.0);
        let params = ArcValueType::new_map(map);

        let response = node.request("math/multiply_numbers", params)
        .await.unwrap();

        // Verify the response
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap().as_type::<f64>().unwrap(),
            15.0
        );

        // Make a request to the divide action with valid parameters
        let mut map = std::collections::HashMap::new();
        map.insert("a_param".to_string(), 6.0);
        map.insert("b_param".to_string(), 3.0);
        let params = ArcValueType::new_map(map);

        let response = node.request("math/divide", params)
        .await.unwrap();

        // Verify the response 
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap().as_type::<f64>().unwrap(),
            2.0
        );

        // Make a request to the divide action with invalid parameters (division by zero)
        let mut map = std::collections::HashMap::new();
        map.insert("a_param".to_string(), 6.0);
        map.insert("b_param".to_string(), 0.0);
        let params = ArcValueType::new_map(map);

        let response = node.request("math/divide", params)
        .await.unwrap();

        // Verify the error response
        assert_eq!(response.status, 500);
        assert!(response.error.unwrap().contains("Division by zero"));


        // Make a request to the get_my_data action
        let mut map = std::collections::HashMap::new();
        map.insert("id".to_string(), 1);
        let params = ArcValueType::new_map(map);

        let response = node.request("math/my_data", params)
        .await.unwrap();

        // Verify the response
        assert_eq!(response.status, 200);
        let my_data = response.data.unwrap().as_type::<MyData>().unwrap();
        assert_eq!(
            my_data,
            MyData {
                id: 1,
                text_field: "test".to_string(),
                number_field: 1,
                boolean_field: true,
                float_field: 15.0,
                vector_field: vec![1, 2, 3],
                map_field: HashMap::new(),
            }
        );

    }
}
