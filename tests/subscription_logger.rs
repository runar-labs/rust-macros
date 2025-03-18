// Subscription logger module for testing
// Provides helper functions to log various pub/sub events to console

/// Log a general subscription event with component, event type, and message
pub fn log_subscription(component: &str, event: &str, message: &str) {
    println!("[{:<10}] [{:<15}] {}", component, event, message);
}

/// Log when a subscription is set up
pub fn log_subscribe(component: &str, topic: &str) {
    log_subscription(component, "SUBSCRIBE", &format!("Subscribing to topic: {}", topic));
}

/// Log when a subscription callback is triggered
pub fn log_callback(component: &str, topic: &str) {
    log_subscription(component, "CALLBACK", &format!("Callback triggered for topic: {}", topic));
}

/// Log when an event is published
pub fn log_event_published(component: &str, topic: &str) {
    log_subscription(component, "PUBLISH", &format!("Publishing event to topic: {}", topic));
}

/// Log an error message
pub fn log_error(component: &str, message: &str) {
    log_subscription(component, "ERROR", message);
} 