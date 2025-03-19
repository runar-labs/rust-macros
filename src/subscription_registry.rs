use std::future::Future;
use std::pin::Pin;
use anyhow::Result;
use async_trait::async_trait;

/// Handler function for a subscription
#[derive(Debug)]
pub struct SubscriptionHandler {
    /// Name of the method that handles this subscription
    pub method_name: String,
    /// Topic to subscribe to
    pub topic: String,
    /// Whether the topic is a full path or needs to be prefixed with service path
    pub is_full_path: bool,
    /// Function to register this subscription
    pub register_fn: fn(
        service: &dyn std::any::Any,
        ctx: &runar_node::services::RequestContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>,
}

// Use inventory crate to collect subscription handlers
inventory::collect!(SubscriptionHandler);

/// Provides access to all registered subscription handlers
pub fn get_subscription_handlers() -> Vec<&'static SubscriptionHandler> {
    inventory::iter::<SubscriptionHandler>
        .into_iter()
        .collect()
}

/// Item used to register a subscription
#[derive(Debug)]
pub struct SubscriptionItem {
    /// Topic to subscribe to
    pub topic: String,
    /// Service type ID
    pub service_type_id: std::any::TypeId,
}

/// Register all subscriptions for a service
pub async fn register_all_subscriptions<S>(
    service: &S,
    ctx: &runar_node::services::RequestContext,
) -> Result<()>
where
    S: std::any::Any + 'static,
{
    // Find all subscription handlers for this service
    for handler in get_subscription_handlers() {
        // Call the registration function for each handler
        (handler.register_fn)(service, ctx).await?;
    }
    
    Ok(())
} 