//! Core module for the Empire-Rust framework.
//!
//! This module contains the fundamental data structures and traits that define the
//! Empire-Rust framework's functionality. It provides the building blocks for both
//! server and client implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::BytesMut;

/// Error types that can occur during Empire operations.
#[derive(Error, Debug)]
pub enum EmpireError {
    /// Error occurred during connection establishment or maintenance
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Error occurred during authentication
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Error occurred during command execution
    #[error("Command execution error: {0}")]
    CommandError(String),

    /// Error occurred during message serialization/deserialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Types of messages that can be exchanged between server and client
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Authentication request from client
    AuthRequest {
        username: String,
        password: String,
    },
    /// Authentication response from server
    AuthResponse {
        success: bool,
        message: String,
        agent_id: Option<String>,
    },
    /// Command to be executed
    Command {
        id: String,
        command: String,
        agent_id: String,
    },
    /// Command execution result
    CommandResult {
        id: String,
        success: bool,
        output: String,
        error: Option<String>,
    },
    /// Heartbeat message
    Heartbeat {
        agent_id: String,
    },
    /// Error message
    Error {
        message: String,
    },
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

/// Represents the current status of a task.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    Pending,
    
    /// Task is currently being executed
    Running,
    
    /// Task has completed successfully
    Completed,
    
    /// Task failed during execution
    Failed,
}

/// Trait for handling communication between server and client
pub trait MessageHandler {
    /// Send a message to the connected peer
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError>;
    
    /// Receive a message from the connected peer
    async fn receive_message(&mut self) -> Result<Message, EmpireError>;
}

/// Implementation of MessageHandler for TcpStream
impl MessageHandler for Framed<TcpStream, LengthDelimitedCodec> {
    async fn send_message(&mut self, message: Message) -> Result<(), EmpireError> {
        let bytes = bincode::serialize(&message)
            .map_err(|e| EmpireError::SerializationError(e.to_string()))?;
        self.send(bytes.into()).await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, EmpireError> {
        if let Some(result) = self.next().await {
            let bytes = result.map_err(|e| EmpireError::ConnectionError(e.to_string()))?;
            let message = bincode::deserialize(&bytes)
                .map_err(|e| EmpireError::SerializationError(e.to_string()))?;
            Ok(message)
        } else {
            Err(EmpireError::ConnectionError("Connection closed".to_string()))
        }
    }
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