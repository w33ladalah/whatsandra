use std::sync::{Arc, Mutex};
use websocket::client::ClientBuilder;
use websocket::OwnedMessage;
use std::thread;
use tokio::sync::mpsc::{self, Sender, Receiver};
use log::{debug, error, info};

use crate::{
    Event,
    error::{WhatsAppError, WhatsAppResult}
};

/// WebSocket message types
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
}

/// Converts between websocket crate's messages and our enum
impl From<OwnedMessage> for WebSocketMessage {
    fn from(msg: OwnedMessage) -> Self {
        match msg {
            OwnedMessage::Text(text) => WebSocketMessage::Text(text),
            OwnedMessage::Binary(data) => WebSocketMessage::Binary(data),
            OwnedMessage::Ping(_) => WebSocketMessage::Ping,
            OwnedMessage::Pong(_) => WebSocketMessage::Pong,
            OwnedMessage::Close(_) => WebSocketMessage::Close,
        }
    }
}

impl Into<OwnedMessage> for WebSocketMessage {
    fn into(self) -> OwnedMessage {
        match self {
            WebSocketMessage::Text(text) => OwnedMessage::Text(text),
            WebSocketMessage::Binary(data) => OwnedMessage::Binary(data),
            WebSocketMessage::Ping => OwnedMessage::Ping(vec![]),
            WebSocketMessage::Pong => OwnedMessage::Pong(vec![]),
            WebSocketMessage::Close => OwnedMessage::Close(None),
        }
    }
}

/// WebSocket connection handler
pub struct WebSocketHandler {
    url: String,
    tx: Arc<Mutex<Option<Sender<WebSocketMessage>>>>,
    event_callback: Arc<Mutex<Box<dyn Fn(Event) + Send + Sync>>>,
    connected: Arc<Mutex<bool>>,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new<F>(url: &str, event_callback: F) -> Self
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        Self {
            url: url.to_string(),
            tx: Arc::new(Mutex::new(None)),
            event_callback: Arc::new(Mutex::new(Box::new(event_callback))),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to the WhatsApp WebSocket server
    pub fn connect(&self) -> WhatsAppResult<()> {
        let url = self.url.clone();
        let tx_clone = self.tx.clone();
        let event_callback = self.event_callback.clone();
        let connected = self.connected.clone();

        // Create a channel for sending messages to the WebSocket
        let (sender, receiver) = mpsc::channel::<WebSocketMessage>(100);

        // Store the sender
        *tx_clone.lock().unwrap() = Some(sender);

        // Start the WebSocket handler in a separate thread
        thread::spawn(move || {
            if let Err(err) = Self::run_websocket(url, receiver, event_callback.clone(), connected.clone()) {
                error!("WebSocket error: {:?}", err);

                // Notify that we're disconnected
                let callback = event_callback.lock().unwrap();
                callback(Event::Disconnected);

                // Update connection status
                *connected.lock().unwrap() = false;
            }
        });

        Ok(())
    }

    /// Run the WebSocket connection
    fn run_websocket(
        url: String,
        mut receiver: Receiver<WebSocketMessage>,
        event_callback: Arc<Mutex<Box<dyn Fn(Event) + Send + Sync>>>,
        connected: Arc<Mutex<bool>>,
    ) -> WhatsAppResult<()> {
        // Build the WebSocket client
        let client = ClientBuilder::new(&url)
            .map_err(|e| WhatsAppError::ConnectionError(e.to_string()))?
            .connect_insecure()
            .map_err(|e| WhatsAppError::ConnectionError(e.to_string()))?;

        let (mut receiver_ws, mut sender_ws) = client.split()
            .map_err(|e| WhatsAppError::ConnectionError(e.to_string()))?;

        // Set connected status
        *connected.lock().unwrap() = true;

        // Notify that we're connected
        let callback = event_callback.lock().unwrap();
        callback(Event::Connected);
        drop(callback);

        // Create a channel for communicating with the WebSocket writer
        let (tx_ws, mut rx_ws) = mpsc::channel::<OwnedMessage>(100);

        // Handle incoming messages in a separate thread
        let event_callback_clone = event_callback.clone();
        let connected_clone = connected.clone();
        let tx_ws_clone = tx_ws.clone();

        thread::spawn(move || {
            loop {
                match receiver_ws.recv_message() {
                    Ok(message) => {
                        let ws_message: WebSocketMessage = message.into();
                        match ws_message {
                            WebSocketMessage::Text(text) => {
                                debug!("Received text message: {}", text);
                                // Parse and handle the message
                                // In a real implementation, we would parse JSON/protobuf messages
                                // and dispatch appropriate events
                            },
                            WebSocketMessage::Binary(data) => {
                                debug!("Received binary message: {} bytes", data.len());
                                // Parse and handle binary message
                                // In a real implementation, we would parse protobuf messages
                            },
                            WebSocketMessage::Ping => {
                                // Respond with pong via channel
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                if let Err(e) = runtime.block_on(async {
                                    tx_ws_clone.send(OwnedMessage::Pong(vec![])).await
                                }) {
                                    error!("Failed to queue pong: {:?}", e);
                                    break;
                                }
                            },
                            WebSocketMessage::Close => {
                                info!("WebSocket connection closed by server");
                                break;
                            },
                            _ => {}
                        }
                    },
                    Err(e) => {
                        error!("Error receiving message: {:?}", e);
                        break;
                    }
                }
            }

            // Update connection status when the loop breaks
            *connected_clone.lock().unwrap() = false;

            // Notify that we're disconnected
            let callback = event_callback_clone.lock().unwrap();
            callback(Event::Disconnected);
        });

        // Thread for handling WebSocket writer
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                while let Some(message) = rx_ws.recv().await {
                    if let Err(e) = sender_ws.send_message(&message) {
                        error!("Failed to send message: {:?}", e);
                        break;
                    }
                }
            });
        });

        // Process outgoing messages from our channel
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| WhatsAppError::ConnectionError(e.to_string()))?;

        runtime.block_on(async {
            while let Some(message) = receiver.recv().await {
                // Convert our message to websocket message
                let ws_message: OwnedMessage = message.into();

                // Send the message using the channel
                if let Err(e) = tx_ws.send(ws_message).await {
                    error!("Failed to queue message: {:?}", e);
                    break;
                }
            }
        });

        Ok(())
    }

    /// Send a message through the WebSocket
    pub fn send(&self, message: WebSocketMessage) -> WhatsAppResult<()> {
        let tx = self.tx.lock().unwrap();

        if let Some(sender) = &*tx {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| WhatsAppError::ConnectionError(e.to_string()))?;

            runtime.block_on(async {
                sender.send(message).await.map_err(|e| {
                    WhatsAppError::ConnectionError(format!("Failed to send message: {}", e))
                })
            })
        } else {
            Err(WhatsAppError::ConnectionError("Not connected".to_string()))
        }
    }

    /// Check if the WebSocket is connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    /// Disconnect from the WebSocket server
    pub fn disconnect(&self) -> WhatsAppResult<()> {
        // Send close message
        self.send(WebSocketMessage::Close)?;

        // Clear the sender
        let mut tx = self.tx.lock().unwrap();
        *tx = None;

        // Update connection status
        let mut connected = self.connected.lock().unwrap();
        *connected = false;

        Ok(())
    }
}
