use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
use quote::quote;

// Internal modules - private, not exported
mod action;
mod action_registry;
mod events;
mod gateway;
mod init;
mod middleware;
mod publish;
mod service;
mod subscribe;
mod subscription_registry;
mod utils;
mod registry; // Change from public to private

/// Macro for defining a service
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    service::service(args, input)
}

/// Macro for defining an action handler
#[proc_macro_attribute]
pub fn action(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input as a function
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // For simplicity, just output the original function for now
    let output = quote! {
        #input_fn
    };
    
    output.into()
}

/// Macro for defining a subscription handler
#[proc_macro_attribute]
pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}

/// Macro for publishing an event
#[proc_macro_attribute]
pub fn publish(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}

/// Macro for defining a gateway
#[proc_macro_attribute]
pub fn gateway(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}

/// Macro for defining a middleware
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}

/// Macro for defining an initialization handler
#[proc_macro_attribute]
pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}

/// Macro for defining event handlers (alias for subscribe)
#[proc_macro_attribute]
pub fn events(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through for now
    input
}