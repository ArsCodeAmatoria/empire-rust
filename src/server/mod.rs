//! Server module for the Empire-Rust framework.
//!
//! This module implements the server-side functionality of the Empire framework.
//! It provides a concrete implementation of the `EmpireServer` trait, managing
//! agents and their tasks.

use crate::core::{Agent, EmpireError, EmpireServer, Message, MessageHandler, Task, TaskStatus};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;
use futures::StreamExt;
use log::{error, info, warn};

/// Implementation of the Empire server.
///
/// The server maintains a list of connected agents and their associated tasks.
/// It provides methods for managing agents and executing commands on them.
///
/// # Examples
///
/// ```no_run
/// use empire_rust::server::Server;
///
/// #[tokio::main]
/// async fn main() {
///     let server = Server::new();
///     server.start().await.expect("Failed to start server");
/// }
/// ```
pub struct Server {
    /// Map of agent IDs to Agent structs
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    
    /// Map of task IDs to Task structs
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    
    /// Server address to bind to
    addr: SocketAddr,
    
    /// Authentication credentials
    credentials: HashMap<String, String>,
}

impl Server {
    /// Creates a new instance of the Empire server.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to bind the server to
    ///
    /// # Returns
    ///
    /// A new `Server` instance with empty agent and task lists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::server::Server;
    ///
    /// let server = Server::new();
    /// ```
    pub fn new(addr: SocketAddr) -> Self {
        let mut credentials = HashMap::new();
        // TODO: Load credentials from configuration
        credentials.insert("admin".to_string(), "password".to_string());
        
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            addr,
            credentials,
        }
    }

    /// Handles a new client connection.
    async fn handle_connection(&self, stream: TcpStream) -> Result<(), EmpireError> {
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        
        // Wait for authentication
        let message = framed.receive_message().await?;
        let (username, password) = match message {
            Message::AuthRequest { username, password } => (username, password),
            _ => return Err(EmpireError::AuthenticationError("Invalid message type".to_string())),
        };

        // Verify credentials
        if !self.verify_credentials(&username, &password) {
            framed.send_message(Message::AuthResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                agent_id: None,
            }).await?;
            return Err(EmpireError::AuthenticationError("Invalid credentials".to_string()));
        }

        // Create new agent
        let agent_id = Uuid::new_v4().to_string();
        let agent = Agent {
            id: agent_id.clone(),
            hostname: "unknown".to_string(), // TODO: Get from client
            username: username.clone(),
            os: "unknown".to_string(), // TODO: Get from client
            last_seen: chrono::Utc::now(),
            tasks: Vec::new(),
        };

        // Add agent to list
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent);
        }

        // Send successful authentication response
        framed.send_message(Message::AuthResponse {
            success: true,
            message: "Authentication successful".to_string(),
            agent_id: Some(agent_id.clone()),
        }).await?;

        // Handle agent communication
        self.handle_agent_communication(framed, agent_id).await
    }

    /// Verifies client credentials.
    fn verify_credentials(&self, username: &str, password: &str) -> bool {
        self.credentials.get(username)
            .map(|stored_password| stored_password == password)
            .unwrap_or(false)
    }

    /// Handles ongoing communication with an authenticated agent.
    async fn handle_agent_communication(
        &self,
        mut framed: Framed<TcpStream, LengthDelimitedCodec>,
        agent_id: String,
    ) -> Result<(), EmpireError> {
        loop {
            match framed.receive_message().await {
                Ok(Message::Heartbeat { agent_id: _ }) => {
                    // Update last seen time
                    if let Some(agent) = self.agents.write().await.get_mut(&agent_id) {
                        agent.last_seen = chrono::Utc::now();
                    }
                }
                Ok(Message::CommandResult { id, success, output, error }) => {
                    // Update task status
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.status = if success { TaskStatus::Completed } else { TaskStatus::Failed };
                        task.output = Some(output);
                    }
                }
                Ok(_) => warn!("Received unexpected message type"),
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }

        // Remove agent on disconnect
        self.agents.write().await.remove(&agent_id);
        Ok(())
    }
}

#[async_trait::async_trait]
impl EmpireServer for Server {
    /// Starts the Empire server and begins accepting connections.
    ///
    /// This method initializes the server's networking components and
    /// sets up the necessary infrastructure for agent communication.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The server fails to bind to the specified address
    /// - The server fails to start listening for connections
    /// - Any other initialization error occurs
    async fn start(&self) -> Result<(), EmpireError> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;

        info!("Server listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Stops the Empire server and disconnects all clients.
    ///
    /// This method gracefully shuts down the server, ensuring all
    /// active connections are properly closed and resources are freed.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The server fails to stop gracefully
    /// - Any active connections cannot be closed properly
    async fn stop(&self) -> Result<(), EmpireError> {
        // TODO: Implement graceful shutdown
        Ok(())
    }

    /// Lists all currently connected agents.
    ///
    /// This method returns a snapshot of all agents currently connected
    /// to the server, including their status and assigned tasks.
    ///
    /// # Returns
    ///
    /// A vector of `Agent` structs representing all connected agents.
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if the agent list cannot be retrieved.
    async fn list_agents(&self) -> Result<Vec<Agent>, EmpireError> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    /// Executes a command on a specific agent.
    ///
    /// This method creates a new task for the specified agent and
    /// returns a task ID that can be used to track the command's execution.
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
    /// Returns an `EmpireError` if:
    /// - The specified agent does not exist
    /// - The command cannot be executed
    /// - Any other error occurs during command execution
    async fn execute_command(&self, agent_id: &str, command: &str) -> Result<String, EmpireError> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            command: command.to_string(),
            status: TaskStatus::Pending,
            output: None,
            created_at: chrono::Utc::now(),
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        // TODO: Send command to agent
        Ok(task_id)
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            tasks: self.tasks.clone(),
            addr: self.addr,
            credentials: self.credentials.clone(),
        }
    }
} 