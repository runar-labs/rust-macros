use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Meta, parse::Parser};
use crate::utils::extract_name_value_pairs;

/// Subscribe macro for defining event handlers in Runar services
///
/// This macro marks methods as event subscription handlers and registers them with the Node's
/// event system during service initialization.
///
/// # Parameters
/// - `topic`: The event topic to subscribe to (default: method name)
///   - Can be a relative path (e.g., "user_created") which will be prefixed with service path
///   - Can be a full path (e.g., "users/user_created") which will be used as-is
///
/// # Examples
/// ```rust
/// // Subscribe to a specific topic (service path will be prefixed)
/// #[subscribe(topic = "user_created")]
/// async fn handle_user_created(&mut self, payload: ValueType) -> Result<()> {
///     // Extract data using vmap! macro
///     let id = vmap!(payload, "id" => String::new());
///     let name = vmap!(payload, "name" => String::new());
///     
///     // Handler implementation
///     Ok(())
/// }
///
/// // Subscribe using a full path
/// #[subscribe(topic = "users/user_created")]
/// async fn handle_user_event(&mut self, payload: ValueType) -> Result<()> {
///     // Handler implementation
///     Ok(())
/// }
///
/// // Default subscription using method name as topic
/// #[subscribe]
/// async fn custom_event(&mut self, payload: ValueType) -> Result<()> {
///     // Handler implementation
///     Ok(())
/// }
/// ```
///
/// # Requirements
/// - The service must implement `Clone` to support multiple subscription handlers
/// - Handler methods must be `async` and return `Result<()>`
/// - Handler methods should take `&mut self` and a `ValueType` parameter
///
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Verify method is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "subscribe handlers must be async"
        ).to_compile_error().into();
    }
    
    // Extract the method name which will be used as default topic if none provided
    let method_name = &input_fn.sig.ident;
    let method_name_str = method_name.to_string();
    
    // Get the topic from attributes
    let topic = if attr.is_empty() {
        // If no attributes, use the method name as the topic
        method_name_str.clone()
    } else {
        // Parse the attribute tokens into a list of Meta items
        let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
        let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
        
        // Convert meta_list into a Vec<Meta>
        let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
        
        // Extract name-value pairs
        let name_value_pairs = extract_name_value_pairs(&meta_vec);
        
        // Find the topic attribute
        name_value_pairs.get("topic")
            .cloned()
            .unwrap_or_else(|| method_name_str.clone())
    };
    
    // Determine if the topic is a full path (contains a slash)
    let is_full_path = topic.contains('/');
    
    // Generate code to register subscription during service initialization
    let output = quote! {
        // Keep the original handler method unchanged
        #input_fn
        
        // Add code to register this subscription during service initialization
        #[doc(hidden)]
        #[allow(non_snake_case)]
        const _: () = {
            // Add this subscription to the service's subscription registry
            #[allow(non_upper_case_globals)]
            static REGISTER_SUBSCRIPTION: () = {
                extern crate std;
                
                // Add the subscription setup code for this handler
                ::inventory::submit! {
                    crate::registry::SubscriptionHandler {
                        method_name: #method_name_str,
                        topic: #topic,
                        is_full_path: #is_full_path,
                        register_fn: |svc, ctx| {
                            Box::pin(async move {
                                // Downcast the service from Any to our concrete type
                                let svc_clone = if let Some(typed_service) = (svc as &dyn std::any::Any).downcast_ref::<Self>() {
                                    typed_service.clone()
                                } else {
                                    // If downcast fails, this is a serious error
                                    return Err(anyhow::anyhow!("Failed to downcast service to concrete type for subscription handler"));
                                };
                                
                                // Construct the full topic path 
                                let topic_path = if #is_full_path {
                                    #topic.to_string()
                                } else {
                                    // For relative paths, prefix with service path
                                    format!("{}/{}", svc_clone.path(), #topic)
                                };
                                
                                // Register directly with the context
                                ctx.subscribe(&topic_path, move |payload| {
                                    // Get another clone for the actual handler execution
                                    let mut service = svc_clone.clone();
                                    
                                    // Return a future that calls our handler method
                                    Box::pin(async move {
                                        // Call the actual method with proper error handling
                                        match service.#method_name(payload).await {
                                            Ok(()) => Ok(()),
                                            Err(e) => {
                                                // Log the error but don't propagate it to prevent subscription cancellation
                                                eprintln!("Error in subscription handler for topic {}: {:?}", #topic, e);
                                                Ok(())
                                            }
                                        }
                                    })
                                }).await.map_err(|e| {
                                    anyhow::anyhow!("Failed to subscribe to topic {}: {}", topic_path, e)
                                })
                            })
                        }
                    }
                };
            };
        };
    };
    
    TokenStream::from(output)
} 