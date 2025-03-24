use anyhow::Result;
use runar_macros::{action, service};
use runar_node::{RequestContext, ValueType};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Storage for events with validation status
#[derive(Clone)]
pub struct EventEmitterStorage {
    published_events: Arc<Mutex<Vec<String>>>,
}

impl EventEmitterStorage {
    pub fn new() -> Self {
        Self {
            published_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// A service for publishing events
/// This implementation uses the Runar macros
#[service(
    name = "event_emitter_service",
    path = "event_emitter",
    description = "Event publishing service for testing",
    version = "1.0"
)]
pub struct MacroEventEmitterService {
    name: String,
    storage: EventEmitterStorage,
}

impl MacroEventEmitterService {
    /// Create a new MacroEventEmitterService
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            storage: EventEmitterStorage::new(),
        }
    }

    // The service macro automatically implements start, stop, state, and name methods
    
    // Initialize the service
    pub async fn init(&mut self, _context: &RequestContext) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    /// Validate event data
    #[action(path = "validate")]
    async fn validate(&self, event_data: String, context: &RequestContext) -> Result<ValueType> {
        println!("[{}] Validating event data: {}", self.name, event_data);
        
        // Simple validation: check if event data contains "valid"
        let is_valid = event_data.contains("valid");
        
        // Create a HashMap directly
        let mut result = HashMap::new();
        result.insert("valid".to_string(), ValueType::Bool(is_valid));
        result.insert("data".to_string(), ValueType::String(event_data));
        
        Ok(ValueType::Map(result))
    }
    
    /// Process a valid event
    #[action(path = "process_valid")]
    async fn process_valid(&self, data: String, context: &RequestContext) -> Result<ValueType> {
        println!("[{}] Processing valid event: {}", self.name, data);
        
        // Create event structure with metadata
        let mut event_data = HashMap::new();
        event_data.insert("data".to_string(), ValueType::String(data.clone()));
        event_data.insert("timestamp".to_string(), ValueType::String(chrono::Utc::now().to_rfc3339()));
        event_data.insert("source".to_string(), ValueType::String(self.name.clone()));
        event_data.insert("type".to_string(), ValueType::String("valid".to_string()));
        
        // Store the event for later retrieval
        {
            let mut events = self.storage.published_events.lock().unwrap();
            events.push(format!("valid:{}", data));
        }
        
        // Publish the event
        context.publish("events/valid", ValueType::Map(event_data)).await?;
        
        // Create a HashMap directly for the response
        let mut result = HashMap::new();
        result.insert("success".to_string(), ValueType::Bool(true));
        result.insert("status".to_string(), ValueType::String("published".to_string()));
        
        Ok(ValueType::Map(result))
    }
    
    /// Process an invalid event
    #[action(path = "process_invalid")]
    async fn process_invalid(&self, data: String, context: &RequestContext) -> Result<ValueType> {
        println!("[{}] Processing invalid event: {}", self.name, data);
        
        // Create event structure with metadata
        let mut event_data = HashMap::new();
        event_data.insert("data".to_string(), ValueType::String(data.clone()));
        event_data.insert("timestamp".to_string(), ValueType::String(chrono::Utc::now().to_rfc3339()));
        event_data.insert("source".to_string(), ValueType::String(self.name.clone()));
        event_data.insert("type".to_string(), ValueType::String("invalid".to_string()));
        event_data.insert("error".to_string(), ValueType::String("Event validation failed".to_string()));
        
        // Store the event for later retrieval
        {
            let mut events = self.storage.published_events.lock().unwrap();
            events.push(format!("invalid:{}", data));
        }
        
        // Publish the event to the error channel
        context.publish("events/invalid", ValueType::Map(event_data)).await?;
        
        // Create a HashMap directly for the response
        let mut result = HashMap::new();
        result.insert("success".to_string(), ValueType::Bool(true));
        result.insert("status".to_string(), ValueType::String("rejected".to_string()));
        
        Ok(ValueType::Map(result))
    }
    
    /// Get published events
    #[action(path = "get_published_events")]
    async fn get_published_events(&self) -> Result<ValueType> {
        let events = self.storage.published_events.lock().unwrap();
        
        // Create a HashMap directly for the response
        let mut result = HashMap::new();
        result.insert("events".to_string(), ValueType::Array(
            events.iter().map(|e| ValueType::String(e.clone())).collect()
        ));
        result.insert("count".to_string(), ValueType::Number(events.len() as f64));
        
        Ok(ValueType::Map(result))
    }
}

impl Clone for MacroEventEmitterService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            storage: self.storage.clone(),
        }
    }
}
