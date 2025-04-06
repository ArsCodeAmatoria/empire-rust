//! Integration tests for server-client interactions in Empire-Rust.
//!
//! This module contains integration tests that verify the interaction between
//! the server and client components of the Empire framework. These tests
//! simulate real-world usage scenarios and verify end-to-end functionality.

use empire_rust::server::Server;
use empire_rust::client::Client;
use empire_rust::core::EmpireError;
use std::time::Duration;
use tokio::time;

/// Test suite for server-client communication.
///
/// This module contains tests that verify the complete communication flow
/// between a server and client, including connection establishment,
/// authentication, and command execution.
#[cfg(test)]
mod communication_tests {
    use super::*;

    /// Test complete server-client interaction flow.
    ///
    /// This test verifies the following sequence of events:
    /// 1. Server startup and listening
    /// 2. Client connection and authentication
    /// 3. Command execution
    /// 4. Result retrieval
    /// 5. Clean shutdown
    #[tokio::test]
    async fn test_complete_interaction() {
        // Create and start the server
        let server = Server::new("127.0.0.1:0".to_string());
        let server_addr = server.local_addr().unwrap();
        
        let server_handle = tokio::spawn(async move {
            server.start().await.unwrap();
        });

        // Wait for server to start
        time::sleep(Duration::from_millis(100)).await;

        // Create and connect the client
        let client = Client::new(
            "127.0.0.1".to_string(),
            server_addr.port(),
            "test_user".to_string(),
            "test_pass".to_string()
        ).unwrap();

        // Connect and authenticate
        assert!(client.connect().await.is_ok());

        // Execute a command
        let result = client.execute_command("whoami", vec![]).await;
        assert!(result.is_ok());

        // Cleanup
        server_handle.abort();
    }

    /// Test server handling of multiple clients.
    ///
    /// This test verifies that the server can handle multiple concurrent
    /// client connections and process their requests independently.
    #[tokio::test]
    async fn test_multiple_clients() {
        // Create and start the server
        let server = Server::new("127.0.0.1:0".to_string());
        let server_addr = server.local_addr().unwrap();
        
        let server_handle = tokio::spawn(async move {
            server.start().await.unwrap();
        });

        // Wait for server to start
        time::sleep(Duration::from_millis(100)).await;

        // Create multiple clients
        let mut client_handles = vec![];
        for i in 0..3 {
            let client = Client::new(
                "127.0.0.1".to_string(),
                server_addr.port(),
                format!("test_user_{}", i),
                "test_pass".to_string()
            ).unwrap();

            let handle = tokio::spawn(async move {
                // Connect and authenticate
                assert!(client.connect().await.is_ok());

                // Execute a command
                let result = client.execute_command("echo", vec!["Hello".to_string()]).await;
                assert!(result.is_ok());
            });

            client_handles.push(handle);
        }

        // Wait for all clients to complete
        for handle in client_handles {
            handle.await.unwrap();
        }

        // Cleanup
        server_handle.abort();
    }

    /// Test server's handling of client disconnections.
    ///
    /// This test verifies that the server properly handles client
    /// disconnections and cleans up associated resources.
    #[tokio::test]
    async fn test_client_disconnection() {
        // Create and start the server
        let server = Server::new("127.0.0.1:0".to_string());
        let server_addr = server.local_addr().unwrap();
        
        let server_handle = tokio::spawn(async move {
            server.start().await.unwrap();
        });

        // Wait for server to start
        time::sleep(Duration::from_millis(100)).await;

        // Create and connect a client
        let client = Client::new(
            "127.0.0.1".to_string(),
            server_addr.port(),
            "test_user".to_string(),
            "test_pass".to_string()
        ).unwrap();

        // Connect and authenticate
        assert!(client.connect().await.is_ok());

        // Disconnect the client
        client.disconnect().await;

        // Verify server has cleaned up
        time::sleep(Duration::from_millis(100)).await;
        assert_eq!(server.agents.len(), 0);

        // Cleanup
        server_handle.abort();
    }
} 