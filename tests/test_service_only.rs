mod common;
use runar_macros::service;
use runar_node::services::abstract_service::{AbstractService, ServiceState};
use runar_common::types::ValueType;

#[service]
pub struct TestService {
    field1: String,
    field2: u32,
}

#[test]
fn test_service_only() {
    let service = TestService {
        field1: "test".to_string(),
        field2: 42,
    };
    
    // Check service basic info
    assert_eq!(service.name(), "test_service");
    assert_eq!(service.path(), "test_service");
    assert_eq!(service.description(), "Service TestService");
    assert_eq!(service.version(), "0.1.0");
    assert_eq!(service.state(), ServiceState::Running);
    
    // Check backward compatibility methods
    assert_eq!(service.service_name(), "test_service");
    assert_eq!(service.service_path(), "/test_service");
    assert_eq!(service.service_description(), "Service TestService");
    assert_eq!(service.service_version(), "0.1.0");
}

#[service(name = "minimal_service")]
pub struct MinimalService {
    value: u64,
}

#[test]
fn test_service_minimal() {
    let service = MinimalService { value: 100 };
    
    // Check service basic info
    assert_eq!(service.name(), "minimal_service");
    assert_eq!(service.path(), "minimal_service");
    assert_eq!(service.description(), "Service MinimalService");
    assert_eq!(service.version(), "0.1.0");
    assert_eq!(service.state(), ServiceState::Running);
    
    // Check backward compatibility methods
    assert_eq!(service.service_name(), "minimal_service");
    assert_eq!(service.service_path(), "/minimal_service");
    assert_eq!(service.service_description(), "Service MinimalService");
    assert_eq!(service.service_version(), "0.1.0");
}

#[service(
    name = "custom_service",
    path = "/api/v1/custom",
    description = "A custom service with all params",
    version = "2.1.0"
)]
pub struct CustomParamService {
    counter: i32,
}

#[test]
fn test_service_with_all_params() {
    let service = CustomParamService { counter: 0 };
    
    // Check service basic info
    assert_eq!(service.name(), "custom_service");
    assert_eq!(service.path(), "api/v1/custom"); // Leading / is removed
    assert_eq!(service.description(), "A custom service with all params");
    assert_eq!(service.version(), "2.1.0");
    assert_eq!(service.state(), ServiceState::Running);
    
    // Check backward compatibility methods
    assert_eq!(service.service_name(), "custom_service");
    assert_eq!(service.service_path(), "/api/v1/custom");
    assert_eq!(service.service_description(), "A custom service with all params");
    assert_eq!(service.service_version(), "2.1.0");
}

#[service(name = "service1")]
pub struct Service1 {}

#[service(name = "service2")]
pub struct Service2 {}

#[test]
fn test_multiple_services() {
    let service1 = Service1 {};
    let service2 = Service2 {};
    
    // Verify each service has its own correct metadata
    assert_eq!(service1.name(), "service1");
    assert_eq!(service2.name(), "service2");
    
    // Make sure services don't interfere with each other
    assert_ne!(service1.name(), service2.name());
    assert_ne!(service1.path(), service2.path());
} 