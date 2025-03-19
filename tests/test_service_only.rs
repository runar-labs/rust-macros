mod common;
use common::ServiceInfo;
use runar_macros::service;

#[test]
fn test_service_only() {
    // Define a service with the service macro
    #[service(
        name = "test_service"
    )]
    struct TestService {
        counter: u32,
    }
    
    impl TestService {
        fn new() -> Self {
            Self { counter: 0 }
        }
    }
    
    // Create an instance of the service
    let service = TestService::new();
    
    // Verify the service name and path from the macro
    assert_eq!(service.service_name(), "test_service");
    assert_eq!(service.service_path(), "/test_service"); // Path is prefixed with /
    
    println!("Service definition test passed!");
}

#[test]
fn test_service_minimal() {
    // Define a service with minimal parameters
    #[service(name = "minimal_service")]
    struct MinimalService {
        counter: u32,
    }
    
    impl MinimalService {
        fn new() -> Self {
            Self { counter: 0 }
        }
    }
    
    // Create an instance of the service
    let service = MinimalService::new();
    
    // Verify the service name and path from the macro
    assert_eq!(service.service_name(), "minimal_service");
    assert_eq!(service.service_path(), "/minimal_service"); // Path is prefixed with /
    assert_eq!(service.service_description(), "Service MinimalService"); // Default description
    assert_eq!(service.service_version(), "0.1.0"); // Default version
}

#[test]
fn test_service_with_all_params() {
    // Define a service with all parameters specified
    #[service(
        name = "custom_service",
        path = "/api/v1/custom",
        description = "A fully customized service",
        version = "2.3.4"
    )]
    struct CustomService {
        value: String,
    }
    
    impl CustomService {
        fn new() -> Self {
            Self { value: "test".to_string() }
        }
    }
    
    // Create an instance of the service
    let service = CustomService::new();
    
    // Verify all service parameters
    assert_eq!(service.service_name(), "custom_service");
    assert_eq!(service.service_path(), "/api/v1/custom"); // Path is already prefixed with /
    assert_eq!(service.service_description(), "A fully customized service"); // Custom description
    assert_eq!(service.service_version(), "2.3.4"); // Custom version
}

#[test]
fn test_multiple_services() {
    // Test that multiple services can be defined
    #[service(name = "service1")]
    struct Service1 {}
    
    #[service(name = "service2", path = "/custom/path")]
    struct Service2 {}
    
    let service1 = Service1 {};
    let service2 = Service2 {};
    
    assert_eq!(service1.service_name(), "service1");
    assert_eq!(service1.service_path(), "/service1");
    
    assert_eq!(service2.service_name(), "service2");
    assert_eq!(service2.service_path(), "/custom/path");
} 