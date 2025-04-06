//! Server module for the Empire-Rust framework.
//!
//! This module implements the server-side functionality of the Empire framework.
//! It provides a concrete implementation of the `EmpireServer` trait, managing
//! agents and their tasks.

use crate::core::{Agent, EmpireError, EmpireServer, Message, MessageHandler, Task, TaskStatus, CommandType};
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
/// It handles authentication, command execution, and file transfers.
pub struct Server {
    /// Address to bind to for incoming connections
    address: String,
    /// Map of connected agents, keyed by agent ID
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    /// Map of active tasks, keyed by task ID
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    /// Map of valid credentials for authentication
    credentials: HashMap<String, String>,
}

/// Represents a connected agent in the Empire framework.
///
/// An agent is a client that has successfully authenticated with the server
/// and can receive and execute commands.
#[derive(Debug, Clone)]
struct Agent {
    /// Unique identifier for the agent
    id: String,
    /// Network address of the agent
    address: SocketAddr,
    /// Timestamp of the last received heartbeat
    last_heartbeat: std::time::Instant,
    /// Current connection status of the agent
    status: AgentStatus,
}

/// Represents the connection status of an agent.
#[derive(Debug, Clone, PartialEq)]
enum AgentStatus {
    /// Agent is currently connected and active
    Connected,
    /// Agent has disconnected
    Disconnected,
}

/// Represents a task to be executed by an agent.
///
/// A task contains all information necessary to execute a command on an agent
/// and track its progress and results.
#[derive(Debug, Clone)]
struct Task {
    /// Unique identifier for the task
    id: String,
    /// ID of the agent this task is assigned to
    agent_id: String,
    /// The command to be executed
    command: CommandType,
    /// Current status of the task
    status: TaskStatus,
    /// Result of the command execution, if completed
    result: Option<String>,
    /// Error message, if the task failed
    error: Option<String>,
}

impl Server {
    /// Creates a new instance of the Empire server.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind to for incoming connections
    ///
    /// # Returns
    ///
    /// A new `Server` instance with empty agent and task lists.
    pub fn new(address: String) -> Self {
        Self {
            address,
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            credentials: HashMap::from([
                ("admin".to_string(), "secret".to_string()),
            ]),
        }
    }

    /// Handles a new connection from a client.
    ///
    /// This method performs the following steps:
    /// 1. Waits for an authentication request
    /// 2. Verifies the credentials
    /// 3. Creates a new agent if authentication is successful
    /// 4. Starts the agent communication loop
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream for the new connection
    /// * `addr` - The address of the connecting client
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the connection was handled successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<(), EmpireError> {
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        // Wait for authentication request
        match framed.receive_message().await? {
            Message::AuthRequest { username, password } => {
                if self.verify_credentials(&username, &password) {
                    // Create new agent
                    let agent_id = Uuid::new_v4().to_string();
                    let agent = Agent {
                        id: agent_id.clone(),
                        address: addr,
                        last_heartbeat: std::time::Instant::now(),
                        status: AgentStatus::Connected,
                    };

                    // Add agent to the list
                    self.agents.write().await.insert(agent_id.clone(), agent);

                    // Send successful authentication response
                    framed.send_message(Message::AuthResponse {
                        success: true,
                        message: "Authentication successful".to_string(),
                        agent_id: Some(agent_id),
                    }).await?;

                    // Handle agent communication
                    self.handle_agent_communication(framed, agent_id).await?;
                } else {
                    // Send failed authentication response
                    framed.send_message(Message::AuthResponse {
                        success: false,
                        message: "Invalid credentials".to_string(),
                        agent_id: None,
                    }).await?;
                }
            }
            _ => {
                // Send error for invalid message type
                framed.send_message(Message::AuthResponse {
                    success: false,
                    message: "Invalid message type".to_string(),
                    agent_id: None,
                }).await?;
            }
        }

        Ok(())
    }

    /// Verifies the provided credentials against the stored credentials.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to verify
    /// * `password` - The password to verify
    ///
    /// # Returns
    ///
    /// `true` if the credentials are valid, `false` otherwise
    fn verify_credentials(&self, username: &str, password: &str) -> bool {
        self.credentials.get(username)
            .map(|stored_password| stored_password == password)
            .unwrap_or(false)
    }

    /// Handles ongoing communication with an authenticated agent.
    ///
    /// This method processes messages from the agent, including:
    /// - Heartbeats
    /// - Command results
    /// - File transfer responses
    ///
    /// # Arguments
    ///
    /// * `framed` - The framed connection to the agent
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    ///
    /// * `Ok(())` if communication was handled successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn handle_agent_communication(
        &self,
        mut framed: Framed<TcpStream, LengthDelimitedCodec>,
        agent_id: String,
    ) -> Result<(), EmpireError> {
        loop {
            match framed.receive_message().await? {
                Message::Heartbeat { agent_id: _ } => {
                    // Update last heartbeat time
                    if let Some(agent) = self.agents.write().await.get_mut(&agent_id) {
                        agent.last_heartbeat = std::time::Instant::now();
                    }
                }
                Message::CommandResult { id, success, output, error } => {
                    // Update task status and result
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.status = if success {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed
                        };
                        task.result = Some(output);
                        task.error = error;
                    }
                }
                Message::FileTransferResponse { id, accepted, message } => {
                    // Handle file transfer response
                    info!("File transfer {}: {}", id, message);
                    if !accepted {
                        if let Some(task) = self.tasks.write().await.get_mut(&id) {
                            task.status = TaskStatus::Failed;
                            task.error = Some(message);
                        }
                    }
                }
                _ => {
                    warn!("Received unexpected message type");
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl EmpireServer for Server {
    /// Starts the Empire server and begins accepting connections.
    ///
    /// This method:
    /// 1. Binds to the specified address
    /// 2. Listens for incoming connections
    /// 3. Spawns a new task for each connection
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the server started successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn start(&self) -> Result<(), EmpireError> {
        let listener = TcpListener::bind(&self.address)
            .await
            .map_err(|e| EmpireError::ConnectionError(e.to_string()))?;

        info!("Server listening on {}", self.address);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, addr).await {
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
    /// # Returns
    ///
    /// * `Ok(())` if the server stopped successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn stop(&self) -> Result<(), EmpireError> {
        // TODO: Implement graceful shutdown
        Ok(())
    }

    /// Lists all currently connected agents.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing the IDs of all connected agents
    /// * `Err(EmpireError)` if an error occurred
    async fn list_agents(&self) -> Result<Vec<String>, EmpireError> {
        Ok(self.agents.read().await.keys().cloned().collect())
    }

    /// Executes a command on a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to execute the command on
    /// * `command` - The command to execute
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the task ID if the command was sent successfully
    /// * `Err(EmpireError)` if an error occurred
    async fn execute_command(&self, agent_id: &str, command: CommandType) -> Result<String, EmpireError> {
        // Create new task
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            agent_id: agent_id.to_string(),
            command: command.clone(),
            status: TaskStatus::Pending,
            result: None,
            error: None,
        };

        // Add task to the list
        self.tasks.write().await.insert(task_id.clone(), task);

        // Send command to agent
        if let Some(agent) = self.agents.read().await.get(agent_id) {
            // TODO: Implement command sending to agent
            Ok(format!("Command sent to agent {}", agent_id))
        } else {
            Err(EmpireError::ConnectionError(format!("Agent {} not found", agent_id)))
        }
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            agents: self.agents.clone(),
            tasks: self.tasks.clone(),
            credentials: self.credentials.clone(),
        }
    }
} 