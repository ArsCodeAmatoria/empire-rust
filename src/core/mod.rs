//! Core module for the Empire-Rust framework.
//!
//! This module contains the core data structures and traits used throughout the framework.
//! It defines the communication protocol, error handling, and basic types used by both
//! the server and client implementations.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;

/// Error types for the Empire framework.
///
/// This enum defines all possible error types that can occur during the operation
/// of the Empire framework. Each variant includes a descriptive message.
#[derive(Debug, Serialize, Deserialize)]
pub enum EmpireError {
    /// Error occurred during network connection establishment or maintenance
    ConnectionError(String),
    /// Error occurred during authentication process
    AuthenticationError(String),
    /// Error occurred during command execution
    CommandError(String),
    /// Error occurred during message serialization or deserialization
    SerializationError(String),
    /// Error occurred during file operations
    FileError(String),
    /// Error occurred due to invalid input data
    ValidationError(String),
    /// Catch-all for unexpected errors
    Unknown(String),
}

impl fmt::Display for EmpireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmpireError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            EmpireError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            EmpireError::CommandError(msg) => write!(f, "Command error: {}", msg),
            EmpireError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            EmpireError::FileError(msg) => write!(f, "File error: {}", msg),
            EmpireError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            EmpireError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl Error for EmpireError {}

/// Represents the current status of a task in the Empire framework.
///
/// Tasks can be in various states during their lifecycle, from creation to completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task has been created but not yet started
    Pending,
    /// Task is currently being executed
    Running,
    /// Task has completed successfully
    Completed,
    /// Task failed during execution
    Failed,
    /// Task was cancelled before completion
    Cancelled,
}

/// Types of commands that can be executed by agents.
///
/// This enum defines all possible command types that can be sent to agents.
/// Each variant contains the necessary data for executing that specific type of command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    /// Execute a shell command on the agent's system
    Shell(String),
    /// Upload a file from the server to the agent
    Upload {
        /// Path to the source file on the server
        source_path: String,
        /// Destination path on the agent's system
        dest_path: String,
    },
    /// Download a file from the agent to the server
    Download {
        /// Path to the source file on the agent's system
        source_path: String,
        /// Destination path on the server
        dest_path: String,
    },
    /// List the contents of a directory on the agent's system
    ListDir(String),
    /// Get system information from the agent
    SystemInfo,
    /// Get a list of running processes on the agent's system
    ProcessList,
    /// Kill a process on the agent's system
    KillProcess(u32),
}

/// Message types for communication between server and client.
///
/// This enum defines all possible message types that can be exchanged between
/// the server and client. Each variant contains the necessary data for that
/// specific type of message.
#[derive(Debug, Serialize, Deserialize)]
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
        id: String,
        /// ID of the agent to execute the command on
        agent_id: String,
        /// The command to execute
        command: CommandType,
    },
    /// Result of a command execution
    CommandResult {
        /// ID of the command that was executed
        id: String,
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
        id: String,
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
        id: String,
        /// Whether the transfer was accepted
        accepted: bool,
        /// Response message explaining the result
        message: String,
    },
    /// A chunk of file data being transferred
    FileChunk {
        /// ID of the file transfer this chunk belongs to
        id: String,
        /// The actual file data
        data: Vec<u8>,
        /// Whether this is the last chunk of the file
        is_last: bool,
    },
}

/// Trait for handling message sending and receiving.
///
/// This trait defines the interface for sending and receiving messages
/// over a connection. It is implemented for various connection types
/// to provide a consistent interface for message handling.
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Send a message over the connection.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message was sent successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError>;

    /// Receive a message from the connection.
    ///
    /// # Returns
    ///
    /// * `Ok(Message)` if a message was received successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn receive_message(&mut self) -> Result<Message, EmpireError>;
}

/// Implementation of MessageHandler for Framed connections.
///
/// This implementation provides message handling for any type that implements
/// AsyncRead and AsyncWrite, using a length-delimited codec for framing.
#[async_trait::async_trait]
impl<T> MessageHandler for Framed<T, LengthDelimitedCodec>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError> {
        // Serialize the message to bytes
        let bytes = bincode::serialize(&message)
            .map_err(|e| EmpireError::SerializationError(e.to_string()))?;
        
        // Send the bytes over the connection
        self.send(bytes.into()).await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;
        
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, EmpireError> {
        // Wait for the next message
        let bytes = self.next().await
            .ok_or_else(|| EmpireError::ConnectionError("Connection closed".to_string()))?
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;
        
        // Deserialize the bytes into a Message
        bincode::deserialize(&bytes)
            .map_err(|e| EmpireError::SerializationError(e.to_string()))
    }
}

/// Represents an agent in the Empire framework.
///
/// An agent is a client that has connected to the server and can execute commands.
/// Each agent has a unique identifier and maintains information about its host system.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: String,
    
    /// Hostname of the system where the agent is running
    pub hostname: String,
    
    /// Username under which the agent is running
    pub username: String,
    
    /// Operating system of the host
    pub os: String,
    
    /// Last time the agent was seen by the server
    pub last_seen: chrono::DateTime<chrono::Utc>,
    
    /// List of tasks assigned to this agent
    pub tasks: Vec<Task>,
}

/// Represents a task to be executed by an agent.
///
/// A task contains the command to be executed, its current status,
/// and any output produced during execution.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,
    
    /// Command to be executed
    pub command: String,
    
    /// Current status of the task
    pub status: TaskStatus,
    
    /// Output produced by the command, if any
    pub output: Option<String>,
    
    /// Time when the task was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Trait defining the server-side functionality of the Empire framework.
///
/// Implementations of this trait should provide methods for managing agents
/// and executing commands on them.
#[async_trait::async_trait]
pub trait EmpireServer {
    /// Start the Empire server and begin accepting connections.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the server fails to start.
    async fn start(&self) -> Result<(), EmpireError>;
    
    /// Stop the Empire server and disconnect all clients.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the server fails to stop properly.
    async fn stop(&self) -> Result<(), EmpireError>;
    
    /// List all currently connected agents.
    ///
    /// # Returns
    ///
    /// A vector of `Agent` structs representing all connected agents.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the agent list cannot be retrieved.
    async fn list_agents(&self) -> Result<Vec<Agent>, EmpireError>;
    
    /// Execute a command on a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to execute the command on
    /// * `command` - The command to execute
    ///
    /// # Returns
    ///
    /// A task ID that can be used to track the command's execution.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the command fails to execute.
    async fn execute_command(&self, agent_id: &str, command: &str) -> Result<String, EmpireError>;
}

/// Trait defining the client-side functionality of the Empire framework.
///
/// Implementations of this trait should provide methods for connecting to
/// an Empire server and executing commands.
#[async_trait::async_trait]
pub trait EmpireClient {
    /// Connect to an Empire server.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the connection fails.
    async fn connect(&self) -> Result<(), EmpireError>;
    
    /// Disconnect from the Empire server.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the disconnection fails.
    async fn disconnect(&self) -> Result<(), EmpireError>;
    
    /// Execute a command on the connected server.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    ///
    /// # Returns
    ///
    /// The output of the executed command.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the command fails to execute.
    async fn execute_command(&self, command: &str) -> Result<String, EmpireError>;
} 