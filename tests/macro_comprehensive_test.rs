mod common;
use common::ServiceInfo;
use runar_macros::{service, action, subscribe, publish};
use runar_node::services::{RequestContext, ServiceResponse, ValueType, ResponseStatus};
use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};

// Test service with multiple macros applied
#[service(
    name = "test_service",
    path = "test/service",
    description = "A comprehensive test service",
    version = "1.0.0"
)]
struct TestService {
    counter: AtomicU64,
    last_event: std::sync::RwLock<Option<String>>,
}

impl TestService {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            last_event: std::sync::RwLock::new(None),
        }
    }

    // Test action with custom name
    #[action(name = "custom_add")]
    async fn add(&self, _context: &RequestContext, a: i32, b: i32) -> Result<ServiceResponse> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(ServiceResponse::success(
            a + b,
            Some(ValueType::Object(serde_json::json!({
                "counter": count + 1
            })))
        ))
    }

    // Test action with default name
    #[action]
    async fn multiply(&self, _context: &RequestContext, a: i32, b: i32) -> Result<ServiceResponse> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(ServiceResponse::success(
            a * b,
            Some(ValueType::Object(serde_json::json!({
                "counter": count + 1
            })))
        ))
    }

    // Test action with publish
    #[action]
    #[publish(topic = "test/events")]
    async fn record_event(&self, _context: &RequestContext, event_name: String) -> Result<ServiceResponse> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        
        // Return success response with metadata that will be published as an event
        Ok(ServiceResponse::success(
            "Event recorded",
            Some(ValueType::Object(serde_json::json!({
                "event_name": event_name,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "counter": count + 1
            })))
        ))
    }

    // Test subscribe handler
    #[subscribe(topic = "test/events")]
    async fn handle_event(&self, payload: ValueType) -> Result<()> {
        if let ValueType::Object(obj) = payload {
            if let Some(event_name) = obj.get("event_name") {
                if let Some(name) = event_name.as_str() {
                    let mut last_event = self.last_event.write().unwrap();
                    *last_event = Some(name.to_string());
                    println!("Received event: {}", name);
                }
            }
        }
        
        Ok(())
    }

    // Helper method to get the last event
    pub fn get_last_event(&self) -> Option<String> {
        let last_event = self.last_event.read().unwrap();
        last_event.clone()
    }
}

#[test]
fn test_service_info() {
    // Test that service info is correctly implemented
    let service = TestService::new();
    
    assert_eq!(service.service_name(), "test_service");
    assert_eq!(service.service_path(), "test/service");
    assert_eq!(service.service_description(), "A comprehensive test service");
    assert_eq!(service.service_version(), "1.0.0");
}

#[test]
fn test_service_structure() {
    // This test just verifies that the service compiles correctly
    // with all macros applied
    let service = TestService::new();
    assert_eq!(service.counter.load(Ordering::SeqCst), 0);
}

// The actual runtime testing would require a full Node instance,
// which is beyond the scope of this test. Additional tests would
// be added in the integration test suite. 