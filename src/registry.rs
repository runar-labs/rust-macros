use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use runar_node::services::{RequestContext, ServiceResponse};
use serde::de::DeserializeOwned;

/// Type for action handler functions 
pub type ActionHandlerFn = Box<
    dyn Fn(
        &dyn Any,                 // Service reference
        &RequestContext,          // Request context
        &str,                     // Operation name
        serde_json::Value,        // Parameters
    ) -> Pin<Box<dyn Future<Output = Result<ServiceResponse>> + Send>> + Send + Sync,
>;

/// Type for subscription registration functions
pub type SubscriptionRegisterFn = Box<
    dyn Fn(
        &dyn Any,                 // Service reference
        &RequestContext,          // Request context
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync,
>;

/// Action registry item for a service method
pub struct ActionItem {
    pub name: String,
    pub service_type_id: TypeId,
    pub handler_fn: ActionHandlerFn,
}

/// Event subscription handler
pub struct SubscriptionHandler {
    pub method_name: String,
    pub topic: String,
    pub is_full_path: bool,
    pub register_fn: SubscriptionRegisterFn,
}

inventory::collect!(ActionItem);
inventory::collect!(SubscriptionHandler);

/// Get all registered action handlers
pub fn get_action_handlers() -> Vec<&'static ActionItem> {
    inventory::iter::<ActionItem>
        .into_iter()
        .collect::<Vec<_>>()
}

/// Get all registered subscription handlers
pub fn get_subscription_handlers() -> Vec<&'static SubscriptionHandler> {
    inventory::iter::<SubscriptionHandler>
        .into_iter()
        .collect::<Vec<_>>()
}

/// Extract a parameter from a value map
pub fn extract_parameter<T, S>(
    params: &serde_json::Value,
    name: S,
    error_msg: impl Into<String>,
) -> Result<T>
where
    T: DeserializeOwned,
    S: AsRef<str>,
{
    let name_str = name.as_ref();
    let error_message = error_msg.into();
    
    // First try to extract from an object if this is a map
    if let serde_json::Value::Object(map) = params {
        if let Some(value) = map.get(name_str) {
            return serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse parameter '{}': {}", name_str, e));
        }
    }
    
    // If not found in object or params is not an object, try converting the entire value
    match serde_json::from_value::<T>(params.clone()) {
        Ok(value) => Ok(value),
        Err(_) => Err(anyhow::anyhow!(error_message)),
    }
} 