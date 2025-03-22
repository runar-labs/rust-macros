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
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    
    // For simplicity, just output the original struct for now
    let output = quote! {
        #input_struct
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