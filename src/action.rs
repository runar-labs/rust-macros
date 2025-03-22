use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident, parse::Parser, Meta, Type, TypePath, Path, PathSegment};
use proc_macro2::Span;
use rand;

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
/// async fn get_user_by_id(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
///     // Implementation
///     Ok(ServiceResponse::success("User found", Some(user_data)))
/// }
///
/// // Default name from method
/// #[action]
/// async fn get_posts(&self, request: ServiceRequest) -> Result<ServiceResponse, anyhow::Error> {
///     // Implementation
///     Ok(ServiceResponse::success("Posts retrieved", Some(posts_data)))
/// }
/// ```
///
/// # Parameter Handling
/// The macro supports extracting parameters from the ServiceRequest:
/// - All actions can be called with: `node.request("service/action", params)`
/// - Parameters are accessed via `request.params`
///
/// # Return Values
/// - Action methods should return `Result<ServiceResponse, anyhow::Error>`
/// - Error handling is done via the `?` operator in the generated code
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
    
    // Generate a unique ID for this action
    let unique_id = rand::random::<u32>();
    let action_id = format!("ACTION_{}", unique_id);
    let action_ident = Ident::new(&action_id, Span::call_site());
    
    // Generate the struct name for this action's registry
    let register_struct_name = format!("RegisterAction{}", unique_id);
    let register_struct_ident = Ident::new(&register_struct_name, Span::call_site());
    
    // Generate the implementation
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;
    
    // The new approach uses a separate struct to register the action at runtime
    // This avoids the Self issues in static contexts
    let output = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        struct #register_struct_ident;
        
        impl #register_struct_ident {
            const NAME: &'static str = #operation_name;
            
            fn register<T: 'static>() {
                inventory::submit!(crate::registry::ActionItem {
                    name: Self::NAME.to_string(),
                    service_type_id: std::any::TypeId::of::<T>(),
                });
            }
        }
        
        // Create an inventory item that will register on service instantiation
        inventory::submit! {
            // This will be registered when the inventory is first accessed
            crate::registry::ActionRegistrar {
                register_fn: |type_id| {
                    // Register with the specified type ID
                    inventory::submit!(crate::registry::ActionItem {
                        name: #operation_name.to_string(),
                        service_type_id: type_id,
                    });
                },
            }
        }
        
        // Keep the original attributes and function
        #(#attrs)*
        #vis #sig {
            #block
        }
    };
    
    TokenStream::from(output)
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