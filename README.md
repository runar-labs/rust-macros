# Kagi Macros

This crate provides procedural macros for the Kagi Node system, making it easier to define services, actions, and handle events.

> **Development Status**: The macros are fully functional and support both compile-time and runtime registration approaches. All tests now pass, including comprehensive end-to-end tests that validate the entire service lifecycle.

## Implementation Approaches

Kagi macros support two implementation approaches:

1. **Distributed Slices (Compile-time)**: Using the `linkme` crate to register handlers, actions, and subscriptions at compile time. This approach is more efficient but requires the unstable `#[used(linker)]` attribute.

2. **Runtime Registration (Default)**: A fallback mechanism that registers handlers, actions, and subscriptions at runtime. This approach is used when the `distributed_slice` feature is not enabled, making it compatible with stable Rust and testing environments.

> **Testing**: The runtime registration approach enables testing without requiring unstable Rust features. Simply run your tests without enabling the `linkme` feature, and macros will automatically use runtime registration.

## Available Macros

### Service Macro
The `service` macro is used to define a Kagi service by implementing the `AbstractService` trait.

```rust
#[service(
    name = "example_service", // required
    // Optional parameters with defaults:
    // path = "example_service", (defaults to name value)
    // description = "ExampleService service", (defaults to struct name + "service")
    // version = "1.0.0" (defaults to "1.0.0")
)]
struct ExampleService {
    // Service fields
}
```

### Action Macro
The `action` macro designates a method as a service action that can be invoked through the node's request system.

```rust
#[action]
async fn my_action(&self, param: String) -> Result<ServiceResponse> {
    // Handle the action
    Ok(ServiceResponse::success("Action completed", None))
}
```

### Action Macro (Generic)
The `action` macro can also be used without a specific name to implement generic request handling logic that delegates to appropriate action methods.

```rust
#[action]
async fn process_request(&self, context: &RequestContext, operation: &str, params: &ValueType) -> Result<ServiceResponse> {
    match operation {
        "my_action" => self.my_action(params["param"].as_str().unwrap_or_default().to_string()).await,
        _ => Ok(ServiceResponse::error(format!("Unknown operation: {}", operation), None)),
    }
}
```

### Event Macros
The `publish` and `subscribe` macros simplify event-based communication.

```rust
// Subscribe to events
#[subscribe("example_topic")]
async fn handle_event(&mut self, context: &RequestContext, payload: ValueType) -> Result<()> {
    // Handle event
    Ok(())
}

// Publish events
#[publish("example_topic")]
async fn publish_event(event: EventData) -> Result<()> {
    // Event will be published to the topic
    Ok(())
}
```

## Testing with Macros

When writing tests that use macros, you don't need to enable the `distributed_slice` feature. The macros automatically use the runtime registration approach in test environments, ensuring your tests can run without requiring unstable Rust features.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use kagi_macros::{service, action};
    use kagi_node::test_utils::TestNode;
    
    #[service(name = "test_service")]
    struct TestService {}
    
    impl TestService {
        #[action]
        async fn test_action(&self) -> Result<ServiceResponse> {
            // Test implementation
            Ok(ServiceResponse::success("Test successful", None))
        }
    }
    
    #[tokio::test]
    async fn test_service_macro() {
        let mut node = TestNode::new();
        let service = TestService {};
        
        // The service and action are registered at runtime
        node.register_service(service).await.unwrap();
        
        // Test the service
        let response = node.request("test_service/test_action", json!({})).await.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }
}
```

## Basic Example

```rust
use kagi_macros::{service, action, process};
use kagi_node::services::{ServiceResponse, RequestContext, ValueType};
use anyhow::Result;

#[service(
    name = "counter_service",
    path = "counter",
    description = "A simple counter service"
)]
struct CounterService {
    value: std::sync::atomic::AtomicU64,
}

impl CounterService {
    fn new() -> Self {
        Self {
            value: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    #[action]
    async fn increment(&self) -> Result<ServiceResponse> {
        let value = self.value.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(ServiceResponse::success("Counter incremented", Some(ValueType::Number((value + 1) as f64))))
    }
    
    #[action]
    async fn process_request(&self, context: &RequestContext, operation: &str, params: &ValueType) -> Result<ServiceResponse> {
        match operation {
            "increment" => self.increment().await,
            _ => Ok(ServiceResponse::error(format!("Unknown operation: {}", operation), None)),
        }
    }
}
```

## Documentation

For more detailed documentation and advanced examples, see the [Macros Documentation](../docs/development/macros.md).