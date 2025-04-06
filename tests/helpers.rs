//! Helper utilities for Empire-Rust tests.
//!
//! This module provides common utilities and helper functions used across
//! different test modules. It includes functions for setting up test
//! environments, creating test data, and managing test resources.

use empire_rust::core::{EmpireError, Message};
use empire_rust::server::Server;
use empire_rust::client::Client;
use std::time::Duration;
use tokio::time;

/// Test configuration constants.
///
/// This module contains constants used for test configuration, such as
/// timeouts, port ranges, and test credentials.
pub mod config {
    /// Default timeout for test operations in milliseconds.
    pub const TEST_TIMEOUT_MS: u64 = 5000;

    /// Port range for test servers.
    pub const TEST_PORT_RANGE: std::ops::Range<u16> = 30000..31000;

    /// Default test credentials.
    pub const TEST_USERNAME: &str = "test_user";
    pub const TEST_PASSWORD: &str = "test_pass";
}

/// Helper function to create a test server instance.
///
/// This function creates a new server instance bound to a random port
/// in the test port range. It handles the selection of an available port
/// and ensures proper server initialization.
///
/// # Arguments
///
/// * `port` - Optional port number. If None, a random port will be selected.
///
/// # Returns
///
/// A tuple containing:
/// - The server instance
/// - The address the server is bound to
pub async fn create_test_server(port: Option<u16>) -> (Server, std::net::SocketAddr) {
    let port = port.unwrap_or_else(|| {
        use std::net::TcpListener;
        for port in config::TEST_PORT_RANGE {
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return port;
            }
        }
        panic!("No available ports in test range");
    });

    let addr = format!("127.0.0.1:{}", port);
    let server = Server::new(addr);
    let server_addr = server.local_addr().unwrap();
    (server, server_addr)
}

/// Helper function to create a test client instance.
///
/// This function creates a new client instance configured to connect to
/// a server running on the specified address. It uses test credentials
/// and handles proper client initialization.
///
/// # Arguments
///
/// * `server_addr` - The address of the server to connect to
///
/// # Returns
///
/// A new `Client` instance configured for testing
pub fn create_test_client(server_addr: std::net::SocketAddr) -> Client {
    Client::new(
        "127.0.0.1".to_string(),
        server_addr.port(),
        config::TEST_USERNAME.to_string(),
        config::TEST_PASSWORD.to_string()
    ).unwrap()
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
/// * `timeout` - Optional timeout duration. If None, uses the default test timeout.
///
/// # Returns
///
/// `true` if the condition became true before the timeout, `false` otherwise
pub async fn wait_for<F, T>(mut condition: F, timeout: Option<Duration>) -> bool
where
    F: FnMut() -> T,
    T: std::future::Future<Output = bool>,
{
    let timeout = timeout.unwrap_or(Duration::from_millis(config::TEST_TIMEOUT_MS));
    let start = time::Instant::now();
    
    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        time::sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Helper function to create test messages.
///
/// This function creates various types of test messages with predefined
/// content. It's useful for testing message handling and serialization.
///
/// # Arguments
///
/// * `message_type` - The type of message to create
///
/// # Returns
///
/// A new `Message` instance of the specified type
pub fn create_test_message(message_type: &str) -> Message {
    match message_type {
        "heartbeat" => Message::Heartbeat,
        "command" => Message::CommandRequest {
            command: "test_command".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
        },
        "file" => Message::FileTransferRequest {
            filename: "test.txt".to_string(),
            size: 1024,
        },
        _ => panic!("Unknown message type: {}", message_type),
    }
}

/// Helper function to clean up test resources.
///
/// This function ensures that all test resources are properly cleaned up
/// after tests complete. It handles tasks such as stopping servers,
/// disconnecting clients, and removing temporary files.
///
/// # Arguments
///
/// * `server_handle` - Optional handle to a running server task
/// * `client` - Optional client instance to disconnect
pub async fn cleanup_test_resources(
    server_handle: Option<tokio::task::JoinHandle<()>>,
    client: Option<Client>,
) {
    if let Some(handle) = server_handle {
        handle.abort();
    }

    if let Some(client) = client {
        let _ = client.disconnect().await;
    }
} 