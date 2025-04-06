//! Message handling for the Empire framework.
//!
//! This module defines the message types and traits used for communication
//! between the server and clients. It includes serialization and deserialization
//! functionality for all message types.

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;

use crate::core::error::{EmpireError, SerializationError};
use crate::core::command::CommandType;

/// Unique identifier for messages and tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Create a new unique message ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message types for communication between server and client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Request to authenticate with the server
    AuthRequest {
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
    },
    /// Response to an authentication request
    AuthResponse {
        /// Whether authentication was successful
        success: bool,
        /// Response message explaining the result
        message: String,
        /// Agent ID assigned if authentication was successful
        agent_id: Option<String>,
    },
    /// Periodic message to indicate the agent is still alive
    Heartbeat {
        /// ID of the agent sending the heartbeat
        agent_id: String,
    },
    /// Request to execute a command on an agent
    CommandRequest {
        /// Unique ID for this command request
        id: MessageId,
        /// ID of the agent to execute the command on
        agent_id: String,
        /// The command to execute
        command: CommandType,
    },
    /// Result of a command execution
    CommandResult {
        /// ID of the command that was executed
        id: MessageId,
        /// Whether the command executed successfully
        success: bool,
        /// Output produced by the command
        output: String,
        /// Error message if the command failed
        error: Option<String>,
    },
    /// Request to transfer a file
    FileTransferRequest {
        /// Unique ID for this file transfer
        id: MessageId,
        /// ID of the agent involved in the transfer
        agent_id: String,
        /// Path to the source file
        source_path: String,
        /// Path to the destination file
        dest_path: String,
        /// Size of the file in bytes
        size: u64,
    },
    /// Response to a file transfer request
    FileTransferResponse {
        /// ID of the file transfer being responded to
        id: MessageId,
        /// Whether the transfer was accepted
        accepted: bool,
        /// Response message explaining the result
        message: String,
    },
    /// A chunk of file data being transferred
    FileChunk {
        /// ID of the file transfer this chunk belongs to
        id: MessageId,
        /// The actual file data
        data: Vec<u8>,
        /// Whether this is the last chunk of the file
        is_last: bool,
    },
}

/// Trait for handling message sending and receiving
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Send a message over the connection
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError>;

    /// Receive a message from the connection
    async fn receive_message(&mut self) -> Result<Message, EmpireError>;
}

/// Implementation of MessageHandler for Framed connections
#[async_trait::async_trait]
impl<T> MessageHandler for Framed<T, LengthDelimitedCodec>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError> {
        let bytes = bincode::serialize(&message)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))?;
        
        self.send(bytes.into()).await
            .map_err(|e| EmpireError::Network(e.into()))?;
        
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, EmpireError> {
        let bytes = self.next().await
            .ok_or_else(|| EmpireError::Network("Connection closed".into()))?
            .map_err(|e| EmpireError::Network(e.into()))?;
        
        bincode::deserialize(&bytes)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()).into())
    }
}

/// Builder for creating messages
pub struct MessageBuilder {
    message: Message,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            message: Message::Heartbeat {
                agent_id: String::new(),
            },
        }
    }

    /// Build an authentication request message
    pub fn auth_request(username: String, password: String) -> Self {
        Self {
            message: Message::AuthRequest {
                username,
                password,
            },
        }
    }

    /// Build a command request message
    pub fn command_request(agent_id: String, command: CommandType) -> Self {
        Self {
            message: Message::CommandRequest {
                id: MessageId::new(),
                agent_id,
                command,
            },
        }
    }

    /// Build a file transfer request message
    pub fn file_transfer_request(
        agent_id: String,
        source_path: String,
        dest_path: String,
        size: u64,
    ) -> Self {
        Self {
            message: Message::FileTransferRequest {
                id: MessageId::new(),
                agent_id,
                source_path,
                dest_path,
                size,
            },
        }
    }

    /// Get the built message
    pub fn build(self) -> Message {
        self.message
    }
} 