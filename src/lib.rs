// Procedural macros for the Runar Node system
//
// This library provides macros for simplifying the implementation of
// services, actions, and events in the Runar Node system.

extern crate proc_macro;

mod service;
mod action;
mod subscribe;
mod publish;
mod utils;

use proc_macro::TokenStream;

/// Macro for implementing a Runar service
///
/// This macro simplifies the implementation of a service by:
/// 1. Automatically implementing the AbstractService trait
/// 2. Generating the necessary init, start, and stop methods
/// 3. Registering actions and events during initialization
///
/// # Example
///
/// ```rust
/// #[service]
/// struct MathService {
///     counter: Arc<Mutex<i32>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::service_macro(attr, item)
}

/// Macro for implementing a Runar action
///
/// This macro simplifies the implementation of an action by:
/// 1. Marking a method as an action to be registered during service initialization
/// 2. Generating the necessary handler code for parameter extraction and response formatting
///
/// # Example
///
/// ```rust
/// #[action]
/// async fn add(&self, a: f64, b: f64, ctx: &RequestContext) -> Result<f64> {
///     Ok(a + b)
/// }
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::action_macro(attr, item)
}

/// Macro for implementing a Runar event subscription
///
/// This macro simplifies the implementation of an event subscription by:
/// 1. Marking a method as an event handler to be registered during service initialization
/// 2. Generating the necessary handler code for parameter extraction
///
/// # Example
///
/// ```rust
/// #[subscribe(path="service/event_name")]
/// async fn on_event(&self, data: MyEventData, ctx: &EventContext) -> Result<()> {
///     // Handle the event
///     ctx.debug(format!("Event received: {}", data.message));
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    subscribe::subscribe_macro(attr, item)
}

/// Macro for automatically publishing the result of an action
///
/// This macro simplifies publishing the result of an action by:
/// 1. Automatically publishing the result to the specified topic
/// 2. Handling errors gracefully
///
/// # Example
///
/// ```rust
/// #[publish(path="math/result")]
/// #[action]
/// async fn add(&self, a: f64, b: f64, ctx: &RequestContext) -> Result<f64> {
///     // The result will be automatically published to "math/result"
///     Ok(a + b)
/// }
/// ```
#[proc_macro_attribute]
pub fn publish(attr: TokenStream, item: TokenStream) -> TokenStream {
    publish::publish_macro(attr, item)
}
