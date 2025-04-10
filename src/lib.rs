use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, ItemStruct, punctuated::Punctuated};
use quote::{quote, format_ident};
use std::collections::HashMap;

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
    
    // Create a new struct with action_registry field
    let mut struct_fields = match &input.fields {
        syn::Fields::Named(fields) => fields.clone(),
        _ => syn::FieldsNamed {
            brace_token: syn::token::Brace::default(),
            named: Punctuated::new(),
        },
    };
    
    // Add the action_registry field
    let action_registry_field: syn::Field = syn::parse_quote! {
        action_registry: ::std::sync::Arc<::std::collections::HashMap<String, ::std::sync::Arc<dyn Fn(::runar_node::services::ServiceRequest) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::anyhow::Result<::runar_node::services::ServiceResponse>> + Send>> + Send + Sync>>>
    };
    
    struct_fields.named.push(action_registry_field);
    
    // Generate the AbstractService implementation
    let abstract_service_impl = quote! {
        #[::async_trait::async_trait]
        impl ::runar_node::services::abstract_service::AbstractService for #struct_name {
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
            
            fn state(&self) -> ::runar_node::services::abstract_service::ServiceState {
                ::runar_node::services::abstract_service::ServiceState::Running
            }
            
            fn actions(&self) -> ::std::vec::Vec<::runar_node::services::abstract_service::ActionMetadata> {
                self.action_registry.keys()
                    .map(|name| ::runar_node::services::abstract_service::ActionMetadata {
                        name: name.clone(),
                    })
                    .collect()
            }
            
            async fn init(&mut self, context: &::runar_node::RequestContext) -> ::anyhow::Result<()> {
                // Register all subscriptions
                self.register_subscriptions(context).await?;
                ::anyhow::Result::Ok(())
            }
            
            async fn start(&mut self) -> ::anyhow::Result<()> {
                ::anyhow::Result::Ok(())
            }
            
            async fn stop(&mut self) -> ::anyhow::Result<()> {
                ::anyhow::Result::Ok(())
            }
            
            async fn handle_request(&self, request: ::runar_node::services::ServiceRequest) -> ::anyhow::Result<::runar_node::services::ServiceResponse> {
                // Look up the action in the registry
                if let ::std::option::Option::Some(handler) = self.action_registry.get(&request.action) {
                    // Call the handler
                    return handler(request).await;
                }
                
                // If we get here, the action wasn't found
                ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                    status: ::runar_node::services::ResponseStatus::Error,
                    message: format!("Unknown action: {}", request.action),
                    data: ::std::option::Option::None,
                })
            }
        }
    };
    
    // Generate the new method implementation
    let new_impl = quote! {
        impl #struct_name {
            pub fn new() -> Self {
                Self {
                    action_registry: ::std::sync::Arc::new(::std::collections::HashMap::new()),
                    // Additional fields would be initialized here if needed
                }
            }
            
            // Default implementation for registering subscriptions
            async fn register_subscriptions(&self, _context: &::runar_node::RequestContext) -> ::anyhow::Result<()> {
                // This will be overridden by the subscribe macro as needed
                ::anyhow::Result::Ok(())
            }
        }
    };
    
    // Create the new fields with named struct fields
    let new_fields = syn::Fields::Named(struct_fields);
    
    // Create a new struct with the added fields
    let new_struct = syn::ItemStruct {
        attrs: input.attrs,
        vis: input.vis,
        struct_token: input.struct_token,
        ident: input.ident,
        generics: input.generics,
        fields: new_fields,
        semi_token: input.semi_token,
    };
    
    // Combine all the generated code
    let expanded = quote! {
        #new_struct
        
        #abstract_service_impl
        
        #new_impl
    };
    
    expanded.into()
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
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    
    // Extract attributes
    let attr_string = attr.to_string();
    let attr_meta = parse_attributes(&attr_string);
    
    // Get action name from attributes or use function name
    let action_name = attr_meta.get("name")
        .cloned()
        .unwrap_or_else(|| fn_name_str.clone());
    
    // Get visibility
    let vis = &input.vis;
    
    // Get struct name from the self parameter
    let self_ty = match input.sig.inputs.first() {
        Some(syn::FnArg::Receiver(receiver)) => {
            // Receiver with &self, &mut self
            match &receiver.reference {
                Some(_) => {
                    // This is a &self or &mut self receiver
                    // We need to extract the struct name from the enclosing impl block
                    // For simplicity, we'll use the function name's prefix as the struct name
                    let impl_name = fn_name_str.split('_').next().unwrap_or("Service");
                    format_ident!("{}", impl_name)
                },
                None => {
                    // This is a self receiver (no &)
                    format_ident!("Self")
                }
            }
        },
        _ => {
            // Not a method with self parameter
            panic!("Action must be a method with a self parameter");
        }
    };
    
    // Process function parameters
    let mut params = Vec::new();
    
    // Skip the first parameter (self)
    for param in input.sig.inputs.iter().skip(1) {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                let param_type = &pat_type.ty;
                
                params.push((param_name.clone(), param_type.clone()));
            }
        }
    }
    
    // Generate the parameter extraction code
    let param_extraction = if params.len() == 1 {
        // Single parameter - direct extraction
        let (param_name, param_type) = &params[0];
        
        quote! {
            // Extract the parameter - accept direct parameter of appropriate type or a map with this key
            let #param_name = match &request.data {
                ::std::option::Option::Some(::runar_common::types::ValueType::String(s)) => {
                    s.clone()
                },
                ::std::option::Option::Some(::runar_common::types::ValueType::Map(m)) => {
                    match m.get(stringify!(#param_name)) {
                        ::std::option::Option::Some(::runar_common::types::ValueType::String(s)) => {
                            s.clone()
                        },
                        _ => {
                            return ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                                status: ::runar_node::services::ResponseStatus::Error,
                                message: format!("Missing or invalid '{}' parameter", stringify!(#param_name)),
                                data: ::std::option::Option::None,
                            });
                        }
                    }
                },
                _ => {
                    return ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                        status: ::runar_node::services::ResponseStatus::Error,
                        message: "This action only accepts a direct string parameter".to_string(),
                        data: ::std::option::Option::None,
                    });
                }
            };
        }
    } else if params.len() > 1 {
        // Multiple parameters - must be a map
        let param_extractions = params.iter().map(|(param_name, _)| {
            quote! {
                let #param_name = match params.get(stringify!(#param_name)) {
                    ::std::option::Option::Some(::runar_common::types::ValueType::String(s)) => {
                        s.clone()
                    },
                    _ => {
                        return ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                            status: ::runar_node::services::ResponseStatus::Error,
                            message: format!("Missing or invalid '{}' parameter", stringify!(#param_name)),
                            data: ::std::option::Option::None,
                        });
                    }
                };
            }
        });
        
        quote! {
            // Extract parameters - must be a map with appropriate keys
            let params = match &request.data {
                ::std::option::Option::Some(::runar_common::types::ValueType::Map(m)) => {
                    m
                },
                _ => {
                    return ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                        status: ::runar_node::services::ResponseStatus::Error,
                        message: format!("Parameters must be provided as a map with appropriate keys"),
                        data: ::std::option::Option::None,
                    });
                }
            };
            
            // Extract each parameter
            #(#param_extractions)*
        }
    } else {
        // No parameters
        quote! {}
    };
    
    // Generate parameter arguments for the original function call
    let param_args = params.iter().map(|(param_name, _)| {
        quote! { #param_name }
    });
    
    // Generate the handler function
    let handler_fn_name = format_ident!("handle_{}", fn_name);
    
    let handler_fn = quote! {
        async fn #handler_fn_name(&self, request: ::runar_node::services::ServiceRequest) -> ::anyhow::Result<::runar_node::services::ServiceResponse> {
            #param_extraction
            
            // Call the original method
            match self.#fn_name(#(#param_args),*).await {
                ::std::result::Result::Ok(result) => {
                    // Return the response
                    ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                        status: ::runar_node::services::ResponseStatus::Success,
                        message: format!("Operation {} executed successfully", #action_name),
                        data: ::std::option::Option::Some(::runar_common::types::ValueType::String(result)),
                    })
                },
                ::std::result::Result::Err(e) => {
                    ::anyhow::Result::Ok(::runar_node::services::ServiceResponse {
                        status: ::runar_node::services::ResponseStatus::Error,
                        message: format!("Error executing operation {}: {}", #action_name, e),
                        data: ::std::option::Option::None,
                    })
                }
            }
        }
    };
    
    // Generate the registration for this action
    let register_fn_name = format_ident!("register_{}_action", fn_name);
    
    let register_fn = quote! {
        #vis fn #register_fn_name(&self) -> ::std::collections::HashMap<String, ::std::sync::Arc<dyn Fn(::runar_node::services::ServiceRequest) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::anyhow::Result<::runar_node::services::ServiceResponse>> + Send>> + Send + Sync>> {
            let mut registry = ::std::collections::HashMap::new();
            
            let self_clone = self.clone();
            let handler = ::std::sync::Arc::new(move |request: ::runar_node::services::ServiceRequest| -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::anyhow::Result<::runar_node::services::ServiceResponse>> + Send>> {
                let self_clone = self_clone.clone();
                ::std::boxed::Box::pin(async move {
                    self_clone.#handler_fn_name(request).await
                })
            });
            
            registry.insert(#action_name.to_string(), handler);
            registry
        }
    };
    
    // Combine the original function, handler function, and registration function
    let expanded = quote! {
        #input
        
        impl #self_ty {
            #handler_fn
            
            #register_fn
        }
    };
    
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
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    
    // Parse the attributes to get the topic
    let attr_string = attr.to_string();
    let topic = if attr_string.contains("topic") {
        // Extract the topic from topic = "..."
        let start = attr_string.find("\"").unwrap_or(0) + 1;
        let end = attr_string.rfind("\"").unwrap_or(attr_string.len());
        attr_string[start..end].to_string()
    } else {
        // Default topic based on function name
        format!("events/{}", fn_name_str)
    };
    
    // Get visibility
    let vis = &input.vis;
    
    // Get struct name from the self parameter
    let self_ty = match input.sig.inputs.first() {
        Some(syn::FnArg::Receiver(receiver)) => {
            // Receiver with &self, &mut self
            match &receiver.reference {
                Some(_) => {
                    // This is a &self or &mut self receiver
                    // We need to extract the struct name from the enclosing impl block
                    // For simplicity, we'll use the function name's prefix as the struct name
                    let impl_name = fn_name_str.split('_').next().unwrap_or("Service");
                    format_ident!("{}", impl_name)
                },
                None => {
                    // This is a self receiver (no &)
                    format_ident!("Self")
                }
            }
        },
        _ => {
            // Not a method with self parameter
            panic!("Subscribe must be a method with a self parameter");
        }
    };
    
    // Process function parameters - should only have one parameter for the payload
    let mut params = Vec::new();
    
    // Skip the first parameter (self)
    for param in input.sig.inputs.iter().skip(1) {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                let param_type = &pat_type.ty;
                
                params.push((param_name.clone(), param_type.clone()));
            }
        }
    }
    
    // Check if we have exactly one parameter (the payload)
    if params.len() != 1 {
        panic!("Subscribe handlers must have exactly one parameter for the payload");
    }
    
    let (param_name, _) = &params[0];
    
    // Generate the registration function for this subscription
    let register_fn_name = format_ident!("register_{}_subscription", fn_name);
    
    let register_fn = quote! {
        #vis async fn #register_fn_name(&self, context: &::runar_node::RequestContext) -> ::anyhow::Result<()> {
            // Create a clone for the closure
            let service = ::std::sync::Arc::new(self.clone());
            
            // Subscribe to the topic
            context.subscribe(#topic, move |payload: ::runar_common::types::ValueType| {
                // Create a clone for each invocation
                let service = service.clone();
                
                // Return a future
                async move {
                    // Extract the payload parameter
                    let #param_name = match payload {
                        ::runar_common::types::ValueType::String(s) => s,
                        _ => {
                            return ::anyhow::Result::Err(::anyhow::anyhow!(
                                "Event payload must be a string"
                            ));
                        }
                    };
                    
                    // Call the handler method
                    if let ::std::result::Result::Err(e) = service.#fn_name(#param_name).await {
                        return ::anyhow::Result::Err(::anyhow::anyhow!(
                            "Failed to handle subscription event: {}", e
                        ));
                    }
                    
                    ::anyhow::Result::Ok(())
                }
            }).await?;
            
            ::anyhow::Result::Ok(())
        }
    };
    
    // Combine the original function and registration function
    let expanded = quote! {
        #input
        
        impl #self_ty {
            #register_fn
            
            // Extend register_subscriptions to call this method
            async fn register_subscriptions(&self, context: &::runar_node::RequestContext) -> ::anyhow::Result<()> {
                // Register this subscription
                self.#register_fn_name(context).await?;
                
                ::anyhow::Result::Ok(())
            }
        }
    };
    
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