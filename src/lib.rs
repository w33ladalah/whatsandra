use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// Constants
pub const WHATSAPP_WEB_URL: &str = "https://web.whatsapp.com";

// Export modules
pub mod error;
pub mod message;
pub mod client;
pub mod websocket;
pub mod crypto;

// Re-export types
pub use error::{WhatsAppError, WhatsAppResult};
pub use client::LogLevel;

/// Represents a WhatsApp JID (Jabber ID)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JID {
    pub user: String,
    pub server: String,
    pub device: Option<u32>,
}

impl JID {
    pub fn new(user: &str, server: &str, device: Option<u32>) -> Self {
        Self {
            user: user.to_string(),
            server: server.to_string(),
            device,
        }
    }

    pub fn is_user(&self) -> bool {
        self.server == "s.whatsapp.net"
    }

    pub fn is_group(&self) -> bool {
        self.server == "g.us"
    }

    pub fn to_string(&self) -> String {
        match self.device {
            Some(device) => format!("{}@{}.{}", self.user, self.server, device),
            None => format!("{}@{}", self.user, self.server),
        }
    }
}

impl std::fmt::Display for JID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Supported media types for WhatsApp
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Sticker,
}

/// Represents a message that can be sent via WhatsApp
#[derive(Debug, Clone)]
pub struct Message {
    pub text: Option<String>,
    pub media_url: Option<String>,
    pub media_type: Option<MediaType>,
    pub mime_type: Option<String>,
    pub caption: Option<String>,
}

/// Client configuration options
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub store_path: String,
    pub log_level: LogLevel,
}

/// WhatsApp events
#[derive(Debug, Clone)]
pub enum Event {
    /// Connection established
    Connected,

    /// Connection lost
    Disconnected,

    /// QR code generated for authentication
    QRCodeGenerated(String),

    /// Authentication successful
    LoggedIn(JID),

    /// Authentication lost
    LoggedOut,

    /// Message received
    MessageReceived(message::Message),

    /// Message status update
    MessageStatus(message::MessageReceipt),

    /// Group update
    GroupUpdate(JID, String),

    /// Presence update
    Presence(JID, bool),

    /// Error event
    Error(error::WhatsAppError),

    /// Custom event
    Custom(String, String),
}

/// Type for event handlers
pub type EventHandler = Box<dyn Fn(Event) + Send + Sync>;

/// Main client for WhatsApp Web API
#[allow(dead_code)]
pub struct Client {
    config: ClientConfig,
    event_handlers: Arc<Mutex<Vec<EventHandler>>>,
    connected: Arc<Mutex<bool>>,
}

impl Client {
    /// Create a new WhatsApp client
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            event_handlers: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to WhatsApp servers
    pub fn connect(&self) -> Result<(), WhatsAppError> {
        // Would implement actual connection logic here
        *self.connected.lock().unwrap() = true;

        // Notify handlers that we're connected
        self.dispatch_event(Event::Connected);

        Ok(())
    }

    /// Disconnect from WhatsApp servers
    pub fn disconnect(&self) -> Result<(), WhatsAppError> {
        // Would implement actual disconnection logic here
        *self.connected.lock().unwrap() = false;

        // Notify handlers that we're disconnected
        self.dispatch_event(Event::Disconnected);

        Ok(())
    }

    /// Add an event handler
    pub fn add_event_handler<F>(&self, handler: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }

    /// Dispatch an event to all registered handlers
    fn dispatch_event(&self, event: Event) {
        let handlers = self.event_handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler(event.clone());
        }
    }

    /// Send a text message
    pub fn send_text_message(&self, to: JID, text: &str) -> Result<(), WhatsAppError> {
        // Would implement actual message sending logic here
        println!("Sending message to {}: {}", to.to_string(), text);
        Ok(())
    }

    /// Send a media message
    pub fn send_media_message(
        &self,
        to: JID,
        media_url: &str,
        _: MediaType,
        mime_type: &str,
        caption: Option<&str>,
    ) -> Result<(), WhatsAppError> {
        // Would implement actual media message sending logic here
        println!(
            "Sending media to {}: {} ({}), caption: {:?}",
            to.to_string(),
            media_url,
            mime_type,
            caption
        );
        Ok(())
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
}
