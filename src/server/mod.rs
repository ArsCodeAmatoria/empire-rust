//! Server implementation for the Empire framework.
//!
//! This module contains the server-side implementation of the Empire framework.
//! It is responsible for managing agents, executing commands, and handling
//! communication with clients.
//!
//! # Overview
//!
//! The server module provides:
//!
//! - Agent connection management
//! - Command distribution
//! - File transfer handling
//! - Heartbeat monitoring
//! - Task tracking
//!
//! # Examples
//!
//! ```no_run
//! use empire_rust::server::{Server, ServerConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create server configuration
//!     let config = ServerConfig {
//!         bind_address: "0.0.0.0:1337".parse().unwrap(),
//!         username: "admin".to_string(),
//!         password: "password".to_string(),
//!         heartbeat_timeout: 30,
//!     };
//!
//!     // Create and start server
//!     let mut server = Server::new(config);
//!     server.start().await?;
//!
//!     // Server is now running and accepting connections
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;
use std::time::Duration;

use crate::core::error::EmpireError;
use crate::core::message::{Message, MessageHandler, MessageId};
use crate::core::command::{CommandType, CommandExecutor, CommandResult};
use crate::core::agent::{AgentInfo, AgentManager, AgentStatus, AgentInfoBuilder};
use crate::core::task::{TaskInfo, TaskManager, TaskStatus, TaskInfoBuilder};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to
    pub bind_address: SocketAddr,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Heartbeat timeout in seconds
    pub heartbeat_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:1337".parse().unwrap(),
            username: "admin".to_string(),
            password: "password".to_string(),
            heartbeat_timeout: 30,
        }
    }
}

/// Server implementation
pub struct Server {
    /// Server configuration
    config: ServerConfig,
    /// Listener for incoming connections
    listener: Option<TcpListener>,
    /// Map of connected agents
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// Map of active tasks
    tasks: Arc<RwLock<HashMap<MessageId, TaskInfo>>>,
}

impl Server {
    /// Create a new server instance
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::server::{Server, ServerConfig};
    ///
    /// let config = ServerConfig::default();
    /// let server = Server::new(config);
    /// ```
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listener: None,
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the server and begin accepting connections
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - The server fails to bind to the specified address
    /// - An error occurs while accepting connections
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::server::{Server, ServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut server = Server::new(ServerConfig::default());
    ///     server.start().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start(&mut self) -> Result<(), EmpireError> {
        let listener = TcpListener::bind(self.config.bind_address).await
            .map_err(|e| EmpireError::Network(e.into()))?;
        
        self.listener = Some(listener);
        
        // Start the heartbeat monitor
        let agents = self.agents.clone();
        let timeout = Duration::from_secs(self.config.heartbeat_timeout);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let mut agents = agents.write().await;
                let stale_agents: Vec<String> = agents.values()
                    .filter(|agent| agent.is_heartbeat_stale(timeout))
                    .map(|agent| agent.id.clone())
                    .collect();
                
                for agent_id in stale_agents {
                    if let Some(agent) = agents.get_mut(&agent_id) {
                        agent.mark_disconnected();
                    }
                }
            }
        });

        // Accept incoming connections
        while let Some(listener) = &self.listener {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let agents = self.agents.clone();
                    let tasks = self.tasks.clone();
                    let config = self.config.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, addr, agents, tasks, config).await {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle a new connection
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream for the connection
    /// * `addr` - The address of the connecting client
    /// * `agents` - The map of connected agents
    /// * `tasks` - The map of active tasks
    /// * `config` - The server configuration
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - Authentication fails
    /// - An error occurs while handling the connection
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: Arc<RwLock<HashMap<MessageId, TaskInfo>>>,
        config: ServerConfig,
    ) -> Result<(), EmpireError> {
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        
        // Wait for authentication
        let message = framed.receive_message().await?;
        let (username, password) = match message {
            Message::AuthRequest { username, password } => (username, password),
            _ => return Err(EmpireError::Auth("Expected authentication request".into())),
        };

        // Verify credentials
        if username != config.username || password != config.password {
            framed.send_message(Message::AuthResponse {
                success: false,
                message: "Invalid credentials".into(),
                agent_id: None,
            }).await?;
            return Err(EmpireError::Auth("Invalid credentials".into()));
        }

        // Generate agent ID
        let agent_id = Uuid::new_v4().to_string();
        
        // Create agent info
        let agent_info = AgentInfoBuilder::new(agent_id.clone(), addr)
            .build();

        // Add agent to list
        agents.write().await.insert(agent_id.clone(), agent_info);

        // Send successful auth response
        framed.send_message(Message::AuthResponse {
            success: true,
            message: "Authentication successful".into(),
            agent_id: Some(agent_id.clone()),
        }).await?;

        // Handle agent communication
        Self::handle_agent_communication(framed, agent_id, agents, tasks).await
    }

    /// Handle communication with an agent
    ///
    /// # Arguments
    ///
    /// * `framed` - The framed connection
    /// * `agent_id` - The ID of the agent
    /// * `agents` - The map of connected agents
    /// * `tasks` - The map of active tasks
    ///
    /// # Errors
    ///
    /// Returns an `EmpireError` if:
    /// - An error occurs while handling messages
    async fn handle_agent_communication(
        mut framed: Framed<TcpStream, LengthDelimitedCodec>,
        agent_id: String,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: Arc<RwLock<HashMap<MessageId, TaskInfo>>>,
    ) -> Result<(), EmpireError> {
        loop {
            match framed.receive_message().await? {
                Message::Heartbeat { agent_id } => {
                    if let Some(agent) = agents.write().await.get_mut(&agent_id) {
                        agent.update_heartbeat();
                    }
                }
                Message::CommandResult { id, success, output, error } => {
                    if let Some(task) = tasks.write().await.get_mut(&id) {
                        if success {
                            task.complete(output);
                        } else {
                            task.fail(error.unwrap_or_else(|| "Unknown error".into()));
                        }
                    }
                }
                Message::FileTransferResponse { id, accepted, message } => {
                    if let Some(task) = tasks.write().await.get_mut(&id) {
                        if accepted {
                            task.complete(message);
                        } else {
                            task.fail(message);
                        }
                    }
                }
                _ => return Err(EmpireError::Validation("Unexpected message type".into())),
            }
        }
    }

    /// Stop the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use empire_rust::server::{Server, ServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut server = Server::new(ServerConfig::default());
    ///     server.start().await?;
    ///     // ... do something ...
    ///     server.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn stop(&mut self) -> Result<(), EmpireError> {
        if let Some(listener) = self.listener.take() {
            drop(listener);
        }
        Ok(())
    }
}

impl AgentManager for Server {
    fn add_agent(&mut self, agent: AgentInfo) -> Result<(), EmpireError> {
        let mut agents = self.agents.blocking_write();
        agents.insert(agent.id.clone(), agent);
        Ok(())
    }

    fn remove_agent(&mut self, agent_id: &str) -> Result<(), EmpireError> {
        let mut agents = self.agents.blocking_write();
        agents.remove(agent_id);
        Ok(())
    }

    fn get_agent(&self, agent_id: &str) -> Option<&AgentInfo> {
        let agents = self.agents.blocking_read();
        agents.get(agent_id)
    }

    fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut AgentInfo> {
        let mut agents = self.agents.blocking_write();
        agents.get_mut(agent_id)
    }

    fn list_agents(&self) -> Vec<&AgentInfo> {
        let agents = self.agents.blocking_read();
        agents.values().collect()
    }

    fn update_heartbeat(&mut self, agent_id: &str) -> Result<(), EmpireError> {
        if let Some(agent) = self.get_agent_mut(agent_id) {
            agent.update_heartbeat();
        }
        Ok(())
    }

    fn check_stale_agents(&mut self, timeout: Duration) -> Vec<String> {
        let agents = self.agents.blocking_read();
        agents.values()
            .filter(|agent| agent.is_heartbeat_stale(timeout))
            .map(|agent| agent.id.clone())
            .collect()
    }

    fn update_system_info(
        &mut self,
        agent_id: &str,
        os_info: String,
        hostname: String,
        username: String,
    ) -> Result<(), EmpireError> {
        if let Some(agent) = self.get_agent_mut(agent_id) {
            agent.update_system_info(os_info, hostname, username);
        }
        Ok(())
    }
}

impl TaskManager for Server {
    fn create_task(&mut self, agent_id: String, command: CommandType) -> Result<MessageId, EmpireError> {
        let task = TaskInfoBuilder::new(agent_id, command).build();
        let id = task.id;
        let mut tasks = self.tasks.blocking_write();
        tasks.insert(id, task);
        Ok(id)
    }

    fn get_task(&self, task_id: MessageId) -> Option<&TaskInfo> {
        let tasks = self.tasks.blocking_read();
        tasks.get(&task_id)
    }

    fn get_task_mut(&mut self, task_id: MessageId) -> Option<&mut TaskInfo> {
        let mut tasks = self.tasks.blocking_write();
        tasks.get_mut(&task_id)
    }

    fn list_tasks(&self) -> Vec<&TaskInfo> {
        let tasks = self.tasks.blocking_read();
        tasks.values().collect()
    }

    fn list_agent_tasks(&self, agent_id: &str) -> Vec<&TaskInfo> {
        let tasks = self.tasks.blocking_read();
        tasks.values()
            .filter(|task| task.agent_id == agent_id)
            .collect()
    }

    fn start_task(&mut self, task_id: MessageId) -> Result<(), EmpireError> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.start();
        }
        Ok(())
    }

    fn complete_task(&mut self, task_id: MessageId, output: String) -> Result<(), EmpireError> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.complete(output);
        }
        Ok(())
    }

    fn fail_task(&mut self, task_id: MessageId, error: String) -> Result<(), EmpireError> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.fail(error);
        }
        Ok(())
    }

    fn cancel_task(&mut self, task_id: MessageId) -> Result<(), EmpireError> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.cancel();
        }
        Ok(())
    }
} 