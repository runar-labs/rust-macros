use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, ImplItemFn, ItemImpl, ImplItem, LitStr, parse::Parse, parse::ParseStream};
use rand::{thread_rng, Rng};

// Parse either an ItemImpl or an ItemFn
enum EventInput {
    Impl(ItemImpl),
    Fn(ItemFn),
}

impl Parse for EventInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![impl]) {
            Ok(EventInput::Impl(input.parse()?))
        } else {
            Ok(EventInput::Fn(input.parse()?))
        }
    }
}

pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the topic argument from the attribute
    let topic = match syn::parse::<LitStr>(args.clone()) {
        Ok(lit_str) => lit_str.value(),
        Err(_) => "test_topic".to_string()  // Default topic for testing
    };
    
    let input_parsed = parse_macro_input!(input as EventInput);
    
    match input_parsed {
        EventInput::Fn(input_fn) => {
            // For standalone functions, we just return the function as is
            // The registration will be handled by the service macro
            TokenStream::from(quote! {
                #input_fn
            })
        },
        EventInput::Impl(mut impl_block) => {
            // For impl blocks, we need to modify the functions directly
            let mut new_items = Vec::new();
            
            // Extract the service name from the impl type
            let self_ty = &impl_block.self_ty;
            
            // Keep track of which functions have the subscribe attribute
            let mut subscribe_fns = Vec::new();
            
            for item in impl_block.items {
                match item {
                    ImplItem::Fn(mut fn_item) if has_subscribe_attr(&fn_item) => {
                        // Get the function name
                        let fn_name = &fn_item.sig.ident;
                        let fn_name_str = format!("{}", fn_name);
                        
                        // Get the topic from the attribute if available
                        let args_str = args.to_string();
                        let topic_str = if is_parameterless(&fn_item) {
                            // Parameterless form - use method name as topic
                            fn_name_str.clone()
                        } else {
                            // Named form - extract topic from attribute
                            get_topic_from_attr(&fn_item, &fn_name_str)
                        };
                        
                        // Store this for later
                        subscribe_fns.push((fn_name.clone(), fn_name_str, topic_str));
                        
                        // Remove the #[subscribe] attribute
                        fn_item.attrs.retain(|attr| !is_subscribe_attr(attr));
                        
                        // Keep the function as is (just remove the attribute)
                        new_items.push(ImplItem::Fn(fn_item));
                    },
                    _ => {
                        // Keep other items unchanged
                        new_items.push(item);
                    }
                }
            }
            
            // Update the impl block with the modified items
            impl_block.items = new_items;
            
            // Generate a unique identifier for this impl block
            let random_suffix: u32 = thread_rng().gen();
            let register_fn_name = format_ident!("register_subscriptions_{}", random_suffix);
            
            // Create a method to register subscriptions
            let subscription_registrations = subscribe_fns.iter().map(|(fn_name, fn_name_str, topic_str)| {
                quote! {
                    println!("Registering subscription '{}' for topic '{}' in service '{}'", 
                        #fn_name_str, #topic_str, service_name);
                    
                    let subscription = kagi_node::services::EventSubscription {
                        topic: #topic_str.to_string(),
                        handler: |payload| {
                            let this = service_instance.clone();
                            Box::pin(async move {
                                this.#fn_name(payload).await
                            })
                        },
                        service: service_name.clone(),
                    };
                    
                    if let Err(e) = kagi_node::init::register_subscription(subscription).await {
                        println!("Failed to register subscription '{}' for topic '{}' in service '{}': {}", 
                            #fn_name_str, #topic_str, service_name, e);
                    }
                }
            });
            
            // Add a new method to the impl block for registering subscriptions
            let register_method = quote! {
                #[cfg(feature = "kagi_node")]
                pub async fn #register_fn_name(service_instance: std::sync::Arc<Self>) {
                    let service_name = service_instance.name().to_string();
                    
                    // Register each subscription
                    #(#subscription_registrations)*
                }
            };
            
            // Return the modified impl block and the registration method
            TokenStream::from(quote! {
                #impl_block
                
                #register_method
            })
        }
    }
}

pub fn publish(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the topic argument from the attribute
    let topic = match syn::parse::<LitStr>(args) {
        Ok(lit_str) => lit_str.value(),
        Err(_) => "test_topic".to_string()  // Default topic for testing
    };
    
    let input_parsed = parse_macro_input!(input as EventInput);
    
    match input_parsed {
        EventInput::Fn(input_fn) => {
            // For standalone functions, we just return the function as is
            // The registration will be handled by the service macro
            TokenStream::from(quote! {
                #input_fn
            })
        },
        EventInput::Impl(mut impl_block) => {
            // For impl blocks, we need to modify the functions directly
            let mut new_items = Vec::new();
            
            // Extract the service name from the impl type
            let self_ty = &impl_block.self_ty;
            
            // Keep track of which functions have the publish attribute
            let mut publish_fns = Vec::new();
            
            for item in impl_block.items {
                match item {
                    ImplItem::Fn(mut fn_item) if has_publish_attr(&fn_item) => {
                        // Get the function name
                        let fn_name = &fn_item.sig.ident;
                        let fn_name_str = format!("{}", fn_name);
                        
                        // Get the topic from the attribute if available
                        let topic_str = get_topic_from_attr(&fn_item, &topic);
                        
                        // Store this for later
                        publish_fns.push((fn_name.clone(), fn_name_str, topic_str));
                        
                        // Remove the #[publish] attribute
                        fn_item.attrs.retain(|attr| !is_publish_attr(attr));
                        
                        // Keep the function as is (just remove the attribute)
                        new_items.push(ImplItem::Fn(fn_item));
                    },
                    _ => {
                        // Keep other items unchanged
                        new_items.push(item);
                    }
                }
            }
            
            // Update the impl block with the modified items
            impl_block.items = new_items;
            
            // Generate a unique identifier for this impl block
            let random_suffix: u32 = thread_rng().gen();
            let register_fn_name = format_ident!("register_publications_{}", random_suffix);
            
            // Create a method to register publications
            let publication_registrations = publish_fns.iter().map(|(fn_name, fn_name_str, topic_str)| {
                quote! {
                    println!("Registering publication '{}' for topic '{}' in service '{}'", 
                        #fn_name_str, #topic_str, service_name);
                    
                    let publication = kagi_node::services::PublicationInfo {
                        topic: #topic_str.to_string(),
                        service: service_name.clone(),
                        description: format!("Publication for topic {} in service {}", #topic_str, service_name),
                    };
                    
                    if let Err(e) = kagi_node::init::register_publication(publication).await {
                        println!("Failed to register publication '{}' for topic '{}' in service '{}': {}", 
                            #fn_name_str, #topic_str, service_name, e);
                    }
                }
            });
            
            // Add a new method to the impl block for registering publications
            let register_method = quote! {
                #[cfg(feature = "kagi_node")]
                pub async fn #register_fn_name(service_instance: std::sync::Arc<Self>) {
                    let service_name = service_instance.name().to_string();
                    
                    // Register each publication
                    #(#publication_registrations)*
                }
            };
            
            // Add a trait implementation for AbstractService that calls our registration methods
            let trait_impl = quote! {
                #[cfg(feature = "kagi_node")]
                impl kagi_node::services::AbstractService for #self_ty {
                    fn name(&self) -> &str {
                        self.name()
                    }
                    
                    fn path(&self) -> &str {
                        self.path()
                    }
                    
                    fn description(&self) -> &str {
                        self.description()
                    }
                    
                    fn version(&self) -> &str {
                        self.version()
                    }
                    
                    async fn initialize(&mut self) -> kagi_node::Result<()> {
                        // Register subscriptions and publications when the service is initialized
                        let instance = std::sync::Arc::new(self.clone());
                        
                        // Register subscriptions if the method exists
                        if let Some(_) = Self::register_subscriptions_method() {
                            Self::#register_fn_name(instance.clone()).await;
                        }
                        
                        // Register publications if any
                        Self::#register_fn_name(instance).await;
                        
                        Ok(())
                    }
                    
                    async fn start(&mut self) -> kagi_node::Result<()> {
                        Ok(())
                    }
                    
                    async fn stop(&mut self) -> kagi_node::Result<()> {
                        Ok(())
                    }
                }
                
                #[cfg(feature = "kagi_node")]
                impl #self_ty {
                    // Helper method to check if the register_subscriptions method exists
                    fn register_subscriptions_method() -> Option<()> {
                        None
                    }
                }
            };
            
            // Return the modified impl block, the registration method, and the trait implementation
            TokenStream::from(quote! {
                #impl_block
                
                #register_method
                
                #trait_impl
            })
        }
    }
}

// Helper function to check if an attribute is a subscribe attribute
fn is_subscribe_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("subscribe")
}

// Helper function to check if a function has a subscribe attribute
fn has_subscribe_attr(fn_item: &ImplItemFn) -> bool {
    fn_item.attrs.iter().any(is_subscribe_attr)
}

// Helper function to check if an attribute is a publish attribute
fn is_publish_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("publish")
}

// Helper function to check if a function has a publish attribute
fn has_publish_attr(fn_item: &ImplItemFn) -> bool {
    fn_item.attrs.iter().any(is_publish_attr)
}

// Helper function to extract topic from attribute
fn get_topic_from_attr(fn_item: &ImplItemFn, default_topic: &str) -> String {
    for attr in &fn_item.attrs {
        if attr.path().is_ident("subscribe") || attr.path().is_ident("publish") {
            let attr_tokens = attr.meta.require_list().ok().map(|meta| meta.tokens.clone());
            if let Some(tokens) = attr_tokens {
                let parser = syn::parse2::<syn::MetaNameValue>(tokens);
                if let Ok(name_value) = parser {
                    if name_value.path.is_ident("topic") {
                        if let syn::Expr::Lit(expr_lit) = &name_value.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                return lit_str.value();
                            }
                        }
                    }
                }
            }
        }
    }
    
    default_topic.to_string()
}

// Helper function to check if an attribute is a subscribe attribute with no parameters
fn is_parameterless(fn_item: &ImplItemFn) -> bool {
    fn_item.attrs.iter().any(|attr| 
        attr.path().is_ident("subscribe") && 
        attr.meta.require_list().is_err() // No args means no list
    )
} 