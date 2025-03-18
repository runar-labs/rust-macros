# Debugging Kagi Macros

This document provides practical tips and techniques for diagnosing and fixing issues with the Kagi macros.

## Common Debug Techniques

### 1. Print Macro Arguments

Adding debug prints to see what's being passed to the macro:

```rust
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("service macro called with args: {:?}", args);
    println!("input: {:?}", input);
    
    // Rest of the implementation...
}
```

### 2. View Expanded Macro Code

Use `cargo expand` to see the generated code:

```bash
# Install cargo expand if you don't have it
cargo install cargo-expand

# Expand the code for a specific test
cargo expand --test minimal_service_test
```

This shows you exactly what code the macro is generating.

### 3. Step-by-Step Tracing

Add trace points throughout the macro to track execution flow:

```rust
// In your macro implementation
println!("Parsing attributes...");
let args = syn::parse::<MacroArgs>(args).unwrap_or_else(|e| {
    println!("Parse error: {}", e);
    // Handle error...
});
println!("Attributes parsed: {:?}", args);

// Continue with more trace points...
```

## Debugging Registration Approaches

Kagi macros support two registration approaches:

1. **Distributed Slices (Compile-time)**: Using the `linkme` crate
2. **Runtime Registration (Default)**: A fallback mechanism for testing

### Debugging Distributed Slices

If you're having issues with the distributed slices approach:

1. Ensure the `linkme` feature is enabled
2. Check that the `distributed_slice` attribute is correctly applied
3. Verify that the static item has the correct type

Example debugging:

```rust
// Adding debug prints to see if distributed slice code is being generated
println!("Generating distributed slice code");
let distributed_code = quote! {
    #[distributed_slice(ACTION_REGISTRY)]
    static #static_ident: fn() -> kagi_node::services::ActionHandler = || {
        kagi_node::services::ActionHandler {
            // Implementation details...
        }
    };
};
println!("Generated distributed slice code: {}", distributed_code);
```

### Debugging Runtime Registration

For issues with the runtime registration approach:

1. Check that the initializer registration code is being generated
2. Verify that the node is properly executing initializers
3. Trace the registration process with debug statements

Example debugging:

```rust
// Adding debug prints to see if runtime registration code is being generated
println!("Generating runtime registration code");
let runtime_code = quote! {
    #[cfg(feature = "node_implementation")]
    kagi_node::init::register_initializer(kagi_node::init::Initializer {
        // Implementation details...
    });
};
println!("Generated runtime registration code: {}", runtime_code);
```

## Service Macro Debugging

### Common Issues

1. **Attribute Parsing Failures**
   - Check that attribute parameters match the expected format: `name = "value"`
   - Ensure string literals are properly quoted
   - Verify the attribute parser handles commas correctly

2. **Missing ServiceInfo Trait**
   - Make sure the `ServiceInfo` trait is in scope where the macro is applied
   - Check that the implementation matches what's expected by the trait

3. **Token Stream Handling**
   - If the expanded code has syntax errors, you may be incorrectly handling the TokenStream
   - Check for unbalanced braces, missing semicolons, etc.

### Diagnostic Techniques

For complex service macro issues:

```rust
// Print the final generated code for inspection
let expanded = quote! { /* your generated code */ };
println!("Expanded code: {}", expanded);
```

## Action Macro Debugging

### Common Issues

1. **Method Signature Mismatches**
   - Ensure the method has the correct return type (usually `Result<ServiceResponse>`)
   - Check that async/await is properly handled

2. **Parameter Extraction**
   - Verify parameters are correctly extracted from the RequestContext
   - Check for type conversion errors

3. **Operation Matching Logic**
   - Check that operations are correctly routed to their handlers
   - Verify string comparison logic is working as expected

4. **Parameter Extraction and Passing**
   - Ensure parameters are correctly extracted and passed to action handlers
   - Check for type conversions and default values

### Diagnostic Techniques

```rust
// In your action macro
println!("Method signature: {:?}", method_sig);
println!("Method parameters: {:?}", params);
println!("Matched operation: {}", operation);
println!("Extracted parameters: {:?}", params);
```

## Event Macros Debugging

### Subscribe Macro

1. **Topic String Formatting**
   - Verify topic strings are correctly formatted
   - Check that service name prefixing is handled properly

2. **Handler Registration**
   - Ensure handler functions are correctly registered with the subscription system
   - Check for lifetime issues with closures

### Publish Macro

1. **Event Data Serialization**
   - Check that event data is properly converted to the expected format
   - Verify metadata is included as expected

2. **Topic Construction**
   - Ensure topics include service name when appropriate
   - Check for proper escaping of special characters

## Testing Environment Debugging

When debugging in test environments:

1. **Feature Flags**: Test environments often run without certain features enabled; add debug prints to check which codepath is being taken:

```rust
#[cfg(feature = "distributed_slice")]
println!("Using distributed slice approach");

#[cfg(not(feature = "distributed_slice"))]
println!("Using runtime registration approach");
```

2. **Runtime Registry Initialization**: Check if runtime registry is being initialized properly:

```rust
// In your test
println!("Before initializing runtime registrations");
registry.init_runtime_registrations().await?;
println!("After initializing runtime registrations");
```

3. **Handler Registration**: Verify handlers are being registered correctly:

```rust
// In your register_initializer function
println!("Registering initializer: {:?}", initializer.name);
```

## Advanced Debugging

### Using Custom Spans for Better Error Messages

```rust
// Create a custom error with the span of the problematic token
let error = syn::Error::new(
    problematic_token.span(),
    "Detailed error message explaining the issue"
);
return error.to_compile_error().into();
```

### Debugging Type Resolution Issues

For complex type resolution problems:

```rust
// Print detailed type information
println!("Type: {:#?}", type_path);
```

## Troubleshooting Checklist

When a macro isn't working as expected:

1. ✅ Check that the macro is being called with the expected arguments
2. ✅ Examine the expanded code using `cargo expand`
3. ✅ Add debug prints to trace execution flow
4. ✅ Verify attribute parsing is working correctly
5. ✅ Check that generated code references the correct types
6. ✅ Ensure imports and dependencies are available
7. ✅ Test with minimal examples before complex cases
8. ✅ Check which registration approach is being used (distributed slice or runtime)
9. ✅ Verify that initializers are being called
10. ✅ Ensure that service registry can find and route to your service 

## Debugging Macros

Debugging procedural macros can be challenging because they run at compile time. Here are some strategies:

### 1. Use the `cargo-expand` tool

Install it with:

```bash
cargo install cargo-expand
```

Then run:

```bash
cargo expand --package kagi_macros
```

This will show the expanded code after macro processing.

### 2. Use the `macro_debug` binary

The `kagi_macros` crate includes a binary target that can be used to debug the macros. You can run it with:

```bash
cargo run --bin macro_debug --features=node_implementation
```

This will print the generated code for a sample service.

### 3. Use the `println!` macro in your procedural macros

You can add `println!` statements to your procedural macros to see what's happening during compilation:

```rust
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("service macro called with args: {:?}", args);
    // ... rest of the macro implementation
}
```

### 4. Examine the generated code

When the `node_implementation` feature is enabled, the macros generate runtime registration code like:

```rust
#[cfg(feature = "node_implementation")]
pub fn register_initializer(registry: &kagi_node::init::InitializerRegistry) {
    registry.register_initializer(
        "my_service",
        Box::new(|node| {
            Box::pin(async move {
                let service = MyService::new();
                node.register_service(service).await?;
                Ok(())
            })
        }),
    );
}
``` 