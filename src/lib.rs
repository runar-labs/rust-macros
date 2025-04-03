use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, ItemStruct, parse_str, Attribute, Ident, LitStr, Token, punctuated::Punctuated, Meta, MetaNameValue};
use quote::{quote, format_ident};
use std::collections::HashMap;
use std::cell::RefCell;

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
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(item as ItemStruct);
    
    // Extract service metadata from attributes
    let attr_string = attr.to_string();
    let attr_meta = parse_attributes(&attr_string);
    
    // Extract the struct name
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();
    
    // Get service metadata with defaults
    let service_name = attr_meta.get("name")
        .cloned()
        .unwrap_or_else(|| to_snake_case(&struct_name_str));
    
    let service_path = attr_meta.get("path")
        .cloned()
        .unwrap_or_else(|| service_name.clone());
    
    let service_description = attr_meta.get("description")
        .cloned()
        .unwrap_or_else(|| format!("{} service", struct_name_str));
    
    let service_version = attr_meta.get("version")
        .cloned()
        .unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the implementation of AbstractService trait
    let expanded = quote! {
        // Original struct definition
        #input
        
        // Implement Clone if not already implemented
        #[cfg(not(any(test, doctest)))]
        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                Self {
                    ..self.clone()
                }
            }
        }
        
        // AbstractService trait implementation
        #[async_trait::async_trait]
        impl runar_node::services::abstract_service::AbstractService for #struct_name {
            fn name(&self) -> &str {
                #service_name
            }
            
            fn path(&self) -> &str {
                #service_path
            }
            
            fn description(&self) -> &str {
                #service_description
            }
            
            fn version(&self) -> &str {
                #service_version
            }
            
            fn state(&self) -> runar_node::services::abstract_service::ServiceState {
                runar_node::services::abstract_service::ServiceState::Running
            }
            
            fn actions(&self) -> Vec<runar_node::services::abstract_service::ActionMetadata> {
                // Return the list of registered actions
                self.action_registry.keys()
                    .map(|name| runar_node::services::abstract_service::ActionMetadata {
                        name: name.clone(),
                    })
                    .collect()
            }
            
            async fn init(&mut self, context: &runar_node::RequestContext) -> anyhow::Result<()> {
                // Register subscriptions if the method exists
                if let Some(register_subscriptions) = self.register_subscriptions() {
                    register_subscriptions(self, context).await?;
                }
                
                Ok(())
            }
            
            async fn start(&mut self) -> anyhow::Result<()> {
                // Start the service
                Ok(())
            }
            
            async fn stop(&mut self) -> anyhow::Result<()> {
                // Stop the service
                Ok(())
            }
            
            async fn handle_request(&self, request: runar_node::services::ServiceRequest) -> anyhow::Result<runar_node::services::ServiceResponse> {
                // Get the operation
                let operation = &request.operation;
                
                // Check if the operation exists in the action registry
                if let Some(handler) = self.action_registry.get(operation) {
                    // Call the handler with the request
                    handler(request).await
                } else {
                    // Return an error if the operation is not found
                    Ok(runar_node::services::ServiceResponse {
                        status: runar_node::services::ResponseStatus::Error,
                        message: format!("Unknown operation: {}", operation),
                        data: None,
                    })
                }
            }
        }
    };
    
    // Return the expanded code
    expanded.into()
}

// Helper function to convert CamelCase to snake_case
fn to_snake_case(camel_case: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in camel_case.char_indices() {
        if i > 0 && c.is_uppercase() {
            snake_case.push('_');
        }
        snake_case.push(c.to_lowercase().next().unwrap());
    }
    snake_case
}

// Helper function to parse attributes from a string
fn parse_attributes(attr_string: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    // Simple parsing for name = "value" pairs
    for pair in attr_string.split(',') {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();
            
            // Remove quotes if present
            if value.starts_with('"') && value.ends_with('"') {
                value = value[1..value.len()-1].to_string();
            }
            
            result.insert(key, value);
        }
    }
    
    result
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
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input = parse_macro_input!(item as ItemFn);
    
    // Extract action metadata from attributes
    let attr_string = attr.to_string();
    let attr_meta = parse_attributes(&attr_string);
    
    // Extract the function name
    let func_name = &input.sig.ident;
    let func_name_str = func_name.to_string();
    
    // Get action name with default
    let action_name = attr_meta.get("name")
        .cloned()
        .unwrap_or_else(|| func_name_str.clone());
    
    // Extract function parameters
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();
    
    for input in &input.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                param_names.push(param_name);
                
                let param_type = &pat_type.ty;
                param_types.push(param_type);
            }
        }
    }
    
    // Generate handler name based on function name
    let handler_name = format_ident!("handle_{}", func_name);
    let register_fn_name = format_ident!("register_{}", func_name);
    
    // Generate the action handler method and registration
    let expanded = quote! {
        // Original function definition
        #input
        
        // Handler method for the action
        async fn #handler_name(&self, request: runar_node::services::ServiceRequest) -> anyhow::Result<runar_node::services::ServiceResponse> {
            use runar_common::types::ValueType;
            use runar_node::services::{ResponseStatus, ServiceResponse};
            
            // Extract parameters from the request data
            #(
                // Extract parameter from request data
                // Handle both direct values and map-based parameters
            )*
            
            // Call the original method
            match self.#func_name().await {
                Ok(result) => {
                    // Return success response
                    Ok(ServiceResponse {
                        status: ResponseStatus::Success,
                        message: format!("Operation {} completed successfully", #action_name),
                        data: Some(ValueType::from(result)),
                    })
                },
                Err(e) => {
                    // Return error response
                    Ok(ServiceResponse {
                        status: ResponseStatus::Error,
                        message: format!("Error in {}: {}", #action_name, e),
                        data: None,
                    })
                }
            }
        }
        
        // Register this action in the action registry
        #[inline]
        fn #register_fn_name(&mut self) {
            use std::future::Future;
            use std::pin::Pin;
            use std::sync::Arc;
            use runar_node::services::{ServiceRequest, ServiceResponse};
            
            // Create a type alias for our action handler function
            type ActionHandlerFn = Arc<dyn Fn(ServiceRequest) -> Pin<Box<dyn Future<Output = anyhow::Result<ServiceResponse>> + Send>> + Send + Sync>;
            
            // Create the handler function
            let self_clone = self.clone();
            let handler: ActionHandlerFn = Arc::new(move |request: ServiceRequest| -> Pin<Box<dyn Future<Output = anyhow::Result<ServiceResponse>> + Send>> {
                let self_clone = self_clone.clone();
                Box::pin(async move {
                    self_clone.#handler_name(request).await
                })
            });
            
            // Register the handler in the action registry
            self.action_registry.insert(#action_name.to_string(), handler);
        }
    };
    
    // Return the expanded code
    expanded.into()
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
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input = parse_macro_input!(item as ItemFn);
    
    // Extract subscription metadata from attributes
    let attr_string = attr.to_string();
    let attr_meta = parse_attributes(&attr_string);
    
    // Extract the function name
    let func_name = &input.sig.ident;
    let func_name_str = func_name.to_string();
    
    // Get topic with default
    let topic = attr_meta.get("topic")
        .cloned()
        .unwrap_or_else(|| func_name_str.clone());
    
    // Generate register function name based on function name
    let register_fn_name = format_ident!("register_{}_subscription", func_name);
    
    // Generate the subscription registration method
    let expanded = quote! {
        // Original function definition
        #input
        
        // Register this subscription
        fn #register_fn_name(&self, context: &runar_node::RequestContext) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
            use std::sync::Arc;
            use runar_common::types::ValueType;
            use runar_common::utils::logging::{warn_log, Component};
            use anyhow::anyhow;
            
            // Create an Arc of self for thread-safe sharing
            let service = Arc::new(self.clone());
            let topic_str = #topic.to_string();
            
            async move {
                // Subscribe to the event using an async callback
                context.subscribe(topic_str, move |payload: ValueType| {
                    // Create a clone for each async callback invocation
                    let service = service.clone();
                    
                    // Return a future that will be properly awaited by the system
                    async move {
                        // Call the original method with proper error propagation
                        if let Err(e) = service.#func_name(payload).await {
                            // Error will be properly seen and logged by the system
                            warn_log(Component::Service, &format!("Error handling event: {}", e)).await;
                            return Err(anyhow!("Failed to handle event: {}", e));
                        }
                        
                        Ok(())
                    }
                }).await
            }
        }
    };
    
    // Return the expanded code
    expanded.into()
}

/// Macro for defining a subscription handler (alias for subscribe)
///
/// Alias for the subscribe macro.
#[proc_macro_attribute]
pub fn sub(args: TokenStream, input: TokenStream) -> TokenStream {
    subscribe(args, input)
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