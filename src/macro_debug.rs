// This file is for debugging macro expansion
// See the generated code by running:
// cargo rustc --features=node_implementation -- -Z unstable-options --pretty=expanded

use kagi_node::services::{RequestContext, ServiceResponse, ValueType};
use kagi_node::anyhow::Result;
use kagi_macros::{service, action, subscribe};
use serde_json::json;

// Simple service for debugging macro expansion
#[service(name = "debug_service")]
struct DebugService {}

impl DebugService {
    // Action macro
    #[action(name = "test_action")]
    async fn test_action(&self, data: String) -> Result<ServiceResponse> {
        println!("Action called with: {}", data);
        Ok(ServiceResponse::success("Action complete", None))
    }
    
    // Action with context
    #[action(name = "test_action_with_context")]
    async fn test_with_context(&self, context: &RequestContext, message: String) -> Result<ServiceResponse> {
        println!("Action with context called: {}", message);
        Ok(ServiceResponse::success("Context action complete", None))
    }
    
    // Action macro for request handling
    #[action]
    async fn process_request(&self, context: &RequestContext, operation: &str, params: &ValueType) -> Result<ServiceResponse> {
        match operation {
            "test" => Ok(ServiceResponse::success("Test processed", None)),
            _ => Ok(ServiceResponse::error("Unknown operation", None)),
        }
    }
    
    // Subscribe macro
    #[subscribe(topic = "test/topic")]
    async fn on_test_event(&self, payload: ValueType) -> Result<()> {
        println!("Event received");
        Ok(())
    }
}

fn main() {
    println!("This file is for debugging the macro expansion");
} 