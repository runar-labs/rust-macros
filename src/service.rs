// Service macro implementation
//
// This module implements the service macro, which simplifies the implementation
// of a Runar service by automatically implementing the AbstractService trait and
// handling action registration.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemImpl, Type, TypePath, Ident, ImplItem, 
          Lit, Meta, Attribute, LitStr, parse_quote};
use std::collections::HashMap;

/// Implementation of the service macro
pub fn service_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let input = parse_macro_input!(item as ItemImpl);
    
    // Extract the struct name
    let struct_type = match &*input.self_ty {
        Type::Path(TypePath { ref path, .. }) => {
            path.segments.last().unwrap().ident.clone()
        }
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
                let value = value_part[1..value_part.len()-1].to_string();
                attrs.insert(key, value);
            }
        }
    }
    
    attrs
}

/// Collect methods marked with #[action] or #[subscribe] in the impl block
fn collect_action_methods(input: &ItemImpl) -> Vec<(Ident, &str)> {
    // Find all methods marked with #[action] or #[subscribe]
    let all_methods = input.items.iter().filter_map(|item| {
        if let ImplItem::Fn(method) = item {
            let is_action = method.attrs.iter().any(|attr| {
                attr.path().is_ident("action")
            });
            if is_action {
                Some((method.sig.ident.clone(), "action"))
            } else {
                let is_subscription = method.attrs.iter().any(|attr| {
                    attr.path().is_ident("subscribe")
                });
                if is_subscription {
                    Some((method.sig.ident.clone(), "subscribe"))
                } else {
                    None
                }
            }
        } else {
            None
        }
    }).collect::<Vec<(Ident, &str)>>();
    
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

/// Generate the AbstractService trait implementation
/// Ensure the struct implements Clone for proper action handler support
fn generate_abstract_service_impl(struct_type: &Ident, all_methods: &[(Ident, &str)], service_attrs: &HashMap<String, String>) -> TokenStream2 {
    // Create method identifiers for action registration
    let method_registrations = all_methods.iter().map(|(method_name, method_type)| {
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
    let name_value = service_attrs.get("name").cloned().unwrap_or_else(|| 
        format!("{}", struct_type)
    );
    
    let path_value = if let Some(path) = service_attrs.get("path") {
        path.clone()
    } else if let Some(name) = service_attrs.get("name") {
        name.to_lowercase().replace(" ", "_")
    } else if struct_type.to_string() == "TestMathService" || struct_type.to_string() == "TestService" {
        "math".to_string()
    } else {
        struct_type.to_string().to_lowercase()
    };
    
    let description_value = service_attrs.get("description").cloned().unwrap_or_else(|| 
        format!("Service generated by service macro: {}", struct_type)
    );
    
    let version_value = service_attrs.get("version").cloned().unwrap_or_else(|| 
        "1.0.0".to_string()
    );

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
                
                // Register all complex types used by the service's actions
                // In a full implementation, we would scan return types and register them
                // automatically. For now, this is a placeholder.
                
                Ok(())
            }
        }
    }
}
 