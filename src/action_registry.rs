use std::future::Future;
use std::pin::Pin;
use anyhow::Result;

/// Function signature for an action handler
pub type ActionFn = fn(
    service: &dyn std::any::Any,
    context: &runar_node::services::RequestContext,
    path: &str,
    params: runar_node::ValueType,
) -> Pin<Box<dyn Future<Output = Result<runar_node::services::ServiceResponse>> + Send>>;

/// Represents an action handler registered by the action macro
#[derive(Debug)]
pub struct ActionItem {
    /// Name of the action
    pub name: String,
    /// Service type ID for this action
    pub service_type_id: std::any::TypeId,
    /// Function to handle this action
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

/// Find an action handler by operation name
pub fn find_action_handler(operation_name: &str) -> Option<&'static ActionItem> {
    get_action_handlers()
        .into_iter()
        .find(|handler| handler.name == operation_name)
}

/// Dispatch a request to the appropriate action handler
/// 
/// This function is used by the service macro to handle requests.
/// It finds the appropriate action handler by operation name and calls it.
/// 
/// # Parameters
/// 
/// - `service`: The service instance to dispatch the request to
/// - `context`: The request context containing metadata and callbacks
/// - `operation`: The operation name to find the handler for
/// - `params`: The parameters for the operation (may be a direct value or a map)
/// 
/// # Returns
/// 
/// A `ServiceResponse` with the result of the operation
pub async fn dispatch_request<S>(
    service: &S,
    context: &runar_node::services::RequestContext,
    operation: &str,
    params: runar_node::ValueType,
) -> Result<runar_node::services::ServiceResponse>
where
    S: std::any::Any + 'static,
{
    // Find the handler for this operation
    let handler = find_action_handler(operation)
        .ok_or_else(|| anyhow::anyhow!("Unknown operation: {}", operation))?;
    
    // Call the handler function directly with the service, context, and params
    (handler.handler_fn)(service, context, operation, params).await.map_err(|e| {
        anyhow::anyhow!("Error handling operation '{}': {}", operation, e)
    })
}

/// Interface for action handler implementation
pub trait ActionHandlerFn<S> {
    async fn call(
        &self,
        service: &S,
        context: &runar_node::services::RequestContext,
        path: &str,
        params: runar_node::ValueType,
    ) -> Result<runar_node::services::ServiceResponse>;
} 