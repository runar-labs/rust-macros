use anyhow::Result;
use chrono;
use runar_macros::{service, action, subscribe};
use runar_node::node::{Node, NodeConfig};
use runar_node::services::{RequestContext, ServiceResponse, ValueType, ResponseStatus};
use runar_node::AbstractService;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::Mutex;
use tokio::test;
use tokio::time::timeout;
use serde_json::json;
use std::collections::HashMap;

// Add module for subscription logging
mod subscription_logger;
use subscription_logger::*;

// Import the common module
mod common;
use common::ServiceInfo;

// Define test timeouts
const TEST_TIMEOUT: Duration = Duration::from_secs(30);
const SERVICE_INIT_TIMEOUT: Duration = Duration::from_millis(500);
const EVENT_PROPAGATION_TIMEOUT: Duration = Duration::from_secs(2);

// Shared storage for events
#[derive(Clone)]
struct EventStorage {
    valid_events: Arc<Mutex<Vec<String>>>,
    invalid_events: Arc<Mutex<Vec<String>>>,
    notifications: Arc<Mutex<Vec<String>>>,
}

impl EventStorage {
    fn new() -> Self {
        Self {
            valid_events: Arc::new(Mutex::new(vec![])),
            invalid_events: Arc::new(Mutex::new(vec![])),
            notifications: Arc::new(Mutex::new(vec![])),
        }
    }
}

// Publisher Service - Using service macro with all attributes
#[service(name = "publisher", path = "publisher")]
#[derive(Clone)]
struct PublisherService {
    storage: EventStorage,
}

impl PublisherService {
    fn new(storage: EventStorage) -> Self {
        Self { storage }
    }

    // Use action macro for validate_event with proper return type
    #[action(name = "validate")]
    async fn validate_event(&self, event_data: String) -> Result<String> {
        // Just a simple validation
        if event_data.contains("valid") {
            Ok("valid".to_string())
        } else {
            // For errors, we can still use anyhow's Error
            Err(anyhow::anyhow!("Event is invalid"))
        }
    }

    // Method to publish an event (called by the process handler)
    async fn publish_event(&self, context: &RequestContext, topic: &str, data: &str) -> Result<()> {
        // Ensure topic includes service name
        let full_topic = if topic.starts_with(&format!("{}/", self.service_name())) {
            topic.to_string()
        } else {
            format!("{}/{}", self.service_name(), topic)
        };

        let event_data = json!({
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        log_event_published(self.service_name(), &full_topic);
        context.publish(&full_topic, event_data).await?;
        log_subscription(
            self.service_name(),
            "PUBLISHED",
            &format!("Event published to topic '{}'", full_topic),
        );

        Ok(())
    }

    // Use action macro for publish_action with proper return type
    #[action(name = "publish")]
    async fn publish_action(&self, context: &RequestContext, topic: String, data: String) -> Result<String> {
        log_subscription(
            self.service_name(),
            "PUBLISH-DATA",
            &format!("Publishing to topic: {} with data: {}", topic, data),
        );

        // Call the publish method
        self.publish_event(context, &topic, &data).await?;

        Ok(format!("Published to topic {}", topic))
    }
    
    // Use action macro for notify_action with proper return type
    #[action(name = "notify")]
    async fn notify_action(&self, context: &RequestContext, message: String) -> Result<String> {
        log_subscription(
            self.service_name(),
            "NOTIFY-ACTION",
            &format!("Sending notification with message: {}", message),
        );

        // Create event data
        let event_data = json!({
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "notification",
        });

        // Directly publish to the notification topic using context
        let topic = format!("{}/notification", self.service_name());
        context.publish(&topic, event_data).await?;
        
        log_subscription(
            self.service_name(),
            "NOTIFICATION-SENT",
            &format!("Notification published to topic '{}'", topic),
        );

        Ok("Notification sent".to_string())
    }
    
    // Use subscribe macro for status updates
    #[subscribe(topic = "listener/status")]
    async fn on_listener_status(&self, payload: ValueType) -> Result<()> {
        log_callback(self.service_name(), "listener/status");
        
        if let ValueType::Json(json_value) = payload {
            if let Some(status) = json_value["status"].as_str() {
                log_subscription(
                    self.service_name(),
                    "STATUS-RECEIVED",
                    &format!("Received status update from listener: {}", status),
                );
                
                // We could do something with this status if needed
            }
        }
        
        Ok(())
    }
}

// Listener Service - Using service macro
#[service(name = "listener", path = "listener")]
#[derive(Clone)]
struct ListenerService {
    storage: Arc<EventStorage>,
}

impl ListenerService {
    pub fn new() -> Self {
        ListenerService {
            storage: Arc::new(EventStorage {
                valid_events: Arc::new(Mutex::new(Vec::new())),
                invalid_events: Arc::new(Mutex::new(Vec::new())),
                notifications: Arc::new(Mutex::new(Vec::new())),
            }),
        }
    }

    // Action to get notifications with proper return type
    #[action(name = "get_notifications")]
    async fn get_notifications(&self) -> Result<usize> {
        let notifications = self.storage.notifications.lock().await;
        Ok(notifications.len())
    }

    #[subscribe(topic = "test/valid")]
    async fn handle_valid_event(&self, payload: ValueType) -> Result<()> {
        println!("Received valid event: {:?}", payload);
        let mut events = self.storage.valid_events.lock().await;
        if let ValueType::String(message) = payload {
            events.push(message);
        }
        Ok(())
    }

    #[subscribe(topic = "test/invalid")]
    async fn handle_invalid_event(&self, payload: ValueType) -> Result<()> {
        println!("Received invalid event: {:?}", payload);
        let mut events = self.storage.invalid_events.lock().await;
        if let ValueType::String(message) = payload {
            events.push(message);
        }
        Ok(())
    }

    #[subscribe(topic = "test/notification")]
    async fn handle_notification(&self, payload: ValueType) -> Result<()> {
        println!("Received notification: {:?}", payload);
        let mut notifications = self.storage.notifications.lock().await;
        if let ValueType::String(message) = payload {
            notifications.push(message);
        }
        Ok(())
    }

    #[action]
    async fn test_action(&self, message: String) -> Result<String> {
        println!("Test action called with message: {}", message);
        Ok(format!("Processed: {}", message))
    }

    #[action(name = "get_valid_events")]
    async fn get_valid_events(&self) -> Result<Vec<String>> {
        let events = self.storage.valid_events.lock().await;
        Ok(events.clone())
    }

    #[action(name = "get_invalid_events")]
    async fn get_invalid_events(&self) -> Result<Vec<String>> {
        let events = self.storage.invalid_events.lock().await;
        Ok(events.clone())
    }
}

#[tokio::test]
async fn test_p2p_publish_subscribe() -> Result<()> {
    timeout(TEST_TIMEOUT, async {
        log_subscription("TEST", "START", "Starting publish/subscribe test with macros");

        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");

        // Configure and create node (with minimal configuration)
        let config = NodeConfig::new(
            "test_network",
            temp_dir.path().to_str().unwrap(),
            db_path.to_str().unwrap(),
        );
        
        let mut node = Node::new(config).await?;

        // Initialize the node
        log_subscription("TEST", "NODE-INIT", "Initializing node");
        timeout(SERVICE_INIT_TIMEOUT, node.init()).await??;

        // Create our services with shared storage
        let storage = EventStorage::new();
        let publisher_service = PublisherService::new(storage.clone());
        let listener_service = ListenerService::new();

        // Add the services to the node
        log_subscription(
            "TEST",
            "ADD-SERVICES",
            "Adding publisher and listener services to node",
        );
        timeout(SERVICE_INIT_TIMEOUT, node.add_service(publisher_service)).await??;
        timeout(SERVICE_INIT_TIMEOUT, node.add_service(listener_service)).await??;

        // Wait a bit for the services to initialize
        log_subscription("TEST", "WAIT", "Waiting for services to initialize");
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Publish a valid event
        let valid_payload = json!({
            "topic": "valid",
            "data": "this is a valid event"
        });
        log_subscription("TEST", "PUBLISH-VALID", "Sending valid event request");
        timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("publisher/publish", valid_payload)
        ).await??;

        // Publish an invalid event
        let invalid_payload = json!({
            "topic": "invalid",
            "data": "this is an invalid event"
        });
        log_subscription("TEST", "PUBLISH-INVALID", "Sending invalid event request");
        timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("publisher/publish", invalid_payload)
        ).await??;
        
        // Send a notification (using context to publish directly)
        let notification_payload = json!({
            "message": "This is a direct notification via context"
        });
        log_subscription("TEST", "SEND-NOTIFICATION", "Sending notification request");
        timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("publisher/notify", notification_payload)
        ).await??;
        
        // Send a status update from listener service (testing auto-generated process_request with context)
        let status_payload = json!({
            "status": "System is operational and processing events"
        });
        log_subscription("TEST", "SEND-STATUS", "Testing auto-generated process handling context parameter");
        timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("listener/send_status", status_payload)
        ).await??;

        // Wait for events to be processed
        log_subscription("TEST", "WAIT", "Waiting for events to be processed");
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check that the listener received the events
        log_subscription("TEST", "CHECK-VALID", "Checking valid events");
        let valid_events_response = timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("listener/get_valid_events", json!({}))
        ).await??;
        log_subscription(
            "TEST",
            "VALID-RESPONSE",
            &format!("Valid events response: {:?}", valid_events_response),
        );

        log_subscription("TEST", "CHECK-INVALID", "Checking invalid events");
        let invalid_events_response = timeout(
            EVENT_PROPAGATION_TIMEOUT,
            node.request("listener/get_invalid_events", json!({}))
        ).await??;
        log_subscription(
            "TEST",
            "INVALID-RESPONSE",
            &format!("Invalid events response: {:?}", invalid_events_response),
        );

        // Extract the events and verify
        if let Some(valid_data) = valid_events_response.data {
            // Using ValueType directly since ServiceResponse contains data as ValueType
            let valid_events: Vec<String> = if let ValueType::Array(events) = valid_data {
                events.into_iter()
                    .filter_map(|e| if let ValueType::String(s) = e { Some(s) } else { None })
                    .collect()
            } else if let ValueType::String(json_str) = valid_data {
                serde_json::from_str(&json_str)?
            } else {
                vec![]
            };
            assert!(valid_events.len() >= 2, "Should have received at least 2 valid events");
            
            // Check for the valid event
            let has_valid_event = valid_events.iter().any(|event| {
                event.contains("valid")
            });
            assert!(has_valid_event, "Should have a valid event");
            
            // Check for the status update
            let has_status_update = valid_events.iter().any(|event| {
                event.contains("STATUS:")
            });
            assert!(has_status_update, "Should have a status update event");
            
            // Check for notification if it was stored as valid
            let has_notification = valid_events.iter().any(|event| {
                event.contains("NOTIFICATION:")
            });
            
            if has_notification {
                println!("Found notification in valid events as expected");
            }
        }

        if let Some(invalid_data) = invalid_events_response.data {
            // Using ValueType directly since ServiceResponse contains data as ValueType
            let invalid_events: Vec<String> = if let ValueType::Array(events) = invalid_data {
                events.into_iter()
                    .filter_map(|e| if let ValueType::String(s) = e { Some(s) } else { None })
                    .collect()
            } else if let ValueType::String(json_str) = invalid_data {
                serde_json::from_str(&json_str)?
            } else {
                vec![]
            };
            assert_eq!(invalid_events.len(), 1, "Should have received 1 invalid event");
            if let Some(invalid_event) = invalid_events.get(0) {
                assert!(invalid_event.contains("invalid"), "Invalid event should contain 'invalid'");
            }
        }

        log_subscription("TEST", "COMPLETE", "Test completed successfully");
        Ok(())
    }).await?
}

#[test]
async fn test_service_info_trait() {
    let storage = EventStorage::new();
    let publisher = PublisherService::new(storage.clone());
    let listener = ListenerService::new();
    
    // Verify service_name works correctly
    assert_eq!("publisher", publisher.service_name());
    assert_eq!("listener", listener.service_name());
    
    // Verify service_path is correctly set
    assert_eq!("publisher", publisher.service_path());
    assert_eq!("listener", listener.service_path());
    
    // Verify default values are used for description and version
    assert_eq!("Service PublisherService", publisher.service_description());
    assert_eq!("Service ListenerService", listener.service_description());
    assert_eq!("0.1.0", publisher.service_version());
    assert_eq!("0.1.0", listener.service_version());
}

#[tokio::test]
async fn test_service_macros() {
    // Create services
    let storage = EventStorage::new();
    let publisher = PublisherService::new(storage.clone());
    let listener = ListenerService::new();

    // Test directly calling the test_action method
    let message = "Test message".to_string();
    let response = listener.test_action(message).await.unwrap();
    
    // Verify the response
    assert_eq!(response, "Processed: Test message");
} 