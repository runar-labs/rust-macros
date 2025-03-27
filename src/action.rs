use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Ident, parse::Parser, Meta, Type, TypePath, Path, PathSegment, FnArg, PatType};
use proc_macro2::Span;
use rand;
use std::collections::HashSet;

/// Action macro for defining service operations in Runar
///
/// This macro marks methods as service operations that can be invoked through the Node API.
/// It handles parameter extraction and result conversion.
///
/// # Parameters
/// - `name`: The operation name that will be used in node.request() calls (default: method name)
///
/// # Examples
/// ```rust
/// // Named action with direct parameters
/// #[action(name = "add")]
/// async fn add(&self, a: i32, b: i32) -> Result<i32, anyhow::Error> {
///     // Implementation directly uses a and b
///     Ok(a + b)
/// }
///
/// // Action using ServiceRequest - supported for backward compatibility
/// #[action]
/// async fn get_posts(&self, request: ServiceRequest) -> Result<Vec<Post>, anyhow::Error> {
///     // Implementation extracts parameters from request
///     Ok(posts)
/// }
/// ```
///
/// # Parameter Handling
/// - For direct parameters: The macro extracts them from the request data automatically
/// - For ServiceRequest parameter: Passed through as-is for backward compatibility
///
/// # Return Values
/// - Action methods can return their actual data types wrapped in Result<T, anyhow::Error>
/// - The macro automatically wraps the return value in ServiceResponse
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Extract the method name which will be used as default operation name if none provided
    let method_name = &input_fn.sig.ident;
    let method_name_str = method_name.to_string();
    
    // Parse the attribute tokens into a list of Meta items
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
    
    // Convert meta_list into a Vec<Meta>
    let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
    
    // Extract name-value pairs
    let name_value_pairs = crate::utils::extract_name_value_pairs(&meta_vec);
    
    // Find the name attribute or default to method name
    let operation_name = name_value_pairs
        .get("name")
        .cloned()
        .unwrap_or_else(|| method_name_str.clone());
    
    // Verify method is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "action methods must be async"
        ).to_compile_error().into();
    }
    
    // Extract parameter information
    let params = extract_params(&input_fn.sig);
    
    // Check if the method takes a ServiceRequest (legacy support)
    let takes_service_request = params.len() == 1 && 
        is_type_named_with_path(&params[0].ty, &["runar_node", "services", "ServiceRequest"]) ||
        is_type_named_with_path(&params[0].ty, &["ServiceRequest"]);
    
    // Generate the appropriate handler based on whether it uses direct parameters or ServiceRequest
    let handler_impl = if takes_service_request {
        // For legacy methods that directly take ServiceRequest, just pass it through
        quote! {
            // Create an operation handler that will be matched in handle_request
            async fn #method_name(&self, request: runar_node::services::ServiceRequest) -> anyhow::Result<runar_node::services::ServiceResponse> {
                // Call the original method with the request
                match self.#method_name(request).await {
                    Ok(result) => {
                        // Return a success response
                        Ok(runar_node::services::ServiceResponse::success(
                            format!("Operation '{}' completed successfully", #operation_name),
                            Some(runar_node::types::ValueType::Number(result as f64))
                        ))
                    },
                    Err(e) => {
                        // Return error response
                        Ok(runar_node::services::ServiceResponse::error(
                            format!("Operation '{}' failed: {}", #operation_name, e)
                        ))
                    }
                }
            }
        }
    } else {
        // For direct parameter methods, extract parameters from request.data
        
        // Generate parameter extraction code
        let param_extractions = params.iter().filter(|p| p.name != "self").map(|param| {
            let param_name = &param.name;
            let param_name_str = param_name.to_string();
            
            quote! {
                // Extract parameter from request data using vmap_i32! macro
                let #param_name = runar_common::vmap_i32!(data, #param_name_str, 0);
            }
        });
        
        // Generate parameter list for passing to the original method
        let param_names = params.iter()
            .filter(|p| p.name != "self")
            .map(|p| &p.name);
        
        // Generate a unique handler name to avoid conflicts
        let handler_name = format_ident!("handle_{}", operation_name);
        
        quote! {
            // Create an operation handler that will be matched in handle_request
            async fn #handler_name(&self, request: runar_node::services::ServiceRequest) -> anyhow::Result<runar_node::services::ServiceResponse> {
                // Extract data from request
                let data = match &request.data {
                    Some(data) => data,
                    None => {
                        return Ok(runar_node::services::ServiceResponse::error(
                            format!("Missing request data for operation '{}'", #operation_name)
                        ));
                    }
                };
                
                // Extract parameters from data
                #(#param_extractions)*
                
                // Call the original method with extracted parameters
                match self.#method_name(#(#param_names),*).await {
                    Ok(result) => {
                        // Return a success response
                        Ok(runar_node::services::ServiceResponse::success(
                            format!("Operation '{}' completed successfully", #operation_name),
                            Some(runar_node::types::ValueType::Number(result as f64))
                        ))
                    },
                    Err(e) => {
                        // Return error response
                        Ok(runar_node::services::ServiceResponse::error(
                            format!("Operation '{}' failed: {}", #operation_name, e)
                        ))
                    }
                }
            }
        }
    };
    
    // Generate the final implementation
    let output = quote! {
        // Keep the original method
        #input_fn
        
        // Generate the operation handler
        #handler_impl
    };
    
    TokenStream::from(output)
}

/// Struct to collect parameter information
struct ParamInfo {
    name: Ident,
    ty: Type,
}

/// Extract parameter information from a function signature
fn extract_params(sig: &syn::Signature) -> Vec<ParamInfo> {
    sig.inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let syn::Pat::Ident(pat_ident) = &**pat {
                    Some(ParamInfo {
                        name: pat_ident.ident.clone(),
                        ty: (*ty.clone()),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Check if a type is named a certain way with a specific path
fn is_type_named_with_path(ty: &Type, path_segments: &[&str]) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == path_segments.len() {
            return path.segments.iter().zip(path_segments.iter())
                .all(|(seg, &name)| seg.ident == name);
        } else if let Some(last_segment) = path_segments.last() {
            // If the path isn't fully qualified, check just the last segment
            if let Some(last_path_segment) = path.segments.last() {
                return &last_path_segment.ident.to_string() == *last_segment;
            }
        }
    }
    false
}

/// Check if the return type is Result<ServiceResponse>
fn is_service_response_return(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            // Check if it's a Result<ServiceResponse>
            match &**ty {
                Type::Path(TypePath { path, .. }) => {
                    // Check if the outer type is Result
                    if is_type_named(path, "Result") {
                        // Check if there are generic arguments
                        if let Some(PathSegment { arguments, .. }) = path.segments.last() {
                            // Check if the first type argument is ServiceResponse
                            if let syn::PathArguments::AngleBracketed(args) = arguments {
                                if let Some(syn::GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.args.first() {
                                    return is_type_named(path, "ServiceResponse");
                                }
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        }
    }
}

/// Check if a type is named a certain way
fn is_type_named(path: &Path, name: &str) -> bool {
    path.segments.last()
        .map(|seg| seg.ident == name)
        .unwrap_or(false)
}