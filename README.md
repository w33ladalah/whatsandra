# WhatsApp API for Rust

A Rust port of the Go WhatsApp API library [whatsmeow](https://github.com/tulir/whatsmeow).

## Features

- Connect to WhatsApp Web
- Send and receive messages
- Support for media messages (images, videos, documents)
- QR code authentication
- Event-based API
- Session management
- End-to-end encryption

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
whatsandra = { git = "https://github.com/w33ladalah/whatsandra" }
```

## Usage Example

```rust
use whatsandra::{
    JID, Event,
    error::WhatsAppError,
    client::{Client, ClientConfig, LogLevel},
    message::Message,
};

fn main() -> Result<(), WhatsAppError> {
    // Initialize env_logger
    env_logger::init();

    // Create a configuration
    let config = ClientConfig {
        store_path: "whatsapp_store".to_string(),
        log_level: LogLevel::Debug,
    };

    // Create the client
    let client = Client::new(config);

    // Add event handler
    client.add_event_handler(move |event| {
        match event {
            Event::Connected => {
                println!("Connected to WhatsApp!");
            },
            Event::QRCodeGenerated(qr) => {
                println!("Scan this QR code with your WhatsApp app:");
                println!("{}", qr);
            },
            Event::MessageReceived(msg) => {
                if let Some(text) = &msg.text {
                    println!("Received message from {}: {}", msg.chat_jid, text);
                }
            },
            _ => {} // Ignore other events
        }
    });

    // Connect to WhatsApp
    client.connect()?;

    // Generate QR code if not authenticated
    if !client.is_authenticated() {
        client.generate_qr_code()?;
    }

    // Send a message (if authenticated)
    if client.is_authenticated() {
        let jid = JID::new("1234567890", "s.whatsapp.net", None);
        let message = Message::new_text(jid, "Hello from Rust!");
        client.send_message(&message)?;
    }

    // Wait for user input before exiting
    println!("Press ENTER to exit");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Disconnect
    client.disconnect()?;

    Ok(())
}
```

## Implementation Status

This is a work in progress and not all features of the original Go library are implemented yet:

- [x] Basic message sending and receiving
- [x] Connection and authentication
- [x] Event system
- [ ] End-to-end encryption (partial implementation)
- [ ] Media handling
- [ ] Group operations
- [ ] Proper protobuf message parsing

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

* [tulir/whatsmeow](https://github.com/tulir/whatsmeow) - The original Go implementation
* WhatsApp for the Web API
