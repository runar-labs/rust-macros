use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Meta, parse::Parser};
use crate::utils::extract_name_value_pairs;

/// Publish macro for defining event publishing methods in Runar services
///
/// This macro marks methods that publish events to specified topics. It can be used
/// to automatically publish events when a method is called.
///
/// # Parameters
/// - `topic`: The event topic to publish to (required)
///   - Can be a relative path (e.g., "user_created") which will be prefixed with service path
///   - Can be a full path (e.g., "users/user_created") which will be used as-is
///
/// # Examples
/// ```rust
/// // Publish to a service-specific topic
/// #[publish(topic = "item_updated")]
/// async fn update_item(&self, ctx: &RequestContext, id: &str, data: ValueType) -> Result<Item> {
///     // Update the item
///     let updated_item = self.db.update_item(id, data).await?;
///     
///     // The event will be published automatically with the return value
///     Ok(updated_item)
/// }
///
/// // Publish to a full path topic
/// #[publish(topic = "inventory/stock_changed")]
/// async fn adjust_stock(&self, ctx: &RequestContext, product_id: &str, quantity: i32) -> Result<i32> {
///     // Adjust stock level
///     let new_stock = self.db.adjust_stock(product_id, quantity).await?;
///     
///     // The new stock level will be published with the return value
///     Ok(new_stock)
/// }
/// ```
///
/// # Implementation Details
/// - The macro wraps the method to automatically publish an event after successful execution
/// - The published event payload includes the method's return value
/// - The context parameter is used for publishing the event
/// - For error handling, the event is only published if the method returns Ok
pub fn publish(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Extract the method name for error messaging
    let method_name = &input_fn.sig.ident;
    let method_name_str = method_name.to_string();
    
    // Verify method is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "publish methods must be async"
        ).to_compile_error().into();
    }
    
    // Parse attributes to get the topic
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
    
    // Convert meta_list into a Vec<Meta>
    let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
    
    // Extract name-value pairs
    let name_value_pairs = extract_name_value_pairs(&meta_vec);
    
    // Topic is required for publish macro
    let topic = match name_value_pairs.get("topic") {
        Some(topic_value) => topic_value.clone(),
        None => {
            return syn::Error::new_spanned(
                input_fn.sig.fn_token,
                "publish macro requires a 'topic' parameter, e.g., #[publish(topic = \"my_event\")]"
            ).to_compile_error().into();
        }
    };
    
    // Determine if the topic is a full path (contains a slash)
    let is_full_path = topic.contains('/');
    
    // The original function signature and body
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    
    // Check for context parameter
    let mut has_context = false;
    for input in &fn_sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = ident.ident.to_string();
                if param_name == "context" || param_name == "ctx" {
                    has_context = true;
                    break;
                }
            }
        }
    }
    
    if !has_context {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "publish methods must have a 'context' or 'ctx' parameter"
        ).to_compile_error().into();
    }
    
    // Generate the wrapped function with publishing logic
    let output = quote! {
        #[allow(non_snake_case)]
        #fn_vis #fn_sig {
            // Get the original function result
            let result = async move { #fn_block }.await;
            
            // Only publish if successful
            if let Ok(ref value) = result {
                // Construct the full topic path
                let topic_path = if #is_full_path {
                    #topic.to_string()
                } else {
                    // For relative paths, prefix with service path
                    format!("{}/{}", self.path(), #topic)
                };
                
                // Publish the event with the result value
                if let Err(publish_err) = context.publish(&topic_path, value).await {
                    // Log the error but don't fail the original method
                    eprintln!("Error publishing event to topic {}: {:?}", topic_path, publish_err);
                }
            }
            
            // Return the original result
            result
        }
    };
    
    TokenStream::from(output)
} 