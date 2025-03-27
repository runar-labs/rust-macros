// Import required structs and traits
use runar_node::services::{RequestContext, ServiceResponse, ServiceRequest, ResponseStatus, ServiceState};
use runar_node::services::abstract_service::AbstractService;
use anyhow::{Result, Error};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use runar_common::types::ValueType;
use runar_common::vmap;

// Define a calculator service that demonstrates the action macro usage with vmap!
#[derive(Clone)]
struct CalculatorService {
    counter: u32,
}

impl CalculatorService {
    fn new() -> Self {
        Self { counter: 0 }
    }

    // Action for adding two numbers using vmap! for parameter extraction
    async fn add(&self, request: &ServiceRequest) -> Result<ServiceResponse> {
        // Create an empty HashMap with a longer lifetime
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        
        // Extract parameters using reference to data or empty map
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract a and b directly from ValueType
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        // Calculate result
        let result = a + b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Addition result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }

    // Action for subtracting two numbers using vmap! for parameter extraction
    async fn subtract(&self, request: &ServiceRequest) -> Result<ServiceResponse> {
        // Create an empty HashMap with a longer lifetime
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        
        // Extract parameters using reference to data or empty map
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract a and b directly from ValueType
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        // Calculate result
        let result = a - b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Subtraction result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }

    // Action for multiplying two numbers using vmap! for parameter extraction
    async fn multiply(&self, request: &ServiceRequest) -> Result<ServiceResponse> {
        // Create an empty HashMap with a longer lifetime
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        
        // Extract parameters using reference to data or empty map
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract a and b directly from ValueType
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        // Calculate result
        let result = a * b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Multiplication result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }

    // Action for dividing two numbers using vmap! for parameter extraction
    async fn divide(&self, request: &ServiceRequest) -> Result<ServiceResponse> {
        // Create an empty HashMap with a longer lifetime
        let empty_map = HashMap::new();
        let empty_value = ValueType::Map(empty_map);
        
        // Extract parameters using reference to data or empty map
        let data = request.data.as_ref().unwrap_or(&empty_value);
        
        // Extract a and b directly from ValueType
        let a = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("a") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        let b = match data {
            ValueType::Map(map) => {
                if let Some(ValueType::Number(num)) = map.get("b") {
                    *num as i32
                } else {
                    0
                }
            },
            _ => 0
        };
        
        // Check for division by zero
        if b == 0 {
            return Ok(ServiceResponse::error("Division by zero is not allowed"));
        }
        
        // Calculate result
        let result = a / b;
        
        // Return success response
        Ok(ServiceResponse::success(
            format!("Division result: {}", result),
            Some(ValueType::Number(result as f64))
        ))
    }
}

#[async_trait]
impl AbstractService for CalculatorService {
    fn name(&self) -> &str {
        "calculator"
    }

    fn path(&self) -> &str {
        "calculator"
    }

    fn state(&self) -> ServiceState {
        ServiceState::Running
    }

    fn description(&self) -> &str {
        "A simple calculator service"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn init(&mut self, _ctx: &RequestContext) -> Result<()> {
        println!("Initializing CalculatorService");
        self.counter = 0;
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        println!("Starting CalculatorService");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        println!("Stopping CalculatorService");
        Ok(())
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        match request.action.as_str() {
            "add" => self.add(&request).await,
            "subtract" => self.subtract(&request).await,
            "multiply" => self.multiply(&request).await,
            "divide" => self.divide(&request).await,
            _ => Ok(ServiceResponse::error(format!("Unknown operation: {}", request.action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_action_handlers() -> Result<(), Error> {
        // Create a calculator service
        let service = CalculatorService::new();
        
        // Create a request context for testing
        let context = Arc::new(RequestContext::new("test", ValueType::Null, Arc::new(service.clone())));
        
        // Test addition
        let add_request = ServiceRequest {
            path: "calculator".to_string(),
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
        
        let add_response = service.handle_request(add_request).await?;
        assert_eq!(add_response.status, ResponseStatus::Success);
        if let Some(ValueType::Number(result)) = add_response.data {
            assert_eq!(result, 8.0);
        } else {
            panic!("Expected Number result in add response");
        }
        
        // Test subtraction
        let sub_request = ServiceRequest {
            path: "calculator".to_string(),
            action: "subtract".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(10.0));
                map.insert("b".to_string(), ValueType::Number(4.0));
                map
            })),
            request_id: Some("test_id_2".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        let sub_response = service.handle_request(sub_request).await?;
        assert_eq!(sub_response.status, ResponseStatus::Success);
        if let Some(ValueType::Number(result)) = sub_response.data {
            assert_eq!(result, 6.0);
        } else {
            panic!("Expected Number result in subtract response");
        }
        
        // Test multiplication
        let mul_request = ServiceRequest {
            path: "calculator".to_string(),
            action: "multiply".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(7.0));
                map.insert("b".to_string(), ValueType::Number(6.0));
                map
            })),
            request_id: Some("test_id_3".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        let mul_response = service.handle_request(mul_request).await?;
        assert_eq!(mul_response.status, ResponseStatus::Success);
        if let Some(ValueType::Number(result)) = mul_response.data {
            assert_eq!(result, 42.0);
        } else {
            panic!("Expected Number result in multiply response");
        }
        
        // Test division
        let div_request = ServiceRequest {
            path: "calculator".to_string(),
            action: "divide".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(20.0));
                map.insert("b".to_string(), ValueType::Number(5.0));
                map
            })),
            request_id: Some("test_id_4".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        let div_response = service.handle_request(div_request).await?;
        assert_eq!(div_response.status, ResponseStatus::Success);
        if let Some(ValueType::Number(result)) = div_response.data {
            assert_eq!(result, 4.0);
        } else {
            panic!("Expected Number result in divide response");
        }
        
        // Test division by zero
        let div_zero_request = ServiceRequest {
            path: "calculator".to_string(),
            action: "divide".to_string(),
            data: Some(ValueType::Map({
                let mut map = HashMap::new();
                map.insert("a".to_string(), ValueType::Number(10.0));
                map.insert("b".to_string(), ValueType::Number(0.0));
                map
            })),
            request_id: Some("test_id_5".to_string()),
            metadata: None,
            context: context.clone(),
            topic_path: None,
        };
        
        let div_zero_response = service.handle_request(div_zero_request).await?;
        assert_eq!(div_zero_response.status, ResponseStatus::Error);
        assert!(div_zero_response.message.contains("Division by zero"));
        
        Ok(())
    }
} 