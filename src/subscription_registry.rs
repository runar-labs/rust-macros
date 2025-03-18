use std::future::Future;
use std::pin::Pin;
use anyhow::Result;

/// Represents a subscription handler registered by the subscribe macro
pub struct SubscriptionHandler {
    /// Name of the method that handles this subscription
    pub method_name: &'static str,
    
    /// Topic name (may be relative or absolute)
    pub topic: &'static str,
    
    /// Whether the topic is a full path
    pub is_full_path: bool,
    
    /// Function to register this subscription
    pub register_fn: fn(&dyn std::any::Any, &kagi_node::services::RequestContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>,
}

// Use inventory crate to collect subscription handlers
inventory::collect!(SubscriptionHandler);

/// Provides access to all registered subscription handlers
pub fn get_subscription_handlers() -> Vec<&'static SubscriptionHandler> {
    inventory::iter::<SubscriptionHandler>
        .into_iter()
        .collect()
}

/// Helper to register all subscriptions for a service
/// 
/// This function is called during service initialization to register
/// all subscription handlers defined with the subscribe macro.
/// 
/// # Parameters
/// 
/// - `service`: The service instance to register subscriptions for
/// - `ctx`: The request context for subscription registration
/// 
/// # Returns
/// 
/// `Ok(())` if all subscriptions were registered successfully, or an error
pub async fn register_all_subscriptions<S>(service: &S, ctx: &kagi_node::services::RequestContext) -> Result<()>
where
    S: Clone + std::any::Any + 'static,
{
    let mut errors = Vec::new();
    
    for handler in get_subscription_handlers() {
        // Call the register function directly with the service and context
        match (handler.register_fn)(service, ctx).await {
            Ok(_) => {}, // Subscription registered successfully
            Err(e) => {
                // Collect errors but continue registering other subscriptions
                errors.push(format!("Failed to register subscription for handler '{}' on topic '{}': {}", 
                    handler.method_name, handler.topic, e));
            }
        }
    }
    
    // If there were any errors, return an error with all the failures
    if !errors.is_empty() {
        Err(anyhow::anyhow!("Failed to register {} subscription(s):\n{}", 
            errors.len(), errors.join("\n")))
    } else {
        Ok(())
    }
} 