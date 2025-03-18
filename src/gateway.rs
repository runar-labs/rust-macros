use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem};

/// The gateway macro to be applied to the implementation for registering API routes.
pub fn gateway(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the implementation block
    let input = parse_macro_input!(input as ItemImpl);
    let ty = &input.self_ty;
    
    // Find methods with route attributes
    let mut routes = Vec::new();
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("route") {
                    let method_name = &method.sig.ident;
                    routes.push(quote! {
                        stringify!(#method_name)
                    });
                }
            }
        }
    }
    
    // Generate route registration
    let expanded = quote! {
        #input
        
        #[cfg_attr(feature = "linkme", distributed_slice(API_GATEWAY_REGISTRY))]
        #[cfg_attr(not(feature = "linkme"), allow(non_upper_case_globals))]
        static __API_GATEWAY: fn() -> ::std::boxed::Box<dyn ::std::any::Any> = || {
            #[cfg(feature = "kagi_node")]
            {
                use kagi_node::services::ApiGateway;
                let mut gateway = ApiGateway::new(stringify!(#ty));
                // Register all routes
                #(gateway.register_route(#routes));*
                ::std::boxed::Box::new(gateway)
            }
            
            #[cfg(not(feature = "kagi_node"))]
            {
                // Dummy implementation for testing
                ::std::boxed::Box::new(())
            }
        };
    };
    
    TokenStream::from(expanded)
}

/// The rest_api macro to be applied to the implementation for registering REST API endpoints.
pub fn rest_api(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the implementation block
    let input = parse_macro_input!(input as ItemImpl);
    let ty = &input.self_ty;
    
    // Find methods with route attributes
    let mut routes = Vec::new();
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("route") {
                    let method_name = &method.sig.ident;
                    routes.push(quote! {
                        stringify!(#method_name)
                    });
                }
            }
        }
    }
    
    // Generate route registration
    let expanded = quote! {
        #input
        
        #[cfg_attr(feature = "linkme", distributed_slice(REST_API_GATEWAY_REGISTRY))]
        #[cfg_attr(not(feature = "linkme"), allow(non_upper_case_globals))]
        static __REST_API_GATEWAY: fn() -> ::std::boxed::Box<dyn ::std::any::Any> = || {
            #[cfg(feature = "kagi_node")]
            {
                use kagi_node::services::RestApiGateway;
                let mut gateway = RestApiGateway::new(stringify!(#ty));
                // Register all routes
                #(gateway.register_route(#routes));*
                ::std::boxed::Box::new(gateway)
            }
            
            #[cfg(not(feature = "kagi_node"))]
            {
                // Dummy implementation for testing
                ::std::boxed::Box::new(())
            }
        };
    };
    
    TokenStream::from(expanded)
}

/// The route macro to be applied to the methods for marking them as API routes.
pub fn route(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
} 