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
        Lit::Int(i) => Some(i.base10_digits().to_string()),
        _ => None,
    }
}

pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Extract function name and arguments
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    
    // Default values
    let mut timeout_ms = 5000; // Default timeout of 5 seconds
    
    // Parse arguments
    let args_parser = |meta: ParseNestedMeta| {
        if meta.path.is_ident("timeout_ms") {
            if let Some(value_str) = extract_value_from_meta(&meta) {
                if let Ok(value) = value_str.parse::<u64>() {
                    timeout_ms = value;
                }
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
    
    // Generate the wrapped function
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#fn_args) -> anyhow::Result<()> {
            use tokio::time::timeout;
            use std::time::Duration;
            
            // Wrap the function body in a timeout
            match timeout(Duration::from_millis(#timeout_ms), async {
                #fn_body
            }).await {
                Ok(result) => result,
                Err(_) => Err(anyhow::anyhow!("Function timed out after {} ms", #timeout_ms)),
            }
        }
    };
    
    TokenStream::from(expanded)
} 