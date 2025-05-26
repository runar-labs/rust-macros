// Service macro implementation
//
// This module implements the service macro, which simplifies the implementation
// of a Runar service by automatically implementing the AbstractService trait and
// handling action registration.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Type, TypePath, Ident, ImplItem, parse_quote};

/// Implementation of the service macro
pub fn service_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as an impl block
    let input = parse_macro_input!(item as ItemImpl);
    
    // Extract the type being implemented
    let struct_type = match &*input.self_ty {
        Type::Path(TypePath { path, .. }) => {
            path.segments.last().unwrap().ident.clone()
        }
        _ => panic!("Service macro can only be applied to struct implementations"),
    };

    // Collect action methods to register in the register_actions method
    let action_methods = collect_action_methods(&input);
    
    // Generate the service metadata wrapper implementation
    let service_metadata = generate_service_metadata(&struct_type);
    
    // Generate a constructor method if it doesn't exist
    let constructor = generate_constructor(&input);
    
    // Add the constructor to the impl block
    let mut modified_input = input.clone();
    if let Some(constructor_method) = constructor {
        modified_input.items.push(ImplItem::Fn(constructor_method));
    }

    // Generate the implementation of the AbstractService trait
    let abstract_service_impl = generate_abstract_service_impl(&struct_type, &action_methods);

    // Combine the original impl block with the generated code
    let expanded = quote! {
        // Add the service metadata implementation
        #service_metadata
        
        // Add the modified impl block
        #modified_input

        // Then add the AbstractService implementation
        #abstract_service_impl
    };

    expanded.into()
}

/// Collect methods marked with #[action] in the impl block
fn collect_action_methods(input: &ItemImpl) -> Vec<Ident> {
    let mut action_methods = Vec::new();
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("action") {
                    action_methods.push(method.sig.ident.clone());
                    break;
                }
            }
        }
    }
    
    action_methods
}



/// Generate the service metadata implementation for the given struct type
fn generate_service_metadata(struct_type: &Ident) -> TokenStream2 {
    let service_name = format!("{}_ServiceMetadata", struct_type);
    let service_metadata_ident = Ident::new(&service_name, struct_type.span());
    
    quote! {
        // Static state holder for service metadata
        struct #service_metadata_ident {
            name: std::sync::OnceLock<String>,
            path: std::sync::OnceLock<String>,
            version: std::sync::OnceLock<String>,
            description: std::sync::OnceLock<String>,
            network_id: std::sync::OnceLock<Option<String>>,
        }
        
        // Initialize the static metadata
        impl #service_metadata_ident {
            const fn new() -> Self {
                Self {
                    name: std::sync::OnceLock::new(),
                    path: std::sync::OnceLock::new(),
                    version: std::sync::OnceLock::new(),
                    description: std::sync::OnceLock::new(),
                    network_id: std::sync::OnceLock::new(),
                }
            }
        }
        
        // The static holder for service metadata
        static SERVICE_METADATA: #service_metadata_ident = #service_metadata_ident::new();
        
        // Extension methods for the service
        impl #struct_type {
            // Internal method to initialize metadata - called by constructor
            fn __init_metadata(name: &str, path: &str) {
                let _ = SERVICE_METADATA.name.get_or_init(|| name.to_string());
                let _ = SERVICE_METADATA.path.get_or_init(|| path.to_string());
                let _ = SERVICE_METADATA.version.get_or_init(|| "1.0.0".to_string());
                let _ = SERVICE_METADATA.description.get_or_init(|| "Generated service".to_string());
                let _ = SERVICE_METADATA.network_id.get_or_init(|| None);
            }
        }
    }
}

/// Generate a constructor method for the service if it doesn't exist
fn generate_constructor(input: &ItemImpl) -> Option<syn::ImplItemFn> {
    // Check if a constructor already exists
    let has_constructor = input.items.iter().any(|item| {
        if let ImplItem::Fn(method) = item {
            method.sig.ident == "new"
        } else {
            false
        }
    });

    if has_constructor {
        return None;
    }

    // Create a new constructor
    let constructor: syn::ImplItemFn = parse_quote! {
        pub fn new(name: &str, path: &str) -> Self {
            // Initialize service metadata
            Self::__init_metadata(name, path);
            
            // Create the service instance
            Self {
                counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    };

    Some(constructor)
}

/// Generate the implementation of the AbstractService trait
fn generate_abstract_service_impl(struct_type: &Ident, action_methods: &[Ident]) -> TokenStream2 {
    // Generate register_actions method calls
    let register_calls = action_methods.iter().map(|method_name| {
        let register_method = quote::format_ident!("register_action_{}", method_name);
        quote! {
            self.#register_method(&context).await?;
        }
    });

    // Generate the register_actions method
    let register_actions_method = quote! {
        impl #struct_type {
            async fn register_actions(&self, context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Register all the actions
                #(#register_calls)*
                
                Ok(())
            }
        }
    };

    // Generate the AbstractService trait implementation
    let trait_impl = quote! {
        #[async_trait::async_trait]
        impl runar_node::AbstractService for #struct_type {
            fn name(&self) -> &str {
                SERVICE_METADATA.name.get().unwrap()
            }

            fn version(&self) -> &str {
                SERVICE_METADATA.version.get().unwrap()
            }

            fn path(&self) -> &str {
                SERVICE_METADATA.path.get().unwrap()
            }

            fn description(&self) -> &str {
                SERVICE_METADATA.description.get().unwrap()
            }

            fn network_id(&self) -> Option<String> {
                SERVICE_METADATA.network_id.get().unwrap().clone()
            }

            async fn init(&self, context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Register all actions
                self.register_actions(context).await
            }

            async fn start(&self, _context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Default implementation does nothing
                Ok(())
            }

            async fn stop(&self, _context: runar_node::services::LifecycleContext) -> anyhow::Result<()> {
                // Default implementation does nothing
                Ok(())
            }
        }
    };
    
    // Also implement Clone for the service
    let clone_impl = quote! {
        impl Clone for #struct_type {
            fn clone(&self) -> Self {
                Self {
                    counter: self.counter.clone(),
                }
            }
        }
    };

    quote! {
        #register_actions_method

        #trait_impl
        
        #clone_impl
    }
}
