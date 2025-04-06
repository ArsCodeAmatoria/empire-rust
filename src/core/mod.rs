use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmpireError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Command execution error: {0}")]
    CommandError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub trait EmpireServer {
    async fn start(&self) -> Result<(), EmpireError>;
    async fn stop(&self) -> Result<(), EmpireError>;
    async fn list_agents(&self) -> Result<Vec<Agent>, EmpireError>;
    async fn execute_command(&self, agent_id: &str, command: &str) -> Result<String, EmpireError>;
}

pub trait EmpireClient {
    async fn connect(&self) -> Result<(), EmpireError>;
    async fn disconnect(&self) -> Result<(), EmpireError>;
    async fn execute_command(&self, command: &str) -> Result<String, EmpireError>;
} 