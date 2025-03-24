// This file is kept empty as the service macro implementation
// has been moved to lib.rs to meet Rust's requirement that
// proc macros must be defined at the crate root. 
//
// NOTE: The service macro implementation has been simplified to only use AbstractService.
// Previously, both ServiceInfo and AbstractService were implemented separately, 
// but this created unnecessary duplication. Now, metadata methods (name, path,
// description, version) are directly implemented in AbstractService.

use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

/// Service macro for defining Runar node services.
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
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    
    // Get the struct name and ident
    let struct_name = &input_struct.ident;
    let struct_name_str = struct_name.to_string();
    
    // Parse attributes
    let attr_str = attr.to_string();
    
    // Extract service attributes with defaults
    let service_name = extract_attr_value(&attr_str, "name").unwrap_or_else(|| to_snake_case(&struct_name_str));
    let service_path = extract_attr_value(&attr_str, "path").unwrap_or_else(|| format!("/{}" ,service_name));
    let service_description = extract_attr_value(&attr_str, "description").unwrap_or_else(|| String::new());
    let service_version = extract_attr_value(&attr_str, "version").unwrap_or_else(|| String::from("0.1.0"));
    
    // Generate the AbstractService trait implementation
    // This implementation provides default behavior that can be overridden
    let output = quote! {
        #input_struct
        
        // Implement AbstractService trait with default behaviors
        // Services can override this implementation by manually implementing the trait
        #[async_trait::async_trait]
        impl runar_node::services::abstract_service::AbstractService for #struct_name {
            async fn init(&mut self, ctx: &runar_node::services::RequestContext) -> anyhow::Result<()> {
                Ok(())
            }
            
            async fn start(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            async fn stop(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            fn state(&self) -> runar_node::services::abstract_service::ServiceState {
                runar_node::services::abstract_service::ServiceState::Running
            }
            
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
            
            fn metadata(&self) -> runar_node::services::abstract_service::ServiceMetadata {
                runar_node::services::abstract_service::ServiceMetadata {
                    name: self.name().to_string(),
                    path: self.path().to_string(),
                    description: self.description().to_string(),
                    version: self.version().to_string(),
                    state: self.state(),
                    operations: self.operations(),
                }
            }
            
            fn operations(&self) -> Vec<String> {
                vec![]
            }
            
            async fn handle_request(&self, request: runar_node::ServiceRequest) -> anyhow::Result<runar_node::ServiceResponse> {
                // The action macros will override this method with a match statement that
                // delegates to the appropriate action handler methods
                // This implementation is just a fallback for when no action handlers are defined
                
                // The action macros will generate code that looks like:
                // match request.operation.as_str() {
                //     "action_name" => {
                //         let result = self.action_name(&request.request_context, params).await?;
                //         Ok(runar_node::ServiceResponse::success(result))
                //     },
                //     _ => { /* fall through to default error */ }
                // }
                
                // Following the architectural guidelines, we return a clear error message
                // that indicates the operation is not implemented if no handler is found
                Ok(runar_node::ServiceResponse::error(
                    format!("Operation not implemented: {}", request.operation)
                ))
            }
        }
    };
    
    output.into()
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

/// Extract attribute value from the attribute string
fn extract_attr_value(attr_str: &str, key: &str) -> Option<String> {
    // Look for key = "value" pattern
    let pattern = format!("{} = \"", key);
    
    if let Some(start_idx) = attr_str.find(&pattern) {
        let value_start = start_idx + pattern.len();
        if let Some(end_idx) = attr_str[value_start..].find('"') {
            return Some(attr_str[value_start..(value_start + end_idx)].to_string());
        }
    }
    
    None
}