use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Meta, parse::Parser, Ident};
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
/// async fn handle_user_created(&self, data: ValueType) -> Result<(), anyhow::Error> {
///     // Extract data from the payload
///     if let ValueType::Map(data) = &data {
///         // Process the data
///     }
///     
///     Ok(())
/// }
///
/// // Subscribe using a full path
/// #[subscribe(topic = "users/user_created")]
/// async fn handle_user_event(&self, data: ValueType) -> Result<(), anyhow::Error> {
///     // Handler implementation
///     Ok(())
/// }
///
/// // Default subscription using method name as topic
/// #[subscribe]
/// async fn custom_event(&self, data: ValueType) -> Result<(), anyhow::Error> {
///     // Handler implementation
///     Ok(())
/// }
/// ```
///
/// # Requirements
/// - The service must implement `Clone` to support multiple subscription handlers
/// - Handler methods must be `async` and return `Result<(), anyhow::Error>`
/// - Handler methods should take `&self` and a `ValueType` parameter
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
    
    // Generate a registration method name that will be called during init
    let register_method_name = format_ident!("register_{}_subscription", method_name);
    
    // Generate the subscription registration implementation
    let output = quote! {
        // Keep the original function
        #input_fn
        
        // Generate a registration method that will be called during service initialization
        async fn #register_method_name(&mut self, context: &runar_node::services::RequestContext) -> anyhow::Result<()> {
            // Get the full topic path, prefixing with service path if needed
            let full_topic = if #is_full_path {
                #topic.to_string()
            } else {
                // Need to prefix with service path
                let service_path = self.path();
                format!("{}/{}", service_path, #topic)
            };
            
            // Create a cloned instance for the subscription callback
            // This requires that the service implements Clone
            let service_clone = self.clone();
            
            // Subscribe to the topic
            context.subscribe(&full_topic, move |payload| {
                let service = service_clone.clone();
                Box::pin(async move {
                    service.#method_name(payload).await
                })
            }).await?;
            
            Ok(())
        }
        
        // Register this subscription in the service subscription registry
        inventory::submit! {
            crate::registry::SubscriptionRegistry {
                type_id: std::any::TypeId::of::<Self>(),
                topic: #topic.to_string(),
                is_full_path: #is_full_path,
                registration_method: stringify!(#register_method_name).to_string(),
            }
        }
    };
    
    TokenStream::from(output)
} 