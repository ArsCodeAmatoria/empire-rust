//! Error handling for the Empire framework.
//!
//! This module defines all error types used throughout the framework.
//! It provides a unified error handling system with detailed error information
//! and proper error conversion traits.

use std::error::Error;
use std::fmt;
use std::io;

/// Main error type for the Empire framework.
///
/// This enum represents all possible errors that can occur during the operation
/// of the Empire framework. Each variant includes detailed error information.
#[derive(Debug, thiserror::Error)]
pub enum EmpireError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    /// Authentication-related errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    /// Command execution errors
    #[error("Command error: {0}")]
    Command(#[from] CommandError),
    
    /// File operation errors
    #[error("File error: {0}")]
    File(#[from] FileError),
    
    /// Message serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    
    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// Unknown or unexpected errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Network-related errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Connection establishment failed
    #[error("Failed to establish connection: {0}")]
    ConnectionFailed(String),
    
    /// Connection was lost
    #[error("Connection lost: {0}")]
    ConnectionLost(String),
    
    /// Timeout occurred
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Authentication-related errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Invalid credentials provided
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    /// Authentication token expired
    #[error("Authentication token expired")]
    TokenExpired,
    
    /// User not authorized for operation
    #[error("User not authorized: {0}")]
    NotAuthorized(String),
}

/// Command execution errors
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Command execution failed
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Command not found
    #[error("Command not found: {0}")]
    NotFound(String),
    
    /// Command timed out
    #[error("Command timed out: {0}")]
    Timeout(String),
    
    /// Command output too large
    #[error("Command output too large: {0}")]
    OutputTooLarge(String),
}

/// File operation errors
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// File already exists
    #[error("File already exists: {0}")]
    AlreadyExists(String),
    
    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Message serialization/deserialization errors
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    /// Serialization failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    /// Deserialization failed
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
}

/// Input validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Invalid input format
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    
    /// Input too long
    #[error("Input too long: {0}")]
    TooLong(String),
    
    /// Input contains invalid characters
    #[error("Input contains invalid characters: {0}")]
    InvalidCharacters(String),
    
    /// Required field missing
    #[error("Required field missing: {0}")]
    MissingField(String),
}

/// Helper trait for converting other error types to EmpireError
pub trait IntoEmpireError<T> {
    /// Convert an error into an EmpireError
    fn into_empire_error(self) -> Result<T, EmpireError>;
}

impl<T, E> IntoEmpireError<T> for Result<T, E>
where
    E: Into<EmpireError>,
{
    fn into_empire_error(self) -> Result<T, EmpireError> {
        self.map_err(Into::into)
    }
}

/// Helper macro for creating EmpireError variants
#[macro_export]
macro_rules! empire_error {
    ($variant:ident, $msg:expr) => {
        EmpireError::$variant($msg.into())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        EmpireError::$variant(format!($fmt, $($arg)*))
    };
} 