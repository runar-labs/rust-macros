use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, Path, Meta};

/// Resolves a crate name for use in macro-generated code.
/// This function attempts to find the crate with the given name in the current scope.
/// If the crate is not found, it returns the original name.
pub fn get_crate_name(name: &str) -> TokenStream {
    // For simplicity, we'll just return the crate name directly
    // In a more sophisticated implementation, this would check if the crate
    // is available in the current scope and handle aliasing
    let ident = Ident::new(name, proc_macro2::Span::call_site());
    quote! { #ident }
}

/// Parses a path string into a syn::Path
pub fn parse_path(path_str: &str) -> syn::Result<Path> {
    syn::parse_str::<Path>(path_str)
}

/// Extracts the last segment of a path
pub fn get_last_path_segment(path: &Path) -> Option<&Ident> {
    path.segments.last().map(|segment| &segment.ident)
}

/// Checks if a function is async
pub fn is_async_fn(item_fn: &syn::ItemFn) -> bool {
    item_fn.sig.asyncness.is_some()
}

/// Generates a unique identifier based on the input
pub fn generate_unique_ident(base: &str) -> Ident {
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    Ident::new(
        &format!("{}_{}", base, unique_suffix),
        proc_macro2::Span::call_site()
    )
}

/// Converts a string to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    
    for (i, c) in s.char_indices() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }
    
    result
}

/// Extract an attribute value as a string
pub fn get_attribute_value(attrs: &[Attribute], name: &str) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident(name) {
            continue;
        }
        
        // Parse the attribute
        let mut result = None;
        
        // Try to extract the value
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("value") {
                let _ = meta.parse_nested_meta(|nested| {
                    if let Ok(lit) = nested.value() {
                        if let Ok(s) = lit.parse::<syn::LitStr>() {
                            result = Some(s.value());
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        });
        
        if result.is_some() {
            return result;
        }
    }
    
    None
}

/// Parse a comma-separated string into a vector of strings
pub fn parse_comma_separated(s: &str) -> Vec<String> {
    s.split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

/// Extract a name-value pair from attribute arguments
pub fn get_name_value_from_args(args: &[syn::Meta]) -> Option<(String, String)> {
    for arg in args {
        if let Meta::NameValue(name_value) = arg {
            if let Some(ident) = name_value.path.get_ident() {
                if let syn::Expr::Lit(expr_lit) = &name_value.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        return Some((ident.to_string(), lit_str.value()));
                    }
                }
            }
        }
    }
    None
}

/// Extract a name from attribute arguments
pub fn get_name_from_args(args: &[syn::Meta]) -> Option<String> {
    for arg in args {
        if let Meta::Path(path) = arg {
            if let Some(ident) = path.get_ident() {
                return Some(ident.to_string());
            }
        }
    }
    None
}

/// Parse attribute meta data into name-value pairs
pub fn parse_meta_args(meta: &syn::Meta) -> Vec<syn::Meta> {
    match meta {
        syn::Meta::List(meta_list) => {
            let parsed_metas = meta_list.tokens.clone().into_iter()
                .filter_map(|token| {
                    if let proc_macro2::TokenTree::Group(group) = &token {
                        syn::parse2::<syn::Meta>(group.stream()).ok()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            
            parsed_metas
        }
        _ => Vec::new(),
    }
}

/// Extract name-value pairs from attribute arguments
pub fn extract_name_value_pairs(args: &[syn::Meta]) -> std::collections::HashMap<String, String> {
    let mut pairs = std::collections::HashMap::new();
    
    for meta in args {
        if let syn::Meta::NameValue(name_value) = meta {
            // Get the path as a string (the name part)
            let name = name_value.path.get_ident()
                .map(|ident| ident.to_string())
                .unwrap_or_default();
            
            // Get the value part
            if let syn::Expr::Lit(expr_lit) = &name_value.value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let value = lit_str.value();
                    pairs.insert(name, value);
                }
            }
        }
    }
    
    pairs
}

/// Check if a type is a string reference (&str)
pub fn is_str_reference(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(type_ref) = ty {
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            if type_path.path.segments.len() == 1 {
                return type_path.path.segments[0].ident == "str";
            }
        }
    }
    false
}

/// Checks if a type path is an i32
pub fn is_i32_type(type_path: &syn::TypePath) -> bool {
    if type_path.path.segments.len() == 1 {
        return type_path.path.segments[0].ident == "i32";
    }
    false
}

/// Checks if a type path is an f64
pub fn is_f64_type(type_path: &syn::TypePath) -> bool {
    if type_path.path.segments.len() == 1 {
        return type_path.path.segments[0].ident == "f64";
    }
    false
}

/// Checks if a type path is a bool
pub fn is_bool_type(type_path: &syn::TypePath) -> bool {
    if type_path.path.segments.len() == 1 {
        return type_path.path.segments[0].ident == "bool";
    }
    false
}

/// Checks if a type is a Result<ServiceResponse>
pub fn is_service_response_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Result" {
            if let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                if args.args.len() == 1 {
                    if let syn::GenericArgument::Type(syn::Type::Path(inner_path)) = &args.args[0] {
                        if inner_path.path.segments.len() == 1 {
                            return inner_path.path.segments[0].ident == "ServiceResponse";
                        }
                    }
                }
            }
        }
    }
    false
}

/// Helper function to extract value from a parameter map
pub fn extract_parameter<T, S>(
    params: &runar_common::types::ValueType,
    name: S,
    error_msg: &str
) -> anyhow::Result<T>
where
    S: AsRef<str>,
    T: TryFrom<runar_common::types::ValueType> + Default,
    T::Error: std::fmt::Debug,
{
    let name_ref = name.as_ref();
    match params {
        // If params is a Map, try to get the parameter by name
        runar_common::types::ValueType::Map(map) => {
            // Look for the parameter in the map
            if let Some(value) = map.get(name_ref) {
                // Try to convert the value to the requested type
                match T::try_from(value.clone()) {
                    Ok(converted) => Ok(converted),
                    Err(e) => Err(anyhow::anyhow!("Error converting parameter '{}': {:?}", name_ref, e)),
                }
            } else {
                // Parameter not found
                Err(anyhow::anyhow!("{}", error_msg))
            }
        },
        // For all other types, assume this is a direct value
        _ => {
            // Try to convert the value to the requested type
            match T::try_from(params.clone()) {
                Ok(converted) => Ok(converted),
                Err(e) => Err(anyhow::anyhow!("Error converting direct parameter: {:?}", e)),
            }
        }
    }
}

/// Alternative version of extract_parameter for use in non-macro context
pub fn extract_param<T>(
    params: &runar_common::types::ValueType,
    name: &str,
) -> anyhow::Result<T>
where
    T: TryFrom<runar_common::types::ValueType>,
    <T as TryFrom<runar_common::types::ValueType>>::Error: std::fmt::Display,
{
    match params {
        runar_common::types::ValueType::Map(map) => {
            if let Some(value) = map.get(name) {
                T::try_from(value.clone()).map_err(|err| {
                    anyhow::anyhow!("Failed to convert parameter '{}': {}", name, err)
                })
            } else {
                Err(anyhow::anyhow!("Parameter '{}' not found", name))
            }
        }
        _ => Err(anyhow::anyhow!("Parameters are not a map")),
    }
}

/// Convert any serializable value to ValueType
pub fn convert_to_value_type<T>(value: T) -> runar_common::types::ValueType
where
    T: serde::Serialize,
{
    // Convert to JSON first
    match serde_json::to_value(value) {
        Ok(json) => runar_common::types::ValueType::Json(json),
        Err(e) => {
            // Log conversion error
            runar_common::utils::logging::warn_log(
                runar_common::utils::logging::Component::Service, 
                &format!("Failed to convert value to ValueType: {}", e)
            );
            runar_common::types::ValueType::Null
        }
    }
}

/// Convert from ValueType to a specific type
pub fn convert_value_to_type<T>(value: runar_common::types::ValueType) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    match value {
        runar_common::types::ValueType::Json(json) => {
            serde_json::from_value(json)
                .map_err(|e| format!("Failed to convert value: {}", e))
        },
        _ => Err(format!("Cannot convert {:?} to the required type", value))
    }
} 