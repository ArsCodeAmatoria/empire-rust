//! Core functionality for the Empire framework.
//!
//! This module provides the fundamental building blocks for the Empire framework,
//! including error handling, message protocols, command execution, agent management,
//! and task management.
//!
//! # Overview
//!
//! The core module is organized into several submodules:
//!
//! - `error`: Error types and handling
//! - `message`: Message protocol and serialization
//! - `command`: Command types and execution
//! - `agent`: Agent management and status tracking
//! - `task`: Task management and tracking
//!
//! # Examples
//!
//! ```no_run
//! use empire_rust::core::{
//!     EmpireError,
//!     Message,
//!     CommandType,
//!     AgentInfo,
//!     TaskInfo,
//! };
//!
//! // Create a new agent
//! let agent = AgentInfoBuilder::new("agent-1", "127.0.0.1:1337".parse().unwrap())
//!     .with_os_info("Linux")
//!     .with_hostname("localhost")
//!     .with_username("user")
//!     .build();
//!
//! // Create a new task
//! let task = TaskInfoBuilder::new(agent.id.clone(), CommandType::Shell {
//!     command: "whoami".to_string(),
//!     args: vec![],
//! }).build();
//! ```

pub mod error;
pub mod message;
pub mod command;
pub mod agent;
pub mod task;

// Re-export commonly used types
pub use error::EmpireError;
pub use message::{Message, MessageHandler, MessageId};
pub use command::{CommandType, CommandExecutor, CommandResult};
pub use agent::{AgentInfo, AgentManager, AgentStatus, AgentInfoBuilder};
pub use task::{TaskInfo, TaskManager, TaskStatus, TaskInfoBuilder}; 