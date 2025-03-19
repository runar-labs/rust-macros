use std::future::Future;
use std::pin::Pin;
use anyhow::Result;
use runar_common::types::ValueType;
use runar_node::services::{RequestContext, ServiceResponse};

/// Type for action handler functions
pub type ActionFn = fn(
    &dyn std::any::Any,
    &RequestContext,
    &str,
    ValueType,
) -> Pin<Box<dyn Future<Output = Result<ServiceResponse>> + Send>>;

/// Item used to register an action handler
pub struct ActionItem {
    /// Name of the action
    pub name: String,
    /// Service type ID for downcasting
    pub service_type_id: std::any::TypeId,
    /// Handler function
    pub handler_fn: ActionFn,
}

// Use inventory crate to collect action handlers
inventory::collect!(ActionItem);

/// Provides access to all registered action handlers
pub fn get_action_handlers() -> Vec<&'static ActionItem> {
    inventory::iter::<ActionItem>
        .into_iter()
        .collect()
}

/// Dispatch a request to the correct action handler
pub async fn dispatch_request(
    service: &dyn std::any::Any,
    context: &RequestContext,
    operation: &str,
    params: ValueType,
) -> Result<ServiceResponse> {
    // Find a handler for this operation and service type
    let type_id = service.type_id();
    let handlers = get_action_handlers();
    
    for handler in handlers {
        if handler.service_type_id == type_id && handler.name == operation {
            // Found a matching handler - call it
            return (handler.handler_fn)(service, context, operation, params).await;
        }
    }
    
    // No handler found
    Err(anyhow::anyhow!("No handler found for operation: {}", operation))
}

/// Trait for action handler functions
pub trait ActionHandlerFn<S> {
    /// Invoke the action handler with service and parameters
    async fn invoke(
        &self,
        service: &S,
        ctx: &RequestContext,
        params: ValueType,
    ) -> Result<ServiceResponse>;
} 