//! Test suite for the Empire-Rust framework.
//!
//! This module contains integration tests and helper functions for testing
//! the Empire framework's functionality. It includes tests for server and
//! client interactions, command execution, and error handling.

use empire_rust::core::{EmpireError, Message, MessageHandler};
use empire_rust::server::Server;
use empire_rust::client::Client;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::time::Duration;
use tokio::time;

/// Helper function to create a test server instance.
///
/// This function creates a new server instance bound to the specified port
/// on the localhost interface.
///
/// # Arguments
///
/// * `port` - The port number to bind the server to
///
/// # Returns
///
/// A new `Server` instance configured for testing
pub async fn create_test_server(port: u16) -> Server {
    Server::new(format!("127.0.0.1:{}", port))
}

/// Helper function to create a test client instance.
///
/// This function creates a new client instance configured to connect to
/// a server running on the specified port on the localhost interface.
///
/// # Arguments
///
/// * `port` - The port number of the server to connect to
///
/// # Returns
///
/// * `Ok(Client)` if the client was created successfully
/// * `Err(EmpireError)` if an error occurred
pub async fn create_test_client(port: u16) -> Result<Client, EmpireError> {
    Client::new(
        "127.0.0.1".to_string(),
        port,
        "test_user".to_string(),
        "test_pass".to_string()
    )
}

/// Helper function to wait for a condition with timeout.
///
/// This function repeatedly checks a condition until it becomes true or
/// the specified timeout is reached. It's useful for testing asynchronous
/// operations that may take some time to complete.
///
/// # Arguments
///
/// * `condition` - A closure that returns a future evaluating to a boolean
/// * `timeout` - The maximum time to wait for the condition to become true
///
/// # Returns
///
/// `true` if the condition became true before the timeout, `false` otherwise
pub async fn wait_for<F, T>(mut condition: F, timeout: Duration) -> bool
where
    F: FnMut() -> T,
    T: std::future::Future<Output = bool>,
{
    let start = time::Instant::now();
    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        time::sleep(Duration::from_millis(100)).await;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test server creation functionality.
    ///
    /// This test verifies that a server can be created successfully and
    /// that it initializes with empty agent and task lists.
    #[tokio::test]
    async fn test_server_creation() {
        let server = create_test_server(1337).await;
        assert_eq!(server.agents.len(), 0);
        assert_eq!(server.tasks.len(), 0);
    }

    /// Test client creation functionality.
    ///
    /// This test verifies that a client can be created successfully with
    /// the specified connection parameters.
    #[tokio::test]
    async fn test_client_creation() {
        let client = create_test_client(1337).await;
        assert!(client.is_ok());
    }

    /// Test authentication process.
    ///
    /// This test verifies the complete authentication flow between a client
    /// and server, including:
    /// 1. Server startup
    /// 2. Client connection
    /// 3. Authentication request
    /// 4. Authentication response
    #[tokio::test]
    async fn test_authentication() {
        let server = create_test_server(1338).await;
        let client = create_test_client(1338).await.unwrap();

        // Start server in background
        let server_handle = tokio::spawn(async move {
            server.start().await.unwrap();
        });

        // Try to connect client
        let result = client.connect().await;
        assert!(result.is_ok());

        // Cleanup
        server_handle.abort();
    }
} 