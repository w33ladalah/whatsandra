use thiserror::Error;

/// WhatsApp error type
#[derive(Debug, Error, Clone)]
pub enum WhatsAppError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Message sending error: {0}")]
    MessageSendError(String),

    #[error("Message receiving error: {0}")]
    MessageReceiveError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Group operation error: {0}")]
    GroupError(String),

    #[error("Media operation error: {0}")]
    MediaError(String),

    #[error("Store error: {0}")]
    StoreError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// Result type for WhatsApp operations
pub type WhatsAppResult<T> = Result<T, WhatsAppError>;
