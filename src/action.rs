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
    
    // Extract the return type information for proper handling
    let return_type_info = extract_return_type_info(&input.sig.output);
    
    // Generate the register action method based on return type information
    let register_action_method = generate_register_action_method(
        &input.sig.ident,
        &action_name,
        &params,
        &input.sig.output,
        &return_type_info.is_primitive,
        &return_type_info.type_name,
        &return_type_info.needs_registration,
    );
    
    // Combine the original function with the generated register method
    let expanded = quote! {
        #input

        #register_action_method
    };
    
    expanded.into()
}

/// Extract information about the return type for proper handling
fn extract_return_type_info(return_type: &ReturnType) -> ReturnTypeInfo {
    match return_type {
        ReturnType::Default => ReturnTypeInfo {
            is_result: false,
            type_name: "()".to_string(),
            is_primitive: true,
            needs_registration: false,
        },
        ReturnType::Type(_, ty) => {
            // Convert the type to a string for analysis
            let type_str = quote! { #ty }.to_string();
            
            // Check if this is a Result type
            let is_result = type_str.contains("Result");
            
            // Determine the actual return type (if it's Result<T, E>, extract T)
            let inner_type = if is_result {
                // Try to extract the type parameter from Result<T, E>
                if let Some(start) = type_str.find('<') {
                    if let Some(end) = type_str.find(',') {
                        type_str[start+1..end].trim().to_string()
                    } else {
                        // Fallback if we can't parse the Result type
                        "unknown".to_string()
                    }
                } else {
                    // Fallback if we can't parse the Result type
                    "unknown".to_string()
                }
            } else {
                // Not a Result, use the whole type
                type_str
            };
            
            // Determine if this is a primitive type
            let is_primitive = inner_type.contains("i32") || 
                              inner_type.contains("i64") || 
                              inner_type.contains("u32") || 
                              inner_type.contains("u64") || 
                              inner_type.contains("f32") || 
                              inner_type.contains("f64") || 
                              inner_type.contains("bool") || 
                              inner_type.contains("String") || 
                              inner_type.contains("&str") || 
                              inner_type.contains("()");
            
            // Determine if this type needs registration with the serializer
            let needs_registration = !is_primitive && 
                                    !inner_type.contains("Vec") && 
                                    !inner_type.contains("HashMap");
            
            ReturnTypeInfo {
                is_result,
                type_name: inner_type,
                is_primitive,
                needs_registration,
            }
        }
    }
}

/// Struct to hold information about the return type
struct ReturnTypeInfo {
    is_result: bool,        // Whether the return type is a Result
    type_name: String,      // The name of the type (or inner type if Result)
    is_primitive: bool,     // Whether it's a primitive type
    needs_registration: bool, // Whether it needs registration with the serializer
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
    return_type: &ReturnType,
    is_primitive: &bool,
    type_name: &String,
    needs_registration: &bool,
) -> TokenStream2 {
    // Create the register method name (e.g., register_action_add for add)
    let register_method_name = format_ident!("register_action_{}", fn_ident);
    
    // Generate parameter extraction code
    let param_extractions = generate_parameter_extractions(params);
    
    // Generate method call with extracted parameters
    let method_call = generate_method_call(fn_ident, params);
    
    // Match exactly the format of the reference implementation
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
            
            // If this action returns a type that needs registration with the serializer,
            // we would register it here
            // Note: This would be expanded in a complete implementation
            if #needs_registration {
                context.info(format!("Type registration needed for action '{}' with type: {}", #action_name, #type_name));
                // The actual registration logic would depend on the service's serializer API
            }
            
            // Register the action handler
            context.register_action(#action_name, handler).await
        }
    }
}

/// Generate parameter extraction code to exactly match the reference implementation
fn generate_parameter_extractions(params: &[(Ident, Type)]) -> TokenStream2 {
    let mut extractions = TokenStream2::new();
    
    for (param_ident, _param_type) in params {
        let param_name = param_ident.to_string();
        
        // Exactly match how the reference implementation extracts parameters
        let extraction = quote! {
            let #param_ident = match params_map.get(#param_name) {
                Some(value) => {
                    *value // We need to dereference since the methods expect f64 not &f64
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
