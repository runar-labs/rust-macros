use std::future::Future;
use std::pin::Pin;
use anyhow::Result;

/// Handler function type for action handlers
pub type ActionHandlerFn = fn(
    service: &dyn std::any::Any,
    context: &kagi_node::services::RequestContext,
    params: kagi_node::services::ValueType,
) -> Pin<Box<dyn Future<Output = Result<kagi_node::services::ServiceResponse>> + Send>>;

/// Represents an action handler registered by the action macro
pub struct ActionHandler {
    /// Name of the method that handles this action
    pub method_name: &'static str,
    
    /// Operation name for Node API requests
    pub operation_name: &'static str,
    
    /// Function to handle this action
    pub handler_fn: ActionHandlerFn,
}

// Use inventory crate to collect action handlers
inventory::collect!(ActionHandler);

/// Provides access to all registered action handlers
pub fn get_action_handlers() -> Vec<&'static ActionHandler> {
    inventory::iter::<ActionHandler>
        .into_iter()
        .collect()
}

/// Find an action handler by operation name
pub fn find_action_handler(operation_name: &str) -> Option<&'static ActionHandler> {
    get_action_handlers()
        .into_iter()
        .find(|handler| handler.operation_name == operation_name)
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
    context: &kagi_node::services::RequestContext,
    operation: &str,
    params: kagi_node::services::ValueType,
) -> Result<kagi_node::services::ServiceResponse>
where
    S: std::any::Any + 'static,
{
    // Find the handler for this operation
    let handler = find_action_handler(operation)
        .ok_or_else(|| anyhow::anyhow!("Unknown operation: {}", operation))?;
    
    // Call the handler function directly with the service, context, and params
    (handler.handler_fn)(service, context, params).await.map_err(|e| {
        anyhow::anyhow!("Error handling operation '{}': {}", operation, e)
    })
} 