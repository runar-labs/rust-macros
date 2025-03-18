# Solutions for Testing Macros with Distributed Slices

This document outlines potential solutions to enable testing of macros that depend on `linkme` distributed slices.

## ✅ Solution Implemented: Dynamic Registration Feature

We have successfully implemented Solution 4 (Dynamic Registration Feature) as outlined below. This solution is now working in the codebase and all tests are passing.

The implementation:
1. Utilizes the runtime registration mechanism when the `distributed_slice` feature is not enabled
2. Properly handles service, action, process, and event registrations
3. Works effectively in testing environments without requiring unstable Rust features

To use this solution, simply run tests without enabling the `linkme` feature:

```sh
cargo test -p kagi_macros --test end_to_end_test --features=node_implementation -- --nocapture
```

## Problem Analysis

The core issue is that the macros (`#[action]`, `#[subscribe]`, etc.) depend on distributed slices declared with `linkme`, but these slices either:

1. Aren't properly activated in test environments due to feature flags
2. Cannot be properly located or resolved by the compiler in the test context
3. Don't have a fallback mechanism for tests to use instead of the distributed slice pattern

## Solution 1: Use the `linkme` crate

The `linkme` crate provides a way to create distributed slice patterns in Rust. This is a good solution for our use case.

### Implementation

1. Add the `linkme` crate as a dependency:

```toml
[dependencies]
linkme = "0.3"
```

2. Define a distributed slice in the `kagi_node` crate:

```rust
#[distributed_slice]
pub static INITIALIZERS: [fn(&InitializerRegistry)] = [..];
```

3. In the `kagi_macros` crate, generate code that adds to this slice:

```rust
#[cfg(feature = "node_implementation")]
#[linkme::distributed_slice(kagi_node::init::INITIALIZERS)]
fn register_my_service(registry: &kagi_node::init::InitializerRegistry) {
    registry.register_initializer(
        "my_service",
        Box::new(|node| {
            Box::pin(async move {
                let service = MyService::new();
                node.register_service(service).await?;
                Ok(())
            })
        }),
    );
}
```

4. In the `kagi_node` crate, iterate over the slice to call all initializers:

```rust
pub fn initialize_all(registry: &InitializerRegistry) {
    for initializer in INITIALIZERS {
        initializer(registry);
    }
}
```

### Testing

To test this solution, run:

```bash
cargo test -p kagi_macros --test end_to_end_test --features=node_implementation -- --nocapture
```

## Solution 2: Use the `inventory` crate

The `inventory` crate is similar to `linkme` but with a different API.

### Implementation

1. Add the `inventory` crate as a dependency:

```toml
[dependencies]
inventory = "0.3"
```

2. Define a collector in the `kagi_node` crate:

```rust
inventory::collect!(InitializerFn);

pub struct InitializerFn(pub fn(&InitializerRegistry));
```

3. In the `kagi_macros` crate, generate code that submits to this collector:

```rust
#[cfg(feature = "node_implementation")]
inventory::submit! {
    kagi_node::init::InitializerFn(|registry: &kagi_node::init::InitializerRegistry| {
        registry.register_initializer(
            "my_service",
            Box::new(|node| {
                Box::pin(async move {
                    let service = MyService::new();
                    node.register_service(service).await?;
                    Ok(())
                })
            }),
        );
    })
}
```

4. In the `kagi_node` crate, iterate over the collector to call all initializers:

```rust
pub fn initialize_all(registry: &InitializerRegistry) {
    for initializer in inventory::iter::<InitializerFn> {
        (initializer.0)(registry);
    }
}
```

## Solution 3: Create a `TestNode` Approach

Implement a `TestNode` struct that mimics the behavior of the regular `Node` but doesn't rely on distributed slices:

```rust
pub struct TestNode {
    services: HashMap<String, Box<dyn AbstractService>>,
    subscriptions: HashMap<String, Vec<EventCallback>>,
}

impl TestNode {
    pub fn register_service(&mut self, service: Box<dyn AbstractService>) {
        self.services.insert(service.name().to_string(), service);
    }
    
    pub fn register_subscription(&mut self, topic: &str, callback: EventCallback) {
        self.subscriptions.entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }
    
    // Additional methods for testing pub/sub functionality
}
```

## Solution 4: Dynamic Registration Feature (✅ IMPLEMENTED)

Add a "dynamic registration" feature that doesn't use distributed slices but provides similar functionality through runtime registration:

```rust
// In kagi_node::init
pub static INITIALIZERS: RwLock<Vec<Initializer>> = RwLock::new(Vec::new());

pub async fn register_initializer(initializer: Initializer) {
    INITIALIZERS.write().await.push(initializer);
}

// This is already implemented in the init.rs macro
#[cfg(feature = "node_implementation")]
kagi_node::init::register_initializer(/* ... */);
```

Enable this feature for all tests:

```toml
# In Cargo.toml
[dev-dependencies]
kagi_node = { path = "../node", features = ["node_implementation"] }
```

## Solution 5: Hybrid Macro Pattern

Redesign the macros to work efficiently with both distributed slices and without them by using a hybrid approach:

```rust
// In action macro
#[cfg(feature = "node_implementation")]
#[distributed_slice(ACTION_REGISTRY)]
static #static_ident: fn() -> kagi_node::services::ActionHandler = || {
    kagi_node::services::ActionHandler { /* ... */ }
};

// No need for dummy implementation - register directly with the service
#[cfg(not(feature = "node_implementation"))]
impl<T: ActionHandlerService> AddAction<T> for #impl_type {
    fn add_action(&mut self, action_name: &str) {
        self.register_action(
            kagi_node::services::ActionHandler { /* ... */ }
        );
    }
}
```

Then update the `service` macro to handle registration of actions, processes, and subscriptions.

## Solution 6: Testing Without Macros

This is a pragmatic solution that's already being used - have separate test files:

1. Integration tests that don't use macros for full system testing
2. Unit tests that focus on individual component functionality
3. Macro tests that verify the code generation but don't execute the full systems

## Recommendation

Solution 4 (Dynamic Registration Feature) has been successfully implemented and is now working in the codebase. This approach leverages the existing fallback mechanism in the macros and provides a robust solution for testing macros without requiring unstable Rust features.