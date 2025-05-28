// Service macro implementation
//
// This module implements the service macro, which simplifies the implementation
// of a Runar service by automatically implementing the AbstractService trait and
// handling action registration.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::{
    parse_macro_input, parse_quote, Attribute, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Lit,
    LitStr, Meta, Pat, PatType, ReturnType, Type, TypePath,
};

/// Implementation of the service macro
pub fn service_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let input = parse_macro_input!(item as ItemImpl);

    // Extract the struct name
    let struct_type = match &*input.self_ty {
        Type::Path(TypePath { ref path, .. }) => path.segments.last().unwrap().ident.clone(),
        _ => panic!("Service macro can only be applied to structs"),
    };

    // Extract the service attributes from the macro annotation
    let service_attrs = extract_service_attributes(attr);

    // Find all methods marked with #[action] or #[subscribe]
    let all_methods = collect_action_methods(&input);

    // Generate the service metadata
    let service_metadata = generate_service_metadata();

    // Generate the trait implementation for the AbstractService trait
    let service_impl = generate_abstract_service_impl(&struct_type, &all_methods, &service_attrs);

    // Return the input struct unchanged along with the trait implementation
    TokenStream::from(quote! {
        #input

        #service_metadata

        #service_impl
    })
}

/// Extract service attributes from the TokenStream
fn extract_service_attributes(attr: TokenStream) -> HashMap<String, String> {
    let mut attrs = HashMap::new();

    if attr.is_empty() {
        return attrs;
    }

    // Convert attribute tokens to a string for simple parsing
    let attr_str = attr.to_string();

    // Simple parsing of name = "value" pairs
    for pair in attr_str.split(',') {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();

            // Extract the string value between quotes
            let value_part = parts[1].trim();
            if value_part.starts_with('"') && value_part.ends_with('"') {
                let value = value_part[1..value_part.len() - 1].to_string();
                attrs.insert(key, value);
            }
        }
    }

    attrs
}

/// Collect methods marked with #[action] or #[subscribe] in the impl block
fn collect_action_methods(input: &ItemImpl) -> Vec<(Ident, &str, ImplItemFn)> {
    // Find all methods marked with #[action] or #[subscribe]
    let all_methods = input
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                let is_action = method
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("action"));
                if is_action {
                    Some((method.sig.ident.clone(), "action", method.clone()))
                } else {
                    let is_subscription = method
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("subscribe"));
                    if is_subscription {
                        Some((method.sig.ident.clone(), "subscribe", method.clone()))
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        })
        .collect::<Vec<(Ident, &str, ImplItemFn)>>();

    all_methods
}

/// Generate the service metadata static holder
fn generate_service_metadata() -> TokenStream2 {
    quote! {
        // Static metadata holders
        static SERVICE_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        static SERVICE_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        static SERVICE_DESCRIPTION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        static SERVICE_VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    }
}

/// Extract types from a method's parameters and return type
fn extract_types_from_method(method: &ImplItemFn) -> Vec<String> {
    let mut types = Vec::new();

    // Extract parameter types
    for arg in &method.sig.inputs {
        if let FnArg::Typed(PatType { ty, pat, .. }) = arg {
            // Skip context parameter
            if let Pat::Ident(pat_ident) = &**pat {
                let param_name = pat_ident.ident.to_string();
                if param_name == "ctx" || param_name == "context" || param_name.ends_with("ctx") {
                    continue;
                }
            }

            // Get the type as a string
            let type_str = quote! { #ty }.to_string();
            types.push(type_str);
        }
    }

    // Extract return type
    if let ReturnType::Type(_, ty) = &method.sig.output {
        // Get the return type as a string
        let return_type_str = quote! { #ty }.to_string();

        // Clean up the string representation
        let clean_return_type = return_type_str
            .replace(" >", ">")
            .replace("< ", "<")
            .replace(" , ", ", ");

        // Check if it's a Result type
        if clean_return_type.starts_with("Result<") || clean_return_type.starts_with("Result <") {
            // Extract the content between the first < and the last >
            let start_idx = clean_return_type.find('<').unwrap_or(0) + 1;
            let end_idx = clean_return_type
                .rfind('>')
                .unwrap_or(clean_return_type.len());

            if start_idx < end_idx {
                let inner_content = &clean_return_type[start_idx..end_idx];

                // Split by the first comma to get the success type (T) and error type (E)
                if let Some(comma_idx) = inner_content.find(',') {
                    // Extract the success type (T)
                    let success_type = inner_content[..comma_idx].trim().to_string();
                    if !success_type.is_empty() && success_type != "()" {
                        types.push(success_type);
                    }

                    // Extract the error type (E) if it's not a standard error type
                    let error_type = inner_content[comma_idx + 1..].trim().to_string();
                    if !error_type.is_empty()
                        && error_type != "()"
                        && error_type != "E"
                        && !error_type.starts_with("anyhow::Error")
                        && !error_type.starts_with("anyhow :: Error")
                    {
                        types.push(error_type);
                    }
                } else {
                    // If there's no comma, just add the whole inner content
                    if !inner_content.is_empty() && inner_content != "()" {
                        types.push(inner_content.to_string());
                    }
                }
            }
        } else {
            // For non-Result types, just add the type directly
            types.push(clean_return_type);
        }
    }

    types
}

/// Format type string to be more readable and filter out standard types
fn format_type_string(type_str: &str) -> Option<String> {
    // Remove extra spaces that quote! adds
    let mut formatted = type_str
        .replace(" >", ">")
        .replace("< ", "<")
        .replace(" , ", ", ");

    // Remove references
    if formatted.starts_with("& ") {
        formatted = formatted[2..].to_string();
    }

    // Filter out standard types that don't need registration
    match formatted.as_str() {
        // Primitive types
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" | "bool" | "char" | "()" | "String" => None,

        // Standard library types that are handled by default
        s if s.starts_with("Vec<") && is_primitive_type(&s[4..s.len() - 1]) => None,
        s if s.starts_with("Option<") && is_primitive_type(&s[7..s.len() - 1]) => None,
        s if s.starts_with("HashMap<") => {
            // Only return None if both key and value are primitive types
            let inner = &s[8..s.len() - 1];
            if let Some(comma_idx) = inner.find(',') {
                let key_type = inner[..comma_idx].trim();
                let value_type = inner[comma_idx + 1..].trim();
                if is_primitive_type(key_type) && is_primitive_type(value_type) {
                    None
                } else {
                    Some(formatted)
                }
            } else {
                Some(formatted)
            }
        }

        // Keep all other types
        _ => Some(formatted),
    }
}

/// Check if a type is a primitive type
fn is_primitive_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "()"
            | "String"
    )
}

/// Generate the AbstractService trait implementation
/// Ensure the struct implements Clone for proper action handler support
fn generate_abstract_service_impl(
    struct_type: &Ident,
    all_methods: &[(Ident, &str, ImplItemFn)],
    service_attrs: &HashMap<String, String>,
) -> TokenStream2 {
    // Create method identifiers for action registration
    let method_registrations = all_methods.iter().map(|(method_name, method_type, _)| {
        if *method_type == "action" {
            let register_method_name = format_ident!("register_action_{}", method_name);
            quote! {
                self.#register_method_name(context_ref).await?;
            }
        } else {
            // Must be a subscription
            let register_method_name = format_ident!("register_subscription_{}", method_name);
            quote! {
                self.#register_method_name(context_ref).await?;
            }
        }
    });

    // Extract attribute values
    let name_value = service_attrs
        .get("name")
        .cloned()
        .unwrap_or_else(|| format!("{}", struct_type));

    // Derive path from attributes or struct name, following a consistent pattern
    let path_value = if let Some(path) = service_attrs.get("path") {
        // Explicit path has highest priority
        path.clone()
    } else if let Some(name) = service_attrs.get("name") {
        // Convert service name to path (lowercase, replace spaces with underscores)
        name.to_lowercase().replace(" ", "_")
    } else {
        // Default to lowercase struct name
        struct_type.to_string().to_lowercase()
    };

    let description_value = service_attrs
        .get("description")
        .cloned()
        .unwrap_or_else(|| format!("Service generated by service macro: {}", struct_type));

    let version_value = service_attrs
        .get("version")
        .cloned()
        .unwrap_or_else(|| "1.0.0".to_string());

    // Extract all types from methods
    let mut all_types = HashSet::new();

    for (_, _, method) in all_methods {
        let types = extract_types_from_method(method);
        for type_str in types {
            if let Some(formatted) = format_type_string(&type_str) {
                // Skip the service type itself
                if formatted != struct_type.to_string() {
                    all_types.insert(formatted);
                }
            }
        }
    }

    // Convert to a vector and sort for consistent output
    let mut sorted_types: Vec<_> = all_types.into_iter().collect();
    sorted_types.sort();

    // Create a string literal with all the types
    let types_str = sorted_types.join("\n");

    // Create type identifiers for each type
    let type_idents = sorted_types
        .iter()
        .map(|t| {
            // Parse the type string into a type path
            let type_path = syn::parse_str::<syn::TypePath>(t)
                .unwrap_or_else(|_| panic!("Failed to parse type: {}", t));
            type_path
        })
        .collect::<Vec<_>>();

    // Generate the code to log the types
    let type_collection_code = quote! {
        context.info(format!("Types used by service {}:\n    {}", stringify!(#struct_type), #types_str));
    };

    quote! {
        #[async_trait::async_trait]
        impl runar_node::services::abstract_service::AbstractService  for #struct_type {
            fn name(&self) -> &str {
                SERVICE_NAME.get_or_init(|| {
                    #name_value.to_string()
                })
            }

            fn path(&self) -> &str {
                SERVICE_PATH.get_or_init(|| {
                    #path_value.to_string()
                })
            }

            fn description(&self) -> &str {
                SERVICE_DESCRIPTION.get_or_init(|| {
                    #description_value.to_string()
                })
            }

            fn version(&self) -> &str {
                SERVICE_VERSION.get_or_init(|| {
                    #version_value.to_string()
                })
            }

            fn network_id(&self) -> Option<String> {
                None
            }

            async fn init(&self, context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Create a reference to the context
                let context_ref = &context;

                // Register all action and subscription methods defined with the #[action] or #[subscribe] macro
                #(#method_registrations)*

                // Register complex types with the serializer
                Self::register_types(context_ref).await?;

                Ok(())
            }

            async fn start(&self, _context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                Ok(())
            }

            async fn stop(&self, _context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                Ok(())
            }
        }

        // Helper method to register complex types with the serializer
        impl #struct_type {
            async fn register_types(context: &runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Acquire a write lock on the serializer
                let mut serializer = context.serializer.write().await;

                // Log all the collected types
                #type_collection_code

                // Register each type with the serializer
                #({
                    context.debug(format!("Registering type: {}", stringify!(#type_idents)));
                    serializer.register::<#type_idents>()?;
                })*

                Ok(())
            }
        }
    }
}
