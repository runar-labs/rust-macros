// Debug utilities for the kagi_macros crate

/// Module for detecting and reporting on feature configurations
pub mod feature_detection {
    pub static FEATURE_NODE_IMPLEMENTATION: &str = if cfg!(feature = "node_implementation") {
        "enabled"
    } else {
        "disabled"
    };

    pub static FEATURE_WEB: &str = if cfg!(feature = "web") {
        "Enabled"
    } else {
        "Disabled"
    };

    pub static FEATURE_MOBILE: &str = if cfg!(feature = "mobile") {
        "Enabled"
    } else {
        "Disabled"
    };

    pub static FEATURE_DESKTOP: &str = if cfg!(feature = "desktop") {
        "Enabled"
    } else {
        "Disabled"
    };
    
    // Add the missing constants
    pub static FEATURE_KAGI_NODE: &str = if cfg!(feature = "kagi_node") {
        "Enabled"
    } else {
        "Disabled"
    };
    
    pub static FEATURE_DISTRIBUTED_SLICE: &str = if cfg!(feature = "distributed_slice") {
        "Enabled"
    } else {
        "Disabled"
    };
    
    pub static FEATURE_LINKME: &str = if cfg!(feature = "linkme") {
        "Enabled"
    } else {
        "Disabled"
    };

    /// Log the current feature status
    pub fn log_feature_status() {
        println!("Feature Status:");
        println!("  - feature=\"node_implementation\": {}", FEATURE_NODE_IMPLEMENTATION);
        println!("  - feature=\"web\": {}", FEATURE_WEB);
        println!("  - feature=\"mobile\": {}", FEATURE_MOBILE);
        println!("  - feature=\"desktop\": {}", FEATURE_DESKTOP);
        println!("  - feature=\"kagi_node\": {}", FEATURE_KAGI_NODE);
        println!("  - feature=\"distributed_slice\": {}", FEATURE_DISTRIBUTED_SLICE);
        println!("  - feature=\"linkme\": {}", FEATURE_LINKME);
    }

    pub fn check_registry_types() {
        println!("Checking registry types...");
        
        #[cfg(feature = "node_implementation")]
        {
            println!("Node registry types should be available");
        }
        
        #[cfg(not(feature = "node_implementation"))]
        {
            println!("Node registry types are NOT available");
        }
    }
}

/// Run all debug checks at once
pub fn run_all_debug_checks() {
    println!("Running all debug checks...");
    
    // Log feature status
    feature_detection::log_feature_status();
    
    // Check registry types
    feature_detection::check_registry_types();
    
    println!("All debug checks completed");
}

pub mod debug_utils {
    use super::feature_detection;

    // Display the feature configuration
    pub fn print_feature_configuration() {
        println!("Feature Configuration:");
        println!("  - feature=\"node_implementation\": {}", feature_detection::FEATURE_NODE_IMPLEMENTATION);
        println!("  - feature=\"web\": {}", feature_detection::FEATURE_WEB);
        println!("  - feature=\"mobile\": {}", feature_detection::FEATURE_MOBILE);
        println!("  - feature=\"desktop\": {}", feature_detection::FEATURE_DESKTOP);
        println!("  - feature=\"kagi_node\": {}", feature_detection::FEATURE_KAGI_NODE);
        println!("  - feature=\"distributed_slice\": {}", feature_detection::FEATURE_DISTRIBUTED_SLICE);
        println!("  - feature=\"linkme\": {}", feature_detection::FEATURE_LINKME);
    }

    // Function to create a test service (only available when node_implementation feature is enabled)
    #[cfg(feature = "node_implementation")]
    pub fn create_test_service() {
        // ... existing code ...
    }

    // Placeholder when node_implementation feature is not enabled
    #[cfg(not(feature = "node_implementation"))]
    pub fn create_test_service() {
        // ... existing code ...
    }
}

#[cfg(test)]
pub mod registry_type_detection {
    // ... existing code ...
    
    pub fn check_registry_types() {
        // Use the main feature_detection module
        use super::feature_detection;
        
        println!("Registry types:");
        println!("  FEATURE_NODE_IMPLEMENTATION: {}", feature_detection::FEATURE_NODE_IMPLEMENTATION);
        println!("  FEATURE_KAGI_NODE: {}", feature_detection::FEATURE_KAGI_NODE);
        println!("  FEATURE_DISTRIBUTED_SLICE: {}", feature_detection::FEATURE_DISTRIBUTED_SLICE);
        println!("  FEATURE_LINKME: {}", feature_detection::FEATURE_LINKME);
    }
} 