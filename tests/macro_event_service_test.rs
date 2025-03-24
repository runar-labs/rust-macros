use anyhow::Result;
use std::time::Duration;

// Import our macro-based services
mod fixtures;
use fixtures::services::{MacroEventEmitterService, MacroEventListenerService};

// Utility functions for testing macro-based services
async fn setup_macro_test_environment() -> Result<(
    runar_node::Node, 
    tempfile::TempDir, 
    MacroEventEmitterService, 
    MacroEventListenerService
)> {
    // Create temporary directory for node storage
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // Create node config
    let mut config = runar_node::NodeConfig::new();
    config.set_network_id("test_network".to_string());
    config.set_storage_path(temp_path.to_string());
    
    // Create and start the node
    let mut node = runar_node::Node::new(config)?;
    node.start().await?;
    
    // Create services
    let mut emitter_service = MacroEventEmitterService::new("test_emitter");
    let mut listener_service = MacroEventListenerService::new("test_listener");
    
    // Register services with the node
    node.add_service(emitter_service.clone()).await?;
    node.add_service(listener_service.clone()).await?;
    
    // Wait for services to be registered
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok((node, temp_dir, emitter_service, listener_service))
}

/// Test macro-based event services with direct topic paths
#[tokio::test]
async fn test_macro_event_services_direct_path() -> Result<()> {
    // Setup test environment
    let (node, _temp_dir, emitter, _listener) = setup_macro_test_environment().await?;
    
    // Wait for services to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create a valid event and publish it
    let request = runar_node::services::ServiceRequest {
        source: "test".to_string(),
        service: "event_emitter_service".to_string(),
        path: "process_valid".to_string(),
        data: Some(runar_node::services::types::ValueType::String("test_valid_event".to_string())),
        id: None,
        topic_path: Some(runar_node::routing::TopicPath::parse("event_emitter/process_valid", "test_network").unwrap()),
    };
    
    // Process the valid event request
    let result = node.request_router().process_service_request(request).await?;
    println!("Valid event processing result: {:?}", result);
    
    // Wait for event propagation
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check listener for received events
    let check_request = runar_node::services::ServiceRequest {
        source: "test".to_string(),
        service: "event_listener_service".to_string(),
        path: "get_events_by_topic".to_string(),
        data: Some(runar_node::services::types::ValueType::String("valid".to_string())),
        id: None,
        topic_path: Some(runar_node::routing::TopicPath::parse("event_listener/get_events_by_topic", "test_network").unwrap()),
    };
    
    let check_result = node.request_router().process_service_request(check_request).await?;
    println!("Received events check: {:?}", check_result);
    
    // Create an invalid event and publish it
    let invalid_request = runar_node::services::ServiceRequest {
        source: "test".to_string(),
        service: "event_emitter_service".to_string(),
        path: "process_invalid".to_string(),
        data: Some(runar_node::services::types::ValueType::String("test_invalid_event".to_string())),
        id: None,
        topic_path: Some(runar_node::routing::TopicPath::parse("event_emitter/process_invalid", "test_network").unwrap()),
    };
    
    // Process the invalid event request
    let invalid_result = node.request_router().process_service_request(invalid_request).await?;
    println!("Invalid event processing result: {:?}", invalid_result);
    
    // Wait for event propagation
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check listener for received events
    let invalid_check_request = runar_node::services::ServiceRequest {
        source: "test".to_string(),
        service: "event_listener_service".to_string(),
        path: "get_events_by_topic".to_string(),
        data: Some(runar_node::services::types::ValueType::String("invalid".to_string())),
        id: None,
        topic_path: Some(runar_node::routing::TopicPath::parse("event_listener/get_events_by_topic", "test_network").unwrap()),
    };
    
    let invalid_check_result = node.request_router().process_service_request(invalid_check_request).await?;
    println!("Received invalid events check: {:?}", invalid_check_result);
    
    // Check all events
    let all_events_request = runar_node::services::ServiceRequest {
        source: "test".to_string(),
        service: "event_listener_service".to_string(),
        path: "get_all_events".to_string(),
        data: None,
        id: None,
        topic_path: Some(runar_node::routing::TopicPath::parse("event_listener/get_all_events", "test_network").unwrap()),
    };
    
    let all_events_result = node.request_router().process_service_request(all_events_request).await?;
    println!("All events: {:?}", all_events_result);
    
    // Shutdown the node
    node.stop().await?;
    
    Ok(())
}
