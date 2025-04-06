//! Command handling for the Empire framework.
//!
//! This module defines the command types that can be executed by agents
//! and provides functionality for command execution and result handling.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::error::EmpireError;

/// Types of commands that can be executed by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    /// Execute a shell command
    Shell {
        /// The command to execute
        command: String,
        /// Arguments to pass to the command
        args: Vec<String>,
    },
    /// Upload a file to the agent
    Upload {
        /// Path to the source file
        source_path: String,
        /// Path to save the file on the agent
        dest_path: String,
    },
    /// Download a file from the agent
    Download {
        /// Path to the file on the agent
        source_path: String,
        /// Path to save the file locally
        dest_path: String,
    },
    /// List files in a directory
    ListDirectory {
        /// Path to the directory to list
        path: String,
    },
    /// Get system information
    SystemInfo,
    /// Get process information
    ProcessInfo {
        /// Optional process ID to get information about
        pid: Option<u32>,
    },
    /// Kill a process
    KillProcess {
        /// ID of the process to kill
        pid: u32,
    },
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::Shell { command, args } => {
                write!(f, "Shell: {} {}", command, args.join(" "))
            }
            CommandType::Upload { source_path, dest_path } => {
                write!(f, "Upload: {} -> {}", source_path, dest_path)
            }
            CommandType::Download { source_path, dest_path } => {
                write!(f, "Download: {} -> {}", source_path, dest_path)
            }
            CommandType::ListDirectory { path } => {
                write!(f, "List Directory: {}", path)
            }
            CommandType::SystemInfo => write!(f, "System Info"),
            CommandType::ProcessInfo { pid } => {
                if let Some(pid) = pid {
                    write!(f, "Process Info: {}", pid)
                } else {
                    write!(f, "Process Info: All")
                }
            }
            CommandType::KillProcess { pid } => {
                write!(f, "Kill Process: {}", pid)
            }
        }
    }
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether the command executed successfully
    pub success: bool,
    /// Output produced by the command
    pub output: String,
    /// Error message if the command failed
    pub error: Option<String>,
}

impl CommandResult {
    /// Create a successful command result
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    /// Create a failed command result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}

/// Trait for executing commands
#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command and return the result
    async fn execute(&self, command: CommandType) -> Result<CommandResult, EmpireError>;
}

/// Builder for creating commands
pub struct CommandBuilder {
    command_type: CommandType,
}

impl CommandBuilder {
    /// Create a new command builder
    pub fn new() -> Self {
        Self {
            command_type: CommandType::SystemInfo,
        }
    }

    /// Build a shell command
    pub fn shell(command: String, args: Vec<String>) -> Self {
        Self {
            command_type: CommandType::Shell { command, args },
        }
    }

    /// Build an upload command
    pub fn upload(source_path: String, dest_path: String) -> Self {
        Self {
            command_type: CommandType::Upload {
                source_path,
                dest_path,
            },
        }
    }

    /// Build a download command
    pub fn download(source_path: String, dest_path: String) -> Self {
        Self {
            command_type: CommandType::Download {
                source_path,
                dest_path,
            },
        }
    }

    /// Build a list directory command
    pub fn list_directory(path: String) -> Self {
        Self {
            command_type: CommandType::ListDirectory { path },
        }
    }

    /// Build a system info command
    pub fn system_info() -> Self {
        Self {
            command_type: CommandType::SystemInfo,
        }
    }

    /// Build a process info command
    pub fn process_info(pid: Option<u32>) -> Self {
        Self {
            command_type: CommandType::ProcessInfo { pid },
        }
    }

    /// Build a kill process command
    pub fn kill_process(pid: u32) -> Self {
        Self {
            command_type: CommandType::KillProcess { pid },
        }
    }

    /// Get the built command
    pub fn build(self) -> CommandType {
        self.command_type
    }
} 