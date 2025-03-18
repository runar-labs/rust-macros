use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Lit};
use syn::meta::ParseNestedMeta;

// Helper function to extract the value from a meta item
fn extract_value_from_meta(meta: &ParseNestedMeta) -> Option<String> {
    let path = meta.path.clone();
    let path_str = path.get_ident()?.to_string();
    
    let value = match meta.value() {
        Ok(value) => value,
        Err(_) => return None,
    };
    
    let lit: Lit = match value.parse() {
        Ok(lit) => lit,
        Err(_) => return None,
    };
    
    match lit {
        Lit::Str(s) => Some(s.value()),
        _ => None,
    }
}

pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Extract function name and arguments
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    
    // Default values
    let mut name = fn_name.to_string();
    let mut path = format!("/{}", name);
    
    // Parse arguments
    let args_parser = |meta: ParseNestedMeta| {
        if meta.path.is_ident("name") {
            if let Some(value_str) = extract_value_from_meta(&meta) {
                name = value_str;
            }
            return Ok(());
        }
        
        if meta.path.is_ident("path") {
            if let Some(value_str) = extract_value_from_meta(&meta) {
                path = value_str;
            }
            return Ok(());
        }
        
        Err(meta.error("Unknown attribute"))
    };
    
    // Try to parse the arguments, but don't fail if we can't
    if let Ok(meta) = syn::parse2::<syn::Meta>(args.clone().into()) {
        if let syn::Meta::List(list) = meta {
            let _ = list.parse_nested_meta(args_parser);
        }
    }
    
    // Generate the middleware registration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#fn_args) #fn_body
        
        #[cfg_attr(feature = "linkme", distributed_slice(MIDDLEWARE_REGISTRY))]
        #[cfg_attr(not(feature = "linkme"), allow(non_upper_case_globals))]
        static __MIDDLEWARE: fn() -> ::std::boxed::Box<dyn ::std::any::Any> = || {
            #[cfg(feature = "kagi_node")]
            {
                use kagi_node::services::Middleware;
                let middleware = Middleware::new(#name, #path, |ctx, next| {
                    Box::pin(async move {
                        #fn_name(ctx, next).await
                    })
                });
                ::std::boxed::Box::new(middleware)
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