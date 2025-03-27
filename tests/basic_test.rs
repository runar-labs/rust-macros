use runar_node::services::{RequestContext, ServiceResponse, ServiceRequest, ResponseStatus, ServiceState};
use runar_node::services::abstract_service::AbstractService;
use anyhow::{Result, Error};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use runar_common::types::ValueType;

// Define a simple test service without using macros
#[derive(Clone)]
struct TestService {}

impl TestService {
    fn new() -> Self {
        Self {}
    }

    // Manual parameter extraction from request
    async fn handle_add(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        // Extract parameters from request.data
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract values using match expressions
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    return Ok(ServiceResponse::error("Missing parameter 'a'"))
                }
            },
            _ => return Ok(ServiceResponse::error("Invalid request format"))
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    return Ok(ServiceResponse::error("Missing parameter 'b'"))
                }
            },
            _ => return Ok(ServiceResponse::error("Invalid request format"))
        };
        
        // Calculate result
        let result = a + b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Addition result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }

    // Manual parameter extraction from request
    async fn handle_multiply(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        // Extract parameters from request.data
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract values using match expressions
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    return Ok(ServiceResponse::error("Missing parameter 'a'"))
                }
            },
            _ => return Ok(ServiceResponse::error("Invalid request format"))
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    return Ok(ServiceResponse::error("Missing parameter 'b'"))
                }
            },
            _ => return Ok(ServiceResponse::error("Invalid request format"))
        };
        
        // Calculate result
        let result = a * b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Multiplication result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }
}

#[async_trait]
impl AbstractService for TestService {
    fn name(&self) -> &str {
        "test"
    }

    fn path(&self) -> &str {
        "test"
    }

    fn state(&self) -> ServiceState {
        ServiceState::Running
    }

    fn description(&self) -> &str {
        "A simple test service"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn init(&mut self, _ctx: &runar_node::services::RequestContext) -> Result<()> {
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        match request.action.as_str() {
            "add" => self.handle_add(request).await,
            "multiply" => self.handle_multiply(request).await,
            _ => Ok(ServiceResponse::error(format!("Unknown operation: {}", request.action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use runar_node::services::NodeRequestHandler;

    // Create a minimal NodeRequestHandler implementation for our test
    struct MinimalNodeHandler;
    
    #[async_trait]
    impl NodeRequestHandler for MinimalNodeHandler {
        async fn request(&self, _path: String, _params: ValueType) -> Result<ServiceResponse> {
            Ok(ServiceResponse::error("Not implemented"))
        }
        
        async fn publish(&self, _topic: String, _data: ValueType) -> Result<()> {
            Ok(())
        }
        
        async fn subscribe(&self, _topic: String, _callback: Box<dyn Fn(ValueType) -> Result<()> + Send + Sync>) -> Result<String> {
            Ok("test-subscription".to_string())
        }
        
        async fn subscribe_with_options(&self, _topic: String, _callback: Box<dyn Fn(ValueType) -> Result<()> + Send + Sync>, _options: runar_node::services::SubscriptionOptions) -> Result<String> {
            Ok("test-subscription".to_string())
        }
        
        async fn unsubscribe(&self, _topic: String, _subscription_id: Option<&str>) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_basic_service() -> Result<(), Error> {
        // Create a service instance
        let service = TestService::new();
        
        // Create a node handler
        let node_handler = Arc::new(MinimalNodeHandler);
        
        // Create a request context
        let context = Arc::new(RequestContext::new(
            "test",
            ValueType::Null,
            node_handler
        ));
        
        // Create test request with data for add
        let add_request = ServiceRequest {
            path: "test".to_string(),
            action: "add".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(5.0));
                map.insert("b".to_string(), ValueType::Number(3.0));
                map
            })),
            request_id: Some("test_id_1".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        // Process the add request
        let add_response = service.handle_request(add_request).await?;
        
        // Check add result
        assert_eq!(add_response.status, ResponseStatus::Success);
        
        if let Some(ValueType::Number(result)) = add_response.data {
            assert_eq!(result, 8.0);
            println!("Add test passed: 5 + 3 = {}", result);
        } else {
            panic!("Expected Number result in add response");
        }
        
        // Create test request with data for multiply
        let multiply_request = ServiceRequest {
            path: "test".to_string(),
            action: "multiply".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(4.0));
                map.insert("b".to_string(), ValueType::Number(7.0));
                map
            })),
            request_id: Some("test_id_2".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        // Process the multiply request  
        let multiply_response = service.handle_request(multiply_request).await?;
        
        // Check multiply result
        assert_eq!(multiply_response.status, ResponseStatus::Success);
        
        if let Some(ValueType::Number(result)) = multiply_response.data {
            assert_eq!(result, 28.0);
            println!("Multiply test passed: 4 * 7 = {}", result);
        } else {
            panic!("Expected Number result in multiply response");
        }
        
        Ok(())
    }
} 