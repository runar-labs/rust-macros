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
    
    // Default to function name
    let fn_name = input.sig.ident.to_string();
    
    // Parse the attributes
    let mut action_name = fn_name.clone();
    let mut action_path = fn_name.clone();
    
    if !attr.is_empty() {
        // Convert attribute tokens to a string for simple parsing
        let attr_str = attr.to_string();
        
        // Extract attributes from the TokenStream
        if attr_str.contains("path") {
            // Try to parse as a name-value attribute
            // For safety, we're using a simple string parsing approach
            let attr_str = attr.to_string();
            
            if attr_str.contains("path") && attr_str.contains('=') && attr_str.contains('"') {
                // Find the path value
                let start_idx = attr_str.find("path").unwrap() + 4; // Skip 'path'
                let equals_idx = attr_str[start_idx..].find('=').unwrap() + start_idx + 1; // Skip '='
                let quote_start_idx = attr_str[equals_idx..].find('"').unwrap() + equals_idx + 1; // Skip opening quote
                let quote_end_idx = attr_str[quote_start_idx..].find('"').unwrap() + quote_start_idx;
                
                // Extract the path value
                action_path = attr_str[quote_start_idx..quote_end_idx].to_string();
            }
        } else {
            // Try to parse as a simple string literal for backward compatibility
            let parser = Punctuated::<Lit, Comma>::parse_terminated;
            if let Ok(lit_args) = parser.parse(attr.clone()) {
                if !lit_args.is_empty() {
                    // Get the first argument as a string literal for the name
                    if let Lit::Str(s) = &lit_args[0] {
                        action_name = s.value();
                        action_path = action_name.clone(); // Use the same value for path if not specified separately
                    }
                }
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
        &action_path,
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
    action_path: &str,
    params: &[(Ident, Type)],
    return_type: &ReturnType,
    is_primitive: &bool,
    type_name: &String,
    needs_registration: &bool,
) -> TokenStream2 {
    // Create a boolean expression for checking if there are parameters
    let has_params = if params.is_empty() {
        quote! { false }
    } else {
        quote! { true }
    };
    
    // Generate parameter extraction code
    let param_extractions = generate_parameter_extractions(params);
    
    // Generate method call with extracted parameters
    let method_call = generate_method_call(fn_ident, params);
    
    // Generate the appropriate result handling based on the return type
    let result_handling = if *is_primitive {
        quote! {
            // Convert the result to ArcValueType
            let value_type = runar_common::types::ArcValueType::new_primitive(result);
            
            Ok(runar_node::services::ServiceResponse {
                status: 200,
                data: Some(value_type),
                error: None,
            })
        }
    } else {
        quote! {
            // Convert the complex result to ArcValueType using appropriate value category
            let value_type = runar_common::types::ArcValueType::from_struct(result);
            
            Ok(runar_node::services::ServiceResponse {
                status: 200,
                data: Some(value_type),
                error: None,
            })
        }
    };
    
    // Generate a unique method name for the action registration
    let register_method_name = format_ident!("register_action_{}", fn_ident);
    
    quote! {
        async fn #register_method_name(&self, context: &runar_node::services::LifecycleContext) -> anyhow::Result<()> {
            context.logger.info(format!("Registering '{}' action", #action_name));
            
            // Create a clone of self that can be moved into the closure
            let self_clone = self.clone();
            
            // Create the action handler as an Arc to match what the register_action expects
            let handler = std::sync::Arc::new(move |params_opt: Option<runar_common::types::ArcValueType>, ctx: runar_node::services::RequestContext| 
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<runar_node::services::ServiceResponse, anyhow::Error>> + Send>> {
                let inner_self = self_clone.clone();
                
                Box::pin(async move {
                    // Extract parameters from the map if available
                    let mut params_value = match params_opt {
                        Some(p) => p,
                        None => {
                            // Check if method expects parameters
                            if #has_params {
                                ctx.error("No parameters provided".to_string());
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some("No parameters provided".to_string()),
                                });
                            } else {
                                // No parameters expected, so create an empty map
                                runar_common::types::ArcValueType::new_map(
                                    std::collections::HashMap::<String, runar_common::types::ArcValueType>::new()
                                )
                            }
                        }
                    };
                    
                    #param_extractions
                    
                    // Call the actual method with the extracted parameters
                    match #method_call.await {
                        Ok(result) => {
                            #result_handling
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
            if #needs_registration {
                context.logger.debug(format!("Type registration needed for action '{}' with type: {}", #action_name, #type_name));
                // The actual registration logic would depend on the service's serializer API
            }
            
            // Register the action handler with the configured path
            context.register_action(
                #action_path.to_string(),
                handler
            ).await
        }
    }
}

/// Generate parameter extraction code to exactly match the reference implementation
fn generate_parameter_extractions(params: &[(Ident, Type)]) -> TokenStream2 {
    let mut extractions = TokenStream2::new();
    
    for (param_ident, param_type) in params {
        let param_name = param_ident.to_string();
        let type_str = quote! { #param_type }.to_string();
        
        // Extract parameters based on their type
        let extraction = if type_str.contains("f64") || type_str.contains("f32") {
            // Floating point extraction
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, f64>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => *value,
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map with f64 values: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map with f64 values: {}", err)),
                        });
                    }
                };
            }
        } else if type_str.contains("i32") {
            // Integer extraction (i32)
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, i32>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => *value,
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map with i32 values: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map with i32 values: {}", err)),
                        });
                    }
                };
            }
        } else if type_str.contains("i64") {
            // Integer extraction (i64)
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, i64>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => *value,
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map with i64 values: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map with i64 values: {}", err)),
                        });
                    }
                };
            }
        } else if type_str.contains("String") || type_str.contains("&str") {
            // String extraction
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, String>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => value.clone(),
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map with String values: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map with String values: {}", err)),
                        });
                    }
                };
            }
        } else if type_str.contains("bool") {
            // Boolean extraction
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, bool>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => *value,
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map with bool values: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map with bool values: {}", err)),
                        });
                    }
                };
            }
        } else {
            // Complex type (struct) extraction - attempt to deserialize
            quote! {
                let #param_ident = match params_value.as_map_ref::<String, runar_common::types::ArcValueType>() {
                    Ok(map) => {
                        match map.get(#param_name) {
                            Some(value) => {
                                match value.as_type::<#param_type>() {
                                    Ok(val) => val,
                                    Err(err) => {
                                        ctx.error(format!("Failed to parse parameter {}: {}", #param_name, err));
                                        return Ok(runar_node::services::ServiceResponse {
                                            status: 400,
                                            data: None,
                                            error: Some(format!("Failed to parse parameter {}: {}", #param_name, err)),
                                        });
                                    }
                                }
                            },
                            None => {
                                ctx.error(format!("Missing parameter {}", #param_name));
                                return Ok(runar_node::services::ServiceResponse {
                                    status: 400,
                                    data: None,
                                    error: Some(format!("Missing parameter {}", #param_name)),
                                });
                            }
                        }
                    },
                    Err(err) => {
                        ctx.error(format!("Failed to parse parameters as map: {}", err));
                        return Ok(runar_node::services::ServiceResponse {
                            status: 400,
                            data: None,
                            error: Some(format!("Failed to parse parameters as map: {}", err)),
                        });
                    }
                };
            }
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
