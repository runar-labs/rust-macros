// This file is kept empty as the service macro implementation
// has been moved to lib.rs to meet Rust's requirement that
// proc macros must be defined at the crate root. 
//
// NOTE: The service macro implementation has been simplified to only use AbstractService.
// Previously, both ServiceInfo and AbstractService were implemented separately, 
// but this created unnecessary duplication. Now, metadata methods (name, path,
// description, version) are directly implemented in AbstractService.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, parse::Parser, Meta};

/// Service macro for defining Runar services
///
/// This macro generates all the necessary implementations for a service to work
/// with the Runar node system. It handles:
/// - Service metadata (name, path, description, version)
/// - AbstractService trait implementation for Node integration
/// - Request handling and dispatching to action methods
/// - Event subscription setup
///
/// # Parameters
/// - `name`: The display name of the service (default: struct name in snake_case)
/// - `path`: The routing path for this service (default: name)
/// - `description`: Human-readable description (default: "{name} service")
/// - `version`: Version string (default: "0.1.0")
///
/// # Examples
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
///
/// // Implement a constructor
/// impl DataProcessorService {
///     pub fn new() -> Self {
///         Self { counter: 0 }
///     }
/// }
///
/// // Register with the Node
/// async fn main() -> anyhow::Result<()> {
///     let mut node = Node::new(config).await?;
///     let service = DataProcessorService::new();
///     node.add_service(service).await?;
///     node.start().await?;
///     Ok(())
/// }
/// ```
///
/// # Generated Code
/// The macro generates implementations for:
/// - AbstractService trait for Node integration
/// - Service metadata methods (name, path, description, version)
/// - Request dispatch to action handlers
/// - Event subscription setup
///
/// # Requirements
/// - Services with event subscriptions must implement `Clone`
/// - The service should be registered with the Node using `node.add_service()`
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the struct definition
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();
    
    // Convert to snake_case for default name
    let default_name = to_snake_case(&struct_name_str);
    
    // Parse the attribute tokens into a list of Meta items
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
    
    // Convert meta_list into a Vec<Meta>
    let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
    
    // Extract name-value pairs
    let attrs = crate::utils::extract_name_value_pairs(&meta_vec);
    
    // Extract service attributes
    let service_name = attrs.get("name").cloned().unwrap_or_else(|| default_name.clone());
    
    // Normalize the path - ensure it starts with a slash
    let raw_path = attrs.get("path").cloned().unwrap_or_else(|| service_name.clone());
    let service_path = if raw_path.starts_with('/') {
        raw_path
    } else {
        format!("/{}", raw_path)
    };
    
    let service_desc = attrs.get("description").cloned()
        .unwrap_or_else(|| format!("Service {}", struct_name_str));
    let service_version = attrs.get("version").cloned().unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the implementation
    let output = quote! {
        // Keep the original struct definition
        #input
        
        // Implement AbstractService trait
        #[async_trait::async_trait]
        impl runar_node::services::AbstractService for #struct_name {
            // Service metadata
            fn name(&self) -> &str {
                #service_name
            }
            
            fn path(&self) -> &str {
                #service_path
            }
            
            fn description(&self) -> &str {
                #service_desc
            }
            
            fn version(&self) -> &str {
                #service_version
            }
            
            fn state(&self) -> runar_node::services::ServiceState {
                runar_node::services::ServiceState::Initialized
            }
            
            fn metadata(&self) -> runar_node::services::ServiceMetadata {
                runar_node::services::ServiceMetadata::new()
            }
            
            // Service initialization - sets up event subscriptions
            async fn init(&mut self, context: &runar_node::services::RequestContext) -> anyhow::Result<()> {
                // Register all event subscriptions
                self.setup_subscriptions(context).await?;
                Ok(())
            }
            
            // Service start
            async fn start(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            // Service stop
            async fn stop(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            // Handle requests from the Node API
            async fn handle_request(
                &self,
                request: runar_node::services::ServiceRequest,
            ) -> anyhow::Result<runar_node::services::ServiceResponse> {
                // Extract operation and parameters
                let operation = &request.operation;
                let params = request.params.unwrap_or(runar_common::types::ValueType::Null);
                let context = &request.context;
                
                // Use the action registry to dispatch to the correct handler
                let service_ref: &dyn std::any::Any = self;
                let handlers = crate::action_registry::get_action_handlers();
                let type_id = std::any::TypeId::of::<#struct_name>();
                
                // Find a handler for this operation
                for handler in handlers {
                    if handler.service_type_id == type_id && handler.name == operation {
                        // Found a matching handler - call it
                        return (handler.handler_fn)(service_ref, context, operation, params).await
                            .map_err(|e| anyhow::anyhow!("Error in {}.{}: {}", #service_name, operation, e));
                    }
                }
                
                // No handler found, return error
                Err(anyhow::anyhow!("Unknown operation: {}.{}", #service_name, operation))
            }
        }
        
        // Add subscription setup method
        impl #struct_name {
            async fn setup_subscriptions(&self, context: &runar_node::services::RequestContext) -> anyhow::Result<()> {
                // Register all event subscriptions defined with the subscribe macro
                let handlers = crate::subscription_registry::get_subscription_handlers();
                let service_ref: &dyn std::any::Any = self;
                
                // Call the registration function for each handler that matches our type
                for handler in handlers {
                    // Call the registration function
                    (handler.register_fn)(service_ref, context).await?;
                }
                
                Ok(())
            }
        }
    };
    
    TokenStream::from(output)
}

/// Convert a CamelCase string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    
    for (i, c) in s.char_indices() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    
    result
} 