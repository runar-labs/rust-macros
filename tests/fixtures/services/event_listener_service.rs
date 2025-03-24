use anyhow::Result;
use runar_macros::{action, service, subscription};
use runar_node::{RequestContext, ValueType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Storage for received events
#[derive(Clone)]
pub struct EventListenerStorage {
    received_events: Arc<Mutex<HashMap<String, Vec<ValueType>>>>,
    event_count: Arc<Mutex<usize>>,
}

impl EventListenerStorage {
    pub fn new() -> Self {
        Self {
            received_events: Arc::new(Mutex::new(HashMap::new())),
            event_count: Arc::new(Mutex::new(0)),
        }
    }
}

/// A service for listening to events
/// This implementation uses the Runar macros
#[service(
    name = "event_listener_service",
    path = "event_listener",
    description = "Event listening service for testing",
    version = "1.0"
)]
pub struct MacroEventListenerService {
    name: String,
    storage: EventListenerStorage,
}

impl MacroEventListenerService {
    /// Create a new MacroEventListenerService
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            storage: EventListenerStorage::new(),
        }
    }

    // The service macro automatically implements start, stop, state, and name methods
    // No explicit init needed as the service macro handles subscription registration

    /// Handle valid events
    #[subscription(path = "event_emitter/valid")]
    pub async fn handle_valid_event(&self, payload: ValueType) -> Result<()> {
        // Store the event as valid
        let mut received_events = self.storage.received_events.lock().unwrap();
        received_events.entry("valid".to_string())
            .or_insert_with(Vec::new)
            .push(payload);
        
        // Increment total event count
        let mut count = self.storage.event_count.lock().unwrap();
        *count += 1;
        
        Ok(())
    }
    
    /// Handle invalid events
    #[subscription(path = "event_emitter/invalid")]
    pub async fn handle_invalid_event(&self, payload: ValueType) -> Result<()> {
        // Store the event as invalid
        let mut received_events = self.storage.received_events.lock().unwrap();
        received_events.entry("invalid".to_string())
            .or_insert_with(Vec::new)
            .push(payload);
        
        // Increment total event count
        let mut count = self.storage.event_count.lock().unwrap();
        *count += 1;
        
        Ok(())
    }

    /// Get all received events
    #[action(path = "get_all_events")]
    pub async fn get_all_events(&self) -> Result<ValueType> {
        let received_events = self.storage.received_events.lock().unwrap();
        let count = *self.storage.event_count.lock().unwrap();
        
        // Convert the HashMap to a format suitable for ValueType
        let mut topics = Vec::new();
        for (topic, events) in received_events.iter() {
            let mut topic_map = HashMap::new();
            topic_map.insert("topic".to_string(), ValueType::String(topic.clone()));
            topic_map.insert("events".to_string(), ValueType::Array(events.clone()));
            topic_map.insert("count".to_string(), ValueType::Number(events.len() as f64));
            topics.push(ValueType::Map(topic_map));
        }
        
        let mut result = HashMap::new();
        result.insert("total_count".to_string(), ValueType::Number(count as f64));
        result.insert("topics".to_string(), ValueType::Array(topics));
        
        Ok(ValueType::Map(result))
    }

    /// Get events for a specific topic
    #[action(path = "get_events_by_topic")]
    pub async fn get_events_by_topic(&self, topic: String) -> Result<ValueType> {
        let received_events = self.storage.received_events.lock().unwrap();
        
        let mut result = HashMap::new();
        result.insert("topic".to_string(), ValueType::String(topic.clone()));
        
        if let Some(events) = received_events.get(&topic) {
            result.insert("events".to_string(), ValueType::Array(events.clone()));
            result.insert("count".to_string(), ValueType::Number(events.len() as f64));
        } else {
            result.insert("events".to_string(), ValueType::Array(Vec::new()));
            result.insert("count".to_string(), ValueType::Number(0.0));
        }
        
        Ok(ValueType::Map(result))
    }

    /// Check if a specific event was received
    #[action(path = "has_received_event")]
    pub async fn has_received_event(&self, topic: String, data_pattern: String) -> Result<ValueType> {
        let received_events = self.storage.received_events.lock().unwrap();
        
        let mut result = HashMap::new();
        result.insert("topic".to_string(), ValueType::String(topic.clone()));
        result.insert("pattern".to_string(), ValueType::String(data_pattern.clone()));
        
        if let Some(events) = received_events.get(&topic) {
            // Check if any event contains the data pattern
            for event in events {
                if let ValueType::Map(map) = event {
                    if let Some(ValueType::String(data)) = map.get("data") {
                        if data.contains(&data_pattern) {
                            result.insert("found".to_string(), ValueType::Bool(true));
                            return Ok(ValueType::Map(result));
                        }
                    }
                }
            }
        }
        
        // Event not found
        result.insert("found".to_string(), ValueType::Bool(false));
        
        Ok(ValueType::Map(result))
    }

    /// Clear all received events
    #[action(path = "clear_events")]
    pub async fn clear_events(&self) -> Result<ValueType> {
        let mut received_events = self.storage.received_events.lock().unwrap();
        let old_count = *self.storage.event_count.lock().unwrap();
        
        // Clear all events
        received_events.clear();
        
        // Reset count
        let mut count = self.storage.event_count.lock().unwrap();
        *count = 0;
        
        let mut result = HashMap::new();
        result.insert("cleared".to_string(), ValueType::Bool(true));
        result.insert("previous_count".to_string(), ValueType::Number(old_count as f64));
        
        Ok(ValueType::Map(result))
    }
}

impl Clone for MacroEventListenerService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            storage: self.storage.clone(),
        }
    }
}
