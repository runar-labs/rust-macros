use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod action;
mod events;
mod gateway;
mod init;
mod middleware;
mod publish;
mod service;
mod subscribe;
mod utils;
mod subscription_registry;
mod action_registry;

// Structure to represent the arguments to the service macro
struct MacroArgs {
    name: Option<String>,
    path: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut path = None;
        let mut description = None;
        let mut version = None;
        
        // If no arguments, just return default
        if input.is_empty() {
            return Ok(MacroArgs {
                name,
                path,
                description,
                version,
            });
        }
        
        // Parse named arguments with format: name = "value", path = "value", etc.
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            
            // Expect an equals sign
            let _: syn::Token![=] = input.parse()?;
            
            // Parse the value as a string
            let value: syn::LitStr = input.parse()?;
            let value_str = value.value();
            
            // Set the appropriate field
            match key_str.as_str() {
                "name" => {
                    name = Some(value_str);
                },
                "path" => {
                    path = Some(value_str);
                },
                "description" => {
                    description = Some(value_str);
                },
                "version" => {
                    version = Some(value_str);
                },
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}. Expected one of: name, path, description, version", key_str)
                    ));
                }
            }
            
            // If there's a comma, consume it
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        
        Ok(MacroArgs {
            name,
            path,
            description,
            version,
        })
    }
}

/// Service macro for defining KAGI services
///
/// This macro generates all the necessary implementations for a service to work
/// with the KAGI node system, including:
/// - AbstractService trait implementation 
/// - Service metadata methods
/// - Request handling with action dispatching
/// - Event subscription setup
///
/// # Examples
///
/// ```rust
/// #[service(
///     name = "data",
///     path = "data_processor",
///     description = "Processes and transforms data", 
///     version = "1.0.0"
/// )]
/// struct DataProcessorService {
///     counter: i32,
/// }
/// ```
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::service(attr, item)
}

/// Action macro for defining service operations
///
/// This macro marks methods as operations that can be invoked through the Node API.
/// It handles parameter extraction and result conversion.
///
/// # Examples
///
/// ```rust
/// #[action(name = "transform")]
/// async fn transform(&self, context: &RequestContext, data: &str) -> Result<String> {
///     // Implementation...
///     Ok(transformed)
/// }
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::action(attr, item)
}

/// Subscribe macro for defining event handlers
///
/// This macro marks methods as event subscription handlers that will receive 
/// events published on the specified topic.
///
/// # Examples
///
/// ```rust
/// #[subscribe(topic = "data_events")]
/// async fn handle_event(&mut self, payload: ValueType) -> Result<()> {
///     // Handle the event...
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    subscribe::subscribe(attr, item)
}

/// Publish macro for defining event publishing methods
///
/// This macro marks methods that publish events to the specified topic.
///
/// # Examples
///
/// ```rust
/// #[publish(topic = "data_events")]
/// async fn notify_data_change(&self, context: &RequestContext, data: &str) -> Result<()> {
///     // Implementation that publishes an event
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn publish(attr: TokenStream, item: TokenStream) -> TokenStream {
    publish::publish(attr, item)
}

/// Init macro for custom service initialization
///
/// This macro can be applied to methods that need to run during service initialization.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    init::init(attr, item)
}

/// Middleware macro for request processing
///
/// This macro can be applied to methods that implement middleware functionality.
#[proc_macro_attribute]
pub fn middleware(attr: TokenStream, item: TokenStream) -> TokenStream {
    middleware::middleware(attr, item)
}

/// Gateway macro for API gateways
///
/// This macro can be applied to structs that serve as API gateways.
#[proc_macro_attribute]
pub fn gateway(attr: TokenStream, item: TokenStream) -> TokenStream {
    gateway::rest_api(attr, item)
}

/// Route macro for API endpoints
///
/// This macro can be applied to methods that handle specific API routes.
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    gateway::route(attr, item)
}