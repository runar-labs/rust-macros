// Procedural macros for the Runar Node system
//
// This library provides macros for simplifying the implementation of
// services, actions, and events in the Runar Node system.

extern crate proc_macro;

mod service;
mod action;
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
