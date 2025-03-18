// This file demonstrates how to use the debug utilities
// It's kept separate from the main tests to keep those clean

// Import the debug utils module
mod debug_utils;
use debug_utils::feature_detection::log_feature_status;
use debug_utils::run_all_debug_checks;

#[test]
fn test_debug_utilities() {
    // Log feature status
    log_feature_status();
    
    // Run all debug checks
    run_all_debug_checks();
    
    println!("Debug utilities test completed");
}

// Example simple type (not using any macros)
#[derive(Debug, Default)]
struct DebugService {
    counter: u32,
}

impl DebugService {
    fn new() -> Self {
        DebugService { counter: 0 }
    }
} 