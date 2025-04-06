//! Unit tests for the core module of Empire-Rust.
//!
//! This module contains unit tests for the core functionality of the Empire framework,
//! including message handling, error types, and basic data structures.

use empire_rust::core::{EmpireError, Message, MessageHandler};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// Test suite for the EmpireError type.
///
/// This module contains tests that verify the behavior of the EmpireError enum
/// and its associated functionality.
#[cfg(test)]
mod error_tests {
    use super::*;

    /// Test creation of NetworkError variant.
    ///
    /// Verifies that a NetworkError can be created with a message and that
    /// the message is stored correctly.
    #[test]
    fn test_network_error() {
        let error = EmpireError::NetworkError("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");
    }

    /// Test creation of AuthenticationError variant.
    ///
    /// Verifies that an AuthenticationError can be created and that its
    /// string representation is correct.
    #[test]
    fn test_authentication_error() {
        let error = EmpireError::AuthenticationError;
        assert_eq!(error.to_string(), "Authentication failed");
    }
}

/// Test suite for the Message type.
///
/// This module contains tests that verify the behavior of the Message enum
/// and its serialization/deserialization functionality.
#[cfg(test)]
mod message_tests {
    use super::*;

    /// Test serialization of Heartbeat message.
    ///
    /// Verifies that a Heartbeat message can be serialized and deserialized
    /// correctly, maintaining all its data.
    #[test]
    fn test_heartbeat_serialization() {
        let message = Message::Heartbeat;
        let serialized = bincode::serialize(&message).unwrap();
        let deserialized: Message = bincode::deserialize(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }

    /// Test serialization of CommandRequest message.
    ///
    /// Verifies that a CommandRequest message with command data can be
    /// serialized and deserialized correctly.
    #[test]
    fn test_command_request_serialization() {
        let message = Message::CommandRequest {
            command: "whoami".to_string(),
            args: vec!["-u".to_string()],
        };
        let serialized = bincode::serialize(&message).unwrap();
        let deserialized: Message = bincode::deserialize(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }
}

/// Test suite for the MessageHandler trait.
///
/// This module contains tests that verify the behavior of the MessageHandler
/// trait implementation for TcpStream.
#[cfg(test)]
mod message_handler_tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};

    /// Test sending and receiving messages over a TCP connection.
    ///
    /// This test verifies that messages can be sent and received correctly
    /// over a TCP connection using the MessageHandler trait implementation.
    #[tokio::test]
    async fn test_message_exchange() {
        // Create a TCP listener
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a task to accept a connection and echo messages
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut framed = Framed::new(socket, LengthDelimitedCodec::new());
            
            // Receive and echo the message
            if let Some(message) = framed.next().await {
                let message = message.unwrap();
                framed.send(message).await.unwrap();
            }
        });

        // Connect to the server
        let socket = TcpStream::connect(addr).await.unwrap();
        let mut framed = Framed::new(socket, LengthDelimitedCodec::new());

        // Send a test message
        let test_message = b"Hello, World!";
        framed.send(test_message.into()).await.unwrap();

        // Receive the echoed message
        let response = framed.next().await.unwrap().unwrap();
        assert_eq!(response, test_message);

        // Cleanup
        server.abort();
    }
} 