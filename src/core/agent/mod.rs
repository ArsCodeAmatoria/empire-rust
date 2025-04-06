//! Agent management for the Empire framework.
//!
//! This module defines the agent types and provides functionality for
//! managing agents, including connection status tracking and heartbeat
//! monitoring.

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::core::error::EmpireError;

/// Status of an agent's connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is currently connected
    Connected,
    /// Agent is disconnected
    Disconnected,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Information about an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique identifier for the agent
    pub id: String,
    /// Network address of the agent
    pub address: SocketAddr,
    /// Current status of the agent
    pub status: AgentStatus,
    /// Time of the last heartbeat
    pub last_heartbeat: Option<Instant>,
    /// Operating system information
    pub os_info: Option<String>,
    /// Hostname of the agent
    pub hostname: Option<String>,
    /// Username running the agent
    pub username: Option<String>,
}

impl AgentInfo {
    /// Create a new agent with the given ID and address
    pub fn new(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            address,
            status: AgentStatus::Connected,
            last_heartbeat: Some(Instant::now()),
            os_info: None,
            hostname: None,
            username: None,
        }
    }

    /// Update the agent's heartbeat time
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        self.status = AgentStatus::Connected;
    }

    /// Check if the agent's heartbeat is stale
    pub fn is_heartbeat_stale(&self, timeout: Duration) -> bool {
        if let Some(last_heartbeat) = self.last_heartbeat {
            Instant::now().duration_since(last_heartbeat) > timeout
        } else {
            true
        }
    }

    /// Update the agent's system information
    pub fn update_system_info(&mut self, os_info: String, hostname: String, username: String) {
        self.os_info = Some(os_info);
        self.hostname = Some(hostname);
        self.username = Some(username);
    }

    /// Mark the agent as disconnected
    pub fn mark_disconnected(&mut self) {
        self.status = AgentStatus::Disconnected;
        self.last_heartbeat = None;
    }
}

/// Trait for managing agents
pub trait AgentManager: Send + Sync {
    /// Add a new agent
    fn add_agent(&mut self, agent: AgentInfo) -> Result<(), EmpireError>;

    /// Remove an agent by ID
    fn remove_agent(&mut self, agent_id: &str) -> Result<(), EmpireError>;

    /// Get an agent by ID
    fn get_agent(&self, agent_id: &str) -> Option<&AgentInfo>;

    /// Get a mutable reference to an agent by ID
    fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut AgentInfo>;

    /// List all agents
    fn list_agents(&self) -> Vec<&AgentInfo>;

    /// Update an agent's heartbeat
    fn update_heartbeat(&mut self, agent_id: &str) -> Result<(), EmpireError>;

    /// Check for stale agents
    fn check_stale_agents(&mut self, timeout: Duration) -> Vec<String>;

    /// Update an agent's system information
    fn update_system_info(
        &mut self,
        agent_id: &str,
        os_info: String,
        hostname: String,
        username: String,
    ) -> Result<(), EmpireError>;
}

/// Builder for creating agent information
pub struct AgentInfoBuilder {
    agent_info: AgentInfo,
}

impl AgentInfoBuilder {
    /// Create a new agent info builder
    pub fn new(id: String, address: SocketAddr) -> Self {
        Self {
            agent_info: AgentInfo::new(id, address),
        }
    }

    /// Set the agent's operating system information
    pub fn os_info(mut self, os_info: String) -> Self {
        self.agent_info.os_info = Some(os_info);
        self
    }

    /// Set the agent's hostname
    pub fn hostname(mut self, hostname: String) -> Self {
        self.agent_info.hostname = Some(hostname);
        self
    }

    /// Set the agent's username
    pub fn username(mut self, username: String) -> Self {
        self.agent_info.username = Some(username);
        self
    }

    /// Build the agent information
    pub fn build(self) -> AgentInfo {
        self.agent_info
    }
} 