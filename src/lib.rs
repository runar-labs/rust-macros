use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
use quote::quote;

// Internal modules - private, not exported
mod action;
mod action_registry;
mod events;
mod gateway;
mod init;
mod middleware;
mod publish;
mod service;
mod subscribe;
mod subscription_registry;
mod utils;
mod registry;
mod vmap;

/// Macro for defining a service
///
/// This macro marks a struct as a Runar service, which can be registered with a Runar node.
/// It will implement necessary traits and set up the service with the provided metadata.
///
/// # Parameters
///
/// * `name` - The name of the service (default: struct name in snake_case)
/// * `path` - The service path (default: `/{name}`)
/// * `description` - A description of the service (default: empty string)
/// * `version` - The service version (default: "0.1.0")
///
/// # Example
///
/// ```
/// #[service(
///     name = "my_service",
///     description = "My example service",
///     version = "1.0.0"
/// )]
/// pub struct MyService {
///     // fields
/// }
/// ```
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    service::service(args, input)
}

/// Macro for defining an action handler
///
/// This macro marks methods as service operations that can be invoked through the Node API.
/// It handles parameter extraction and result conversion.
///
/// # Parameters
/// - `name`: The operation name that will be used in node.request() calls (default: method name)
///
/// # Examples
/// ```rust
/// // Named action
/// #[action(name = "get_user")]
/// async fn get_user_by_id(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
///     // Implementation
///     Ok(ServiceResponse::success("User found", Some(user_data)))
/// }
///
/// // Default name from method
/// #[action]
/// async fn get_posts(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
///     // Implementation
///     Ok(ServiceResponse::success("Posts retrieved", Some(posts_data)))
/// }
/// ```
#[proc_macro_attribute]
pub fn action(args: TokenStream, input: TokenStream) -> TokenStream {
    action::action(args, input)
}

/// Macro for defining a subscription handler
///
/// This macro marks methods as event subscription handlers and registers them with the Node's
/// event system during service initialization.
///
/// # Parameters
/// - `topic`: The event topic to subscribe to (default: method name)
///   - Can be a relative path (e.g., "user_created") which will be prefixed with service path
///   - Can be a full path (e.g., "users/user_created") which will be used as-is
///
/// # Examples
/// ```rust
/// // Subscribe to a specific topic (service path will be prefixed)
/// #[subscribe(topic = "user_created")]
/// async fn handle_user_created(&self, data: ValueType) -> Result<(), anyhow::Error> {
///     // Handler implementation
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    subscribe::subscribe(args, input)
}

/// Macro for defining a subscription handler (alias for subscribe)
///
/// Alias for the subscribe macro.
#[proc_macro_attribute]
pub fn sub(args: TokenStream, input: TokenStream) -> TokenStream {
    subscribe::subscribe(args, input)
}

/// Macro for publishing an event
#[proc_macro_attribute]
pub fn publish(args: TokenStream, input: TokenStream) -> TokenStream {
    publish::publish(args, input)
}

/// Macro for defining a gateway
#[proc_macro_attribute]
pub fn gateway(args: TokenStream, input: TokenStream) -> TokenStream {
    gateway::gateway(args, input)
}

/// Macro for defining a middleware
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    middleware::middleware(args, input)
}

/// Macro for defining an initialization handler
#[proc_macro_attribute]
pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    init::init(args, input)
}

/// Macro for defining event handlers (alias for subscribe)
#[proc_macro_attribute]
pub fn events(args: TokenStream, input: TokenStream) -> TokenStream {
    events::events(args, input)
}