use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{error, info};

use crate::{
    JID, Event,
    error::{WhatsAppError, WhatsAppResult},
    message::Message,
    websocket::{WebSocketHandler, WebSocketMessage},
    crypto::{Crypto, KeyPair},
};

/// Logging level
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Client configuration
pub struct ClientConfig {
    pub store_path: String,
    pub log_level: LogLevel,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            store_path: "whatsapp_store".to_string(),
            log_level: LogLevel::Info,
        }
    }
}

/// Device store for saving and loading client state
pub struct DeviceStore {
    path: String,
    data: Mutex<HashMap<String, String>>,
}

impl DeviceStore {
    /// Create a new device store
    pub fn new(path: &str) -> Self {
        let data = if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(data) => data,
                        Err(_) => HashMap::new(),
                    }
                },
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self {
            path: path.to_string(),
            data: Mutex::new(data),
        }
    }

    /// Get a value from the store
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    /// Set a value in the store
    pub fn set(&self, key: &str, value: &str) -> WhatsAppResult<()> {
        {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_string(), value.to_string());
        }

        self.save()
    }

    /// Remove a value from the store
    pub fn remove(&self, key: &str) -> WhatsAppResult<()> {
        {
            let mut data = self.data.lock().unwrap();
            data.remove(key);
        }

        self.save()
    }

    /// Save the store to disk
    pub fn save(&self) -> WhatsAppResult<()> {
        let data = self.data.lock().unwrap();
        let content = serde_json::to_string(&*data)
            .map_err(|e| WhatsAppError::IOError(e.to_string()))?;

        fs::write(&self.path, content)
            .map_err(|e| WhatsAppError::IOError(e.to_string()))?;

        Ok(())
    }
}

/// WhatsApp client
#[allow(dead_code)]
pub struct Client {
    config: ClientConfig,
    store: Arc<DeviceStore>,
    websocket: Arc<WebSocketHandler>,
    event_handlers: Mutex<Vec<Box<dyn Fn(Event) + Send + Sync>>>,
    device_id: String,
    auth_state: Mutex<Option<AuthState>>,
}

/// Authentication state
#[allow(dead_code)]
struct AuthState {
    pub jid: JID,
    pub key_pair: KeyPair,
    pub session_id: String,
    pub secret: Vec<u8>,
}

impl Client {
    /// Create a new WhatsApp client
    pub fn new(config: ClientConfig) -> Arc<Self> {
        // Create the store directory if it doesn't exist
        if !Path::new(&config.store_path).exists() {
            if let Err(e) = fs::create_dir_all(&config.store_path) {
                error!("Failed to create store directory: {}", e);
            }
        }

        // Create store path
        let store_path = format!("{}/store.json", config.store_path);
        let store = Arc::new(DeviceStore::new(&store_path));

        // Generate device ID or use existing one
        let device_id = match store.get("device_id") {
            Some(id) => id,
            None => {
                let id = format!("rust_{}", hex::encode(Crypto::random_bytes(4)));
                if let Err(e) = store.set("device_id", &id) {
                    error!("Failed to store device ID: {}", e);
                }
                id
            }
        };

        // Create client
        let client = Arc::new(Self {
            config,
            store,
            event_handlers: Mutex::new(Vec::new()),
            device_id,
            auth_state: Mutex::new(None),
            websocket: Arc::new(WebSocketHandler::new(
                "wss://web.whatsapp.com/ws",
                |event| {
                    info!("WebSocket event: {:?}", event);
                    // In a real implementation, we would dispatch to the client's handlers
                },
            )),
        });

        client
    }

    /// Add an event handler
    pub fn add_event_handler<F>(&self, handler: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }

    /// Connect to WhatsApp
    pub fn connect(&self) -> WhatsAppResult<()> {
        self.websocket.connect()
    }

    /// Generate QR code for pairing
    pub fn generate_qr_code(&self) -> WhatsAppResult<String> {
        // Generate key pair
        let key_pair = Crypto::generate_key_pair()?;

        // Generate random session ID
        let session_id = hex::encode(Crypto::random_bytes(8));

        // Store credentials in auth_state
        let auth_state = AuthState {
            jid: JID::new("placeholder", "s.whatsapp.net", None),
            key_pair,
            session_id: session_id.clone(),
            secret: Crypto::random_bytes(32),
        };

        // Update auth state
        *self.auth_state.lock().unwrap() = Some(auth_state);

        // In a real implementation, we would generate an actual QR code
        // For now, return a placeholder
        Ok(format!("whatsapp://1234567890?key={}", session_id))
    }

    /// Send a message
    pub fn send_message(&self, message: &Message) -> WhatsAppResult<String> {
        if !self.is_connected() {
            return Err(WhatsAppError::ConnectionError("Not connected".to_string()));
        }

        // Check if authenticated
        if self.auth_state.lock().unwrap().is_none() {
            return Err(WhatsAppError::AuthError("Not authenticated".to_string()));
        }

        // Convert message to JSON
        let json = message.to_json()?;

        // Send message through WebSocket
        self.websocket.send(WebSocketMessage::Text(json))?;

        // Return message ID
        Ok(message.id.clone())
    }

    /// Check if connected to WhatsApp
    pub fn is_connected(&self) -> bool {
        self.websocket.is_connected()
    }

    /// Check if authenticated to WhatsApp
    pub fn is_authenticated(&self) -> bool {
        self.auth_state.lock().unwrap().is_some()
    }

    /// Logout from WhatsApp
    pub fn logout(&self) -> WhatsAppResult<()> {
        // Clear auth state
        *self.auth_state.lock().unwrap() = None;

        // Clear store
        self.store.remove("credentials")?;

        // Disconnect
        self.websocket.disconnect()?;

        Ok(())
    }

    /// Get device ID
    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    /// Create a message to a group
    pub fn create_group_message(&self, group_id: &str, text: &str) -> WhatsAppResult<Message> {
        let jid = JID::new(group_id, "g.us", None);
        Ok(Message::new_text(jid, text))
    }

    /// Create a message to a user
    pub fn create_user_message(&self, user_id: &str, text: &str) -> WhatsAppResult<Message> {
        let jid = JID::new(user_id, "s.whatsapp.net", None);
        Ok(Message::new_text(jid, text))
    }

    /// Disconnect from WhatsApp
    pub fn disconnect(&self) -> WhatsAppResult<()> {
        self.websocket.disconnect()
    }
}
