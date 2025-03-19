use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident, parse::Parser, Meta};
use proc_macro2::Span;

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
/// // Named action
/// #[action(name = "get_user")]
/// async fn get_user_by_id(&self, ctx: &RequestContext, id: &str) -> Result<User> {
///     // Implementation
///     Ok(user)
/// }
///
/// // Default name from method
/// #[action]
/// async fn get_posts(&self, ctx: &RequestContext, user_id: &str) -> Result<Vec<Post>> {
///     // Implementation
///     Ok(posts)
/// }
/// ```
///
/// # Parameter Handling
/// The macro supports extracting parameters from both direct values and mapped parameters:
/// - Single parameter actions can be called with direct values: `node.request("service/action", "value")`
/// - All actions can be called with mapped parameters: `node.request("service/action", vmap!{"param" => value})`
///
/// # Return Values
/// - Action methods should return `Result<T>`, not `ServiceResponse`
/// - The macro handles converting the result to a `ServiceResponse` automatically
/// - Error handling is done via the `?` operator in the generated code
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Extract the method name which will be used as default operation name if none provided
    let method_name = &input_fn.sig.ident;
    let method_name_str = method_name.to_string();
    
    // Get the operation name from attributes or use method name
    let operation_name = if attr.is_empty() {
        method_name_str.clone()
    } else {
        // Parse the attribute tokens into a list of Meta items
        let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
        let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
        
        // Convert meta_list into a Vec<Meta>
        let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
        
        // Extract name-value pairs
        let name_value_pairs = crate::utils::extract_name_value_pairs(&meta_vec);
        
        // Find the name attribute
        name_value_pairs.get("name")
            .cloned()
            .unwrap_or_else(|| method_name_str.clone())
    };
    
    // Verify method is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "action methods must be async"
        ).to_compile_error().into();
    }
    
    // Extract parameters from the function signature
    let parameters = extract_parameters(&input_fn.sig.inputs);
    
    // Generate parameter extraction code
    let param_extraction = generate_parameter_extraction(&parameters);
    
    // Generate parameter names for the method call
    let param_names = parameters.iter()
        .filter(|p| p.name != "context" && p.name != "ctx" && p.name != "_context" && p.name != "_ctx")
        .map(|p| Ident::new(&p.name, Span::call_site()));
    
    // Full implementation with node_implementation feature
    let output = quote! {
        // Keep the original function implementation
        #input_fn
        
        // Add code to register this action in the service's operation dispatch table
        #[doc(hidden)]
        #[allow(non_snake_case)]
        const _: () = {
            // Register this action in the action registry
            #[allow(non_upper_case_globals)]
            static REGISTER_ACTION: () = {
                extern crate std;
                
                // Register this action with the service's action registry
                ::inventory::submit! {
                    crate::action_registry::ActionItem {
                        name: #operation_name.to_string(),
                        service_type_id: std::any::TypeId::of::<Self>(),
                        handler_fn: |svc, context, _operation, params| {
                            Box::pin(async move {
                                // Extract parameters from the request
                                #param_extraction
                                
                                // Downcast the service to our concrete type
                                let concrete_svc = match (svc as &dyn std::any::Any).downcast_ref::<Self>() {
                                    Some(s) => s,
                                    None => return Err(anyhow::anyhow!("Failed to downcast service to the required type"))
                                };
                                
                                // Call the actual method with extracted parameters and get the result
                                let result = match concrete_svc.#method_name(context, #(#param_names),*).await {
                                    // Success case - wrap the result in a ServiceResponse
                                    Ok(value) => runar_node::services::ServiceResponse::success(
                                        value,
                                        None
                                    ),
                                    // Error case - create an error response
                                    Err(e) => runar_node::services::ServiceResponse::error(
                                        e.to_string(),
                                        None
                                    ),
                                };
                                
                                // Return the response
                                Ok(result)
                            })
                        }
                    }
                };
            };
        };
    };
    
    TokenStream::from(output)
}

/// Represents a parameter in a function signature
struct Parameter {
    name: String,
    ty: String,
    is_reference: bool,
}

/// Extracts parameters from a function signature
fn extract_parameters(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    
    for input in inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            // Extract parameter name
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = ident.ident.to_string();
                
                // Skip self parameter
                if param_name == "self" {
                    continue;
                }
                
                // Extract parameter type
                let type_str = quote! { #pat_type.ty }.to_string();
                
                // Check if it's a reference
                let is_reference = if let syn::Type::Reference(_) = &*pat_type.ty {
                    true
                } else {
                    false
                };
                
                parameters.push(Parameter {
                    name: param_name,
                    ty: type_str,
                    is_reference,
                });
            }
        }
    }
    
    parameters
}

/// Generates code to extract parameters from a request
fn generate_parameter_extraction(parameters: &[Parameter]) -> proc_macro2::TokenStream {
    let mut extraction_code = proc_macro2::TokenStream::new();
    
    for param in parameters {
        // Skip context parameter
        if param.name == "context" || param.name == "ctx" || param.name == "_context" || param.name == "_ctx" {
            continue;
        }
        
        // Generate parameter extraction based on type
        let param_name = Ident::new(&param.name, Span::call_site());
        let param_extraction = quote! {
            let #param_name = crate::utils::extract_parameter::<_, _>(
                &params, 
                #param_name,
                concat!("Missing required parameter: ", #param_name)
            )?;
        };
        
        extraction_code.extend(param_extraction);
    }
    
    extraction_code
} 