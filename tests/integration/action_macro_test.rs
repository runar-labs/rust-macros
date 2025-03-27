use anyhow::Result;
use async_trait::async_trait;
use runar_macros::{action, service};
use runar_node::{
    services::{ServiceResponse, ServiceRequest, ResponseStatus, registry::AbstractService},
    vmap, ValueType, Node,
};

// Create an implementation of the AbstractService trait for our CalculatorService
#[derive(Clone)]
struct CalculatorService {
    counter: std::sync::atomic::AtomicU32,
}

#[async_trait]
impl AbstractService for CalculatorService {
    fn id(&self) -> &str {
        "calculator_service"
    }

    fn name(&self) -> &str {
        "Calculator Service"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "A simple calculator service for testing action macros"
    }

    async fn init(&self) -> Result<()> {
        Ok(())
    }

    fn path(&self) -> Option<&str> {
        Some("calculator")
    }
}

impl CalculatorService {
    fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn increment_counter(&self) {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn get_counter(&self) -> u32 {
        self.counter.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl CalculatorService {
    #[action]
    async fn add(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
        self.increment_counter();
        
        // Extract parameters using vmap! macro
        if let Some(params) = &request.params {
            let a = runar_node::vmap!(params, "a" => 0.0);
            let b = runar_node::vmap!(params, "b" => 0.0);
            
            let result = a + b;
            
            let result_vmap = runar_node::vmap! {
                "result" => result
            };
            
            Ok(ServiceResponse::success(
                "Addition performed successfully".to_string(),
                Some(runar_node::ValueType::from(result_vmap)),
            ))
        } else {
            Ok(ServiceResponse::error("Parameters required for addition".to_string()))
        }
    }

    #[action]
    async fn subtract(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
        self.increment_counter();
        
        // Extract parameters using vmap! macro
        if let Some(params) = &request.params {
            let a = runar_node::vmap!(params, "a" => 0.0);
            let b = runar_node::vmap!(params, "b" => 0.0);
            
            let result = a - b;
            
            let result_vmap = runar_node::vmap! {
                "result" => result
            };
            
            Ok(ServiceResponse::success(
                "Subtraction performed successfully".to_string(),
                Some(runar_node::ValueType::from(result_vmap)),
            ))
        } else {
            Ok(ServiceResponse::error("Parameters required for subtraction".to_string()))
        }
    }

    #[action]
    async fn multiply(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
        self.increment_counter();
        
        // Extract parameters using vmap! macro
        if let Some(params) = &request.params {
            let a = runar_node::vmap!(params, "a" => 0.0);
            let b = runar_node::vmap!(params, "b" => 0.0);
            
            let result = a * b;
            
            let result_vmap = runar_node::vmap! {
                "result" => result
            };
            
            Ok(ServiceResponse::success(
                "Multiplication performed successfully".to_string(),
                Some(runar_node::ValueType::from(result_vmap)),
            ))
        } else {
            Ok(ServiceResponse::error("Parameters required for multiplication".to_string()))
        }
    }

    #[action]
    async fn divide(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
        self.increment_counter();
        
        // Extract parameters using vmap! macro
        if let Some(params) = &request.params {
            let a = runar_node::vmap!(params, "a" => 0.0);
            let b = runar_node::vmap!(params, "b" => 0.0);
            
            if b == 0.0 {
                return Ok(ServiceResponse::error("Division by zero is not allowed".to_string()));
            }
            
            let result = a / b;
            
            let result_vmap = runar_node::vmap! {
                "result" => result
            };
            
            Ok(ServiceResponse::success(
                "Division performed successfully".to_string(),
                Some(runar_node::ValueType::from(result_vmap)),
            ))
        } else {
            Ok(ServiceResponse::error("Parameters required for division".to_string()))
        }
    }
}

#[tokio::test]
async fn test_action_handlers() -> Result<()> {
    // Initialize the node with our calculator service
    let mut node = runar_node::Node::new("memory".to_string(), "test".to_string()).await?;
    let calculator = CalculatorService::new();
    node.add_service(calculator).await?;
    
    // Test add action
    let add_params = runar_node::vmap! {
        "a" => 5.0,
        "b" => 3.0
    };
    
    let add_response = node
        .execute_service_action(
            "calculator_service",
            "add",
            Some(runar_node::ValueType::from(add_params)),
        )
        .await?;

    assert_eq!(add_response.status, ResponseStatus::Success);
    if let Some(data) = &add_response.data {
        let result = runar_node::vmap!(data, "result" => 0.0);
        assert_eq!(result, 8.0);
    } else {
        panic!("Expected response data from add request");
    }

    // Test subtract action
    let subtract_params = runar_node::vmap! {
        "a" => 10.0,
        "b" => 4.0
    };
    
    let subtract_response = node
        .execute_service_action(
            "calculator_service",
            "subtract",
            Some(runar_node::ValueType::from(subtract_params)),
        )
        .await?;
        
    assert_eq!(subtract_response.status, ResponseStatus::Success);
    if let Some(data) = &subtract_response.data {
        let result = runar_node::vmap!(data, "result" => 0.0);
        assert_eq!(result, 6.0);
    } else {
        panic!("Expected response data from subtract request");
    }

    // Test multiply action
    let multiply_params = runar_node::vmap! {
        "a" => 7.0,
        "b" => 6.0
    };
    
    let multiply_response = node
        .execute_service_action(
            "calculator_service",
            "multiply",
            Some(runar_node::ValueType::from(multiply_params)),
        )
        .await?;
        
    assert_eq!(multiply_response.status, ResponseStatus::Success);
    if let Some(data) = &multiply_response.data {
        let result = runar_node::vmap!(data, "result" => 0.0);
        assert_eq!(result, 42.0);
    } else {
        panic!("Expected response data from multiply request");
    }

    // Test divide action
    let divide_params = runar_node::vmap! {
        "a" => 20.0,
        "b" => 5.0
    };
    
    let divide_response = node
        .execute_service_action(
            "calculator_service",
            "divide",
            Some(runar_node::ValueType::from(divide_params)),
        )
        .await?;
        
    assert_eq!(divide_response.status, ResponseStatus::Success);
    if let Some(data) = &divide_response.data {
        let result = runar_node::vmap!(data, "result" => 0.0);
        assert_eq!(result, 4.0);
    } else {
        panic!("Expected response data from divide request");
    }

    // Test division by zero error handling
    let divide_by_zero_params = runar_node::vmap! {
        "a" => 20.0,
        "b" => 0.0
    };
    
    let divide_by_zero_response = node
        .execute_service_action(
            "calculator_service",
            "divide",
            Some(runar_node::ValueType::from(divide_by_zero_params)),
        )
        .await?;
        
    assert_eq!(divide_by_zero_response.status, ResponseStatus::Error);
    
    Ok(())
} 