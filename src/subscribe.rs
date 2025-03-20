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
    
    // Extract the method name
    let method_name = &input_fn.sig.ident;
    let method_name_str = method_name.to_string();
    
    // Parse the attribute tokens into a list of Meta items
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse(attr.clone().into()).unwrap_or_default();
    
    // Convert meta_list into a Vec<Meta>
    let meta_vec: Vec<Meta> = meta_list.into_iter().collect();
    
    // Extract name-value pairs
    let attrs = extract_name_value_pairs(&meta_vec);
    
    // Extract the topic from attributes or use method name
    let topic = attrs.get("topic").cloned().unwrap_or_else(|| method_name_str.clone());
    
    // Check if this is a full path
    let is_full_path = topic.contains('/');
    
    // Extract the receiver type (service type) from the first parameter
    let self_ty = match &input_fn.sig.inputs.first() {
        Some(syn::FnArg::Receiver(receiver)) => {
            // Find the impl block this method is part of
            match &receiver.self_token {
                _ => {
                    // Use turbofish syntax with generics in the impl block
                    quote! { Self }
                }
            }
        }
        _ => {
            // This is an error - event handlers must be methods
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Event handlers must be methods with &self or &mut self parameter",
            )
            .to_compile_error()
            .into();
        }
    };
    
    // The generated subscription handler
    let output = quote! {
        // Keep the original function
        #input_fn
        
        // Register the subscription handler
        inventory::submit! {
            crate::registry::SubscriptionHandler {
                method_name: #method_name_str.to_string(),
                topic: #topic.to_string(),
                is_full_path: #is_full_path,
                service_type_id: std::any::TypeId::of::<#self_ty>(),
                register_fn: Box::new(|service_ref, ctx| {
                    Box::pin(async move {
                        // Cast to the correct service type
                        let service = service_ref.downcast_ref::<#self_ty>()
                            .ok_or_else(|| anyhow::anyhow!("Service type mismatch in subscribe macro"))?;
                        
                        // Get the service path for topic prefixing if needed
                        let mut full_topic = #topic.to_string();
                        if !#is_full_path {
                            // Prefix with service path to make a full topic
                            let service_path = service.path();
                            let path_prefix = if service_path.starts_with('/') {
                                &service_path[1..]  // Remove leading slash
                            } else {
                                service_path
                            };
                            
                            if !path_prefix.is_empty() {
                                full_topic = format!("{}/{}", path_prefix, full_topic);
                            }
                        }
                        
                        // Create a cloned service for the subscription to avoid ownership issues
                        let mut service_clone = service.clone();
                        
                        // Subscribe to the topic
                        ctx.subscribe(full_topic, move |payload| {
                            let mut service = service_clone.clone();
                            Box::pin(async move {
                                service.#method_name(payload).await
                            })
                        }).await?;
                        
                        Ok(())
                    })
                }),
            }
        }
    };
    
    TokenStream::from(output)
} 