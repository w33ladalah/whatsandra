use std::io;

use whatsandra::{
    Event, WhatsAppError,
    client::{Client, ClientConfig, LogLevel},
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
    let client_clone = client.clone();

    // Add event handler
    client.add_event_handler(move |event| {
        match event {
            Event::Connected => {
                println!("✅ Connected to WhatsApp!");
            },
            Event::Disconnected => {
                println!("❌ Disconnected from WhatsApp");
            },
            Event::QRCodeGenerated(qr) => {
                println!("🔐 Scan this QR code with your WhatsApp app:");
                println!("{}", qr);

                // In a real application, we would display the QR code as an image
            },
            Event::LoggedIn(jid) => {
                println!("🎉 Logged in as {}", jid);
            },
            Event::LoggedOut => {
                println!("👋 Logged out");
            },
            Event::MessageReceived(msg) => {
                if let Some(text) = &msg.text {
                    println!("📩 Received message from {}: {}", msg.chat_jid, text);

                    // Echo the message back
                    if let Ok(reply) = client_clone.create_user_message(&msg.chat_jid.user, &format!("Echo: {}", text)) {
                        if let Err(e) = client_clone.send_message(&reply) {
                            println!("Failed to send reply: {:?}", e);
                        }
                    }
                }
            },
            _ => {
                // Ignore other events
            }
        }
    });

    // Connect to WhatsApp
    println!("Connecting to WhatsApp...");
    client.connect()?;

    // Generate QR code if not authenticated
    if !client.is_authenticated() {
        println!("Not authenticated, generating QR code...");
        client.generate_qr_code()?;
    }

    // Main loop
    println!("Press ENTER to exit");
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| WhatsAppError::IOError(e.to_string()))?;

    // Disconnect
    client.disconnect()?;

    Ok(())
}
