use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use runar_node::services::{RequestContext, ServiceResponse};
use serde::de::DeserializeOwned;
use inventory;

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
    pub service_type_id: TypeId,
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

/// Registry item for action handlers
/// 
/// This struct is used to register action handlers with the inventory system.
/// Each entry maps a service type ID and operation name to an action handler.
#[derive(Debug)]
pub struct OperationRegistry {
    /// Type ID of the service that owns this action
    pub type_id: TypeId,
    /// Name of the operation
    pub operation_name: String,
    /// Name of the handler method for this operation
    pub handler_name: String,
}

inventory::collect!(OperationRegistry);

/// Registry item for service operations
///
/// This struct is used to register operations with the service's operations list.
/// Each entry provides the operation name for a service type.
#[derive(Debug)]
pub struct ServiceOperations {
    /// Type ID of the service that owns this operation
    pub type_id: TypeId,
    /// Name of the operation
    pub operation: String,
}

inventory::collect!(ServiceOperations);

/// Registry item for subscription handlers
///
/// This struct is used to register subscription handlers with the inventory system.
/// Each entry maps a service type ID and topic to a subscription handler registration method.
#[derive(Debug)]
pub struct SubscriptionRegistry {
    /// Type ID of the service that owns this subscription
    pub type_id: TypeId,
    /// The event topic to subscribe to
    pub topic: String,
    /// Whether the topic is a full path (contains '/') or a relative path
    pub is_full_path: bool,
    /// Name of the registration method for this subscription
    pub registration_method: String,
}

inventory::collect!(SubscriptionRegistry);

/// Stub trait to implement for type registration
///
/// This trait is used to register types with the inventory system.
/// It has no methods and is only used for its TypeId.
pub trait TypeRegistration {} 