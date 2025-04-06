//! Task management for the Empire framework.
//!
//! This module defines the task types and provides functionality for
//! managing tasks, including task creation, status tracking, and result
//! handling.

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::EmpireError;
use crate::core::command::CommandType;
use crate::core::message::MessageId;

/// Status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Information about a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Unique identifier for the task
    pub id: MessageId,
    /// ID of the agent the task is assigned to
    pub agent_id: String,
    /// The command to execute
    pub command: CommandType,
    /// Current status of the task
    pub status: TaskStatus,
    /// Time when the task was created
    pub created_at: Instant,
    /// Time when the task was started
    pub started_at: Option<Instant>,
    /// Time when the task was completed
    pub completed_at: Option<Instant>,
    /// Output produced by the task
    pub output: Option<String>,
    /// Error message if the task failed
    pub error: Option<String>,
}

impl TaskInfo {
    /// Create a new task with the given agent ID and command
    pub fn new(agent_id: String, command: CommandType) -> Self {
        Self {
            id: MessageId::new(),
            agent_id,
            command,
            status: TaskStatus::Pending,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            output: None,
            error: None,
        }
    }

    /// Start the task
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Instant::now());
    }

    /// Complete the task successfully
    pub fn complete(&mut self, output: String) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Instant::now());
        self.output = Some(output);
    }

    /// Fail the task
    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Instant::now());
        self.error = Some(error);
    }

    /// Cancel the task
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Instant::now());
    }

    /// Get the duration of the task
    pub fn duration(&self) -> Option<Duration> {
        if let Some(started) = self.started_at {
            if let Some(completed) = self.completed_at {
                Some(completed.duration_since(started))
            } else {
                Some(Instant::now().duration_since(started))
            }
        } else {
            None
        }
    }
}

/// Trait for managing tasks
pub trait TaskManager: Send + Sync {
    /// Create a new task
    fn create_task(&mut self, agent_id: String, command: CommandType) -> Result<MessageId, EmpireError>;

    /// Get a task by ID
    fn get_task(&self, task_id: MessageId) -> Option<&TaskInfo>;

    /// Get a mutable reference to a task by ID
    fn get_task_mut(&mut self, task_id: MessageId) -> Option<&mut TaskInfo>;

    /// List all tasks
    fn list_tasks(&self) -> Vec<&TaskInfo>;

    /// List tasks for a specific agent
    fn list_agent_tasks(&self, agent_id: &str) -> Vec<&TaskInfo>;

    /// Start a task
    fn start_task(&mut self, task_id: MessageId) -> Result<(), EmpireError>;

    /// Complete a task successfully
    fn complete_task(&mut self, task_id: MessageId, output: String) -> Result<(), EmpireError>;

    /// Fail a task
    fn fail_task(&mut self, task_id: MessageId, error: String) -> Result<(), EmpireError>;

    /// Cancel a task
    fn cancel_task(&mut self, task_id: MessageId) -> Result<(), EmpireError>;
}

/// Builder for creating tasks
pub struct TaskInfoBuilder {
    task_info: TaskInfo,
}

impl TaskInfoBuilder {
    /// Create a new task info builder
    pub fn new(agent_id: String, command: CommandType) -> Self {
        Self {
            task_info: TaskInfo::new(agent_id, command),
        }
    }

    /// Build the task information
    pub fn build(self) -> TaskInfo {
        self.task_info
    }
} 