use proc_macro::TokenStream;

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

/// Macro for defining a service
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    service::service(args, input)
}

/// Macro for defining an action handler
#[proc_macro_attribute]
pub fn action(args: TokenStream, input: TokenStream) -> TokenStream {
    action::action(args, input)
}

/// Macro for defining a subscription handler
#[proc_macro_attribute]
pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    subscribe::subscribe(args, input)
}

/// Macro for publishing an event
#[proc_macro_attribute]
pub fn publish(args: TokenStream, input: TokenStream) -> TokenStream {
    publish::publish(args, input)
}

/// Macro for defining a gateway
#[proc_macro_attribute]
pub fn gateway(args: TokenStream, input: TokenStream) -> TokenStream {
    gateway::gateway(args, input)
}

/// Macro for defining a middleware
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    middleware::middleware(args, input)
}

/// Macro for defining an initialization handler
#[proc_macro_attribute]
pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    init::init(args, input)
}

/// Macro for defining event handlers (alias for subscribe)
#[proc_macro_attribute]
pub fn events(args: TokenStream, input: TokenStream) -> TokenStream {
    subscribe::subscribe(args, input)
}