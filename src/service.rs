// This file is kept empty as the service macro implementation
// has been moved to lib.rs to meet Rust's requirement that
// proc macros must be defined at the crate root. 

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, parse::Parser, Meta};

/// Service macro for defining KAGI services
///
/// This macro generates all the necessary implementations for a service to work
/// with the KAGI node system. It handles:
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
    let service_path = attrs.get("path").cloned().unwrap_or_else(|| service_name.clone());
    let service_desc = attrs.get("description").cloned()
        .unwrap_or_else(|| format!("{} service", service_name));
    let service_version = attrs.get("version").cloned().unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the implementation
    let output = quote! {
        // Keep the original struct definition
        #input
        
        // Implement AbstractService trait
        #[async_trait::async_trait]
        impl kagi_node::services::AbstractService for #struct_name {
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
            
            // Service initialization - sets up event subscriptions
            async fn init(&mut self, context: &kagi_node::services::RequestContext) -> anyhow::Result<()> {
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
                request: kagi_node::services::ServiceRequest,
            ) -> anyhow::Result<kagi_node::services::ServiceResponse> {
                // Extract operation and parameters
                let operation = &request.operation;
                let params = request.params.unwrap_or(kagi_node::services::ValueType::Null);
                
                // Use the action registry to dispatch to the correct handler
                crate::action_registry::dispatch_request(
                    self,
                    &request.context,
                    operation,
                    params,
                ).await.map_err(|e| {
                    // Add more context to errors
                    anyhow::anyhow!("Error in {}.{}: {}", #service_name, operation, e)
                })
            }
        }
        
        // Add subscription setup method
        impl #struct_name {
            async fn setup_subscriptions(&self, context: &kagi_node::services::RequestContext) -> anyhow::Result<()> {
                // Register all event subscriptions defined with the subscribe macro
                crate::subscription_registry::register_all_subscriptions(self, context).await
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