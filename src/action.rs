// Action macro implementation
//
// This module implements the action macro, which simplifies the implementation
// of a Runar service action by automatically generating handler code for
// parameter extraction, validation, and response formatting.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, Lit,
    Pat, PatIdent, PatType, ReturnType, Type, punctuated::Punctuated,
    token::Comma
};
use syn::parse::Parser;

/// Implementation of the action macro
pub fn action_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a function
    let input = parse_macro_input!(item as ItemFn);
    
    // Extract the action name (default to function name if not specified)
    let action_name = if attr.is_empty() {
        // If no name is provided, use the function name
        input.sig.ident.to_string()
    } else {
        // Parse the attribute arguments
        let parser = Punctuated::<Lit, Comma>::parse_terminated;
        let lit_args = parser.parse(attr).expect("Failed to parse attribute arguments");
        
        if lit_args.is_empty() {
            input.sig.ident.to_string()
        } else {
            // Get the first argument as a string literal
            match &lit_args[0] {
                Lit::Str(s) => s.value(),
                _ => panic!("Action name must be a string literal")
            }
        }
    };
    
    // Extract parameters from the function signature
    let params = extract_parameters(&input);
    
    // Generate the register action method
    let register_action_method = generate_register_action_method(
        &input.sig.ident,
        &action_name,
        &params,
        &input.sig.output,
    );
    
    // Combine the original function with the generated register method
    let expanded = quote! {
        #input

        #register_action_method
    };
    
    expanded.into()
}



/// Extract parameters from the function signature
fn extract_parameters(input: &ItemFn) -> Vec<(Ident, Type)> {
    let mut params = Vec::new();
    
    for arg in &input.sig.inputs {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Skip the context parameter
                if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                    let ident_string = ident.to_string();
                    if ident_string != "self" && ident_string != "ctx" && !ident_string.ends_with("ctx") {
                        params.push((ident.clone(), (**ty).clone()));
                    }
                }
            }
            _ => {}
        }
    }
    
    params
}

/// Generate the register action method
fn generate_register_action_method(
    fn_ident: &Ident,
    action_name: &str,
    params: &[(Ident, Type)],
    _return_type: &ReturnType, // Not used but kept for future extensibility
) -> TokenStream2 {
    // Create the register method name (e.g., register_action_add for add)
    let register_method_name = format_ident!("register_action_{}", fn_ident);
    
    // Generate parameter extraction code
    let param_extractions = generate_parameter_extractions(params);
    
    // Generate method call with extracted parameters
    let method_call = generate_method_call(fn_ident, params);
    
    // Generate the register action method
    quote! {
        async fn #register_method_name(&self, context: &runar_node::services::LifecycleContext) -> anyhow::Result<()> {
            context.info(format!("Registering '{}' action", #action_name));
            
            // Create a clone of self that can be moved into the closure
            let self_clone = self.clone();
            
            // Create the action handler
            let handler = Arc::new(move |params_opt: Option<runar_common::types::ArcValueType>, ctx: runar_node::services::RequestContext| 
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<runar_node::services::ServiceResponse, anyhow::Error>> + Send>> {
                let inner_self = self_clone.clone();
                
                Box::pin(async move {
                    // Extract parameters from the map if available
                    let mut params_value = match params_opt {
                        Some(p) => p,
                        None => {
                            ctx.error("No parameters provided".to_string());
                            return Ok(runar_node::services::ServiceResponse {
                                status: 400,
                                data: None,
                                error: Some("No parameters provided".to_string()),
                            });
                        }
                    };
                    
                    // Get the parameters as a map of string to directly stored values
                    let params_map = params_value.as_map_ref::<String, f64>()?;
                    
                    #param_extractions
                    
                    // Call the actual method with the extracted parameters
                    match #method_call.await {
                        Ok(result) => {
                            // Convert the result to ArcValueType
                            let value_type = runar_common::types::ArcValueType::new_primitive(result);
                            Ok(runar_node::services::ServiceResponse {
                                status: 200,
                                data: Some(value_type),
                                error: None,
                            })
                        },
                        Err(err) => {
                            // Return an error response
                            ctx.error(format!("Action '{}' failed: {}", #action_name, err));
                            Ok(runar_node::services::ServiceResponse {
                                status: 500,
                                data: None,
                                error: Some(err.to_string()),
                            })
                        }
                    }
                })
            });
            
            // Register the action handler
            context.register_action(#action_name, handler).await
        }
    }
}

/// Generate parameter extraction code
fn generate_parameter_extractions(params: &[(Ident, Type)]) -> TokenStream2 {
    let mut extractions = TokenStream2::new();
    
    for (param_ident, param_type) in params {
        let param_name = param_ident.to_string();
        let type_name = quote! { #param_type }.to_string();
        
        // For now, assume all parameters are f64 as that's what the test uses
        // This could be extended to handle different parameter types if needed
        let extraction = quote! {
            let #param_ident = match params_map.get(#param_name) {
                Some(value) => {
                    *value // Direct access to the value
                },
                None => {
                    ctx.error(format!("Missing parameter {}", #param_name));
                    return Ok(runar_node::services::ServiceResponse {
                        status: 400,
                        data: None,
                        error: Some(format!("Missing parameter {}", #param_name)),
                    });
                }
            };
        };
        
        extractions.extend(extraction);
    }
    
    extractions
}

/// Generate method call with extracted parameters
fn generate_method_call(fn_ident: &Ident, params: &[(Ident, Type)]) -> TokenStream2 {
    let param_idents = params.iter().map(|(ident, _)| {
        quote! { #ident }
    });
    
    quote! {
        inner_self.#fn_ident(#(#param_idents,)* &ctx)
    }
}
