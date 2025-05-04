use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::JID;

/// Message types supported by WhatsApp
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    Document,
    Contact,
    Location,
    Sticker,
    GroupInvite,
}

/// Information about a media attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub mime_type: String,
    pub sha256: Vec<u8>,
    pub file_length: u64,
    pub file_name: Option<String>,
    pub caption: Option<String>,
    pub url: Option<String>,
}

/// A WhatsApp message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_me: bool,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub chat_jid: JID,
    pub sender_jid: Option<JID>,
    pub text: Option<String>,
    pub media: Option<MediaInfo>,
    pub quoted: Option<Box<Message>>,
    pub mentioned_jids: Vec<JID>,
    pub is_ephemeral: bool,
    pub ephemeral_expiration: Option<u32>,
    pub context_info: HashMap<String, String>,
}

impl Message {
    /// Create a new text message
    pub fn new_text(chat_jid: JID, text: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: Self::generate_message_id(),
            from_me: true,
            timestamp: now,
            message_type: MessageType::Text,
            chat_jid,
            sender_jid: None,
            text: Some(text.to_string()),
            media: None,
            quoted: None,
            mentioned_jids: Vec::new(),
            is_ephemeral: false,
            ephemeral_expiration: None,
            context_info: HashMap::new(),
        }
    }

    /// Create a new image message
    pub fn new_image(chat_jid: JID, mime_type: &str, data: &[u8], caption: Option<&str>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // In a real implementation, we would calculate SHA256 of data
        let sha256 = crate::crypto::Crypto::sha256(data);

        Self {
            id: Self::generate_message_id(),
            from_me: true,
            timestamp: now,
            message_type: MessageType::Image,
            chat_jid,
            sender_jid: None,
            text: None,
            media: Some(MediaInfo {
                mime_type: mime_type.to_string(),
                sha256,
                file_length: data.len() as u64,
                file_name: None,
                caption: caption.map(|s| s.to_string()),
                url: None,
            }),
            quoted: None,
            mentioned_jids: Vec::new(),
            is_ephemeral: false,
            ephemeral_expiration: None,
            context_info: HashMap::new(),
        }
    }

    /// Quote another message
    pub fn quote(mut self, message: &Message) -> Self {
        self.quoted = Some(Box::new(message.clone()));
        self
    }

    /// Set message as ephemeral/disappearing
    pub fn make_ephemeral(mut self, expiration_seconds: u32) -> Self {
        self.is_ephemeral = true;
        self.ephemeral_expiration = Some(expiration_seconds);
        self
    }

    /// Mention users in the message
    pub fn mention(mut self, jids: Vec<JID>) -> Self {
        self.mentioned_jids = jids;
        self
    }

    /// Generate a random message ID
    fn generate_message_id() -> String {
        let random_bytes = crate::crypto::Crypto::random_bytes(8);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        format!("{}_{}", timestamp, hex::encode(random_bytes))
    }

    /// Convert the message to JSON format for sending
    pub fn to_json(&self) -> Result<String, crate::error::WhatsAppError> {
        serde_json::to_string(self)
            .map_err(|e| crate::error::WhatsAppError::SerializationError(e.to_string()))
    }
}

/// Message receipt status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptStatus {
    Sent,
    Delivered,
    Read,
    Played,  // For audio/video
    Failed,
}

/// Message receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub message_id: String,
    pub status: ReceiptStatus,
    pub timestamp: u64,
    pub recipient: JID,
}

/// Parser for incoming WhatsApp protocol messages
pub struct MessageParser;

impl MessageParser {
    /// Parse a binary message from WhatsApp
    pub fn parse_binary(_: &[u8]) -> Result<Message, crate::error::WhatsAppError> {
        // In a real implementation, this would use proper protobuf parsing
        // For this port example, we'll return an error
        Err(crate::error::WhatsAppError::ParsingError("Binary message parsing not implemented".to_string()))
    }

    /// Parse a JSON message from WhatsApp
    pub fn parse_json(data: &str) -> Result<Message, crate::error::WhatsAppError> {
        serde_json::from_str(data)
            .map_err(|e| crate::error::WhatsAppError::ParsingError(e.to_string()))
    }
}
