use std::sync::Arc;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::thread;

#[test]
fn test_mcp_buffer_basic() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    
    let buffer = MCPBuffer::new(1024, Duration::from_secs(5));
    
    // Test client message buffering
    assert!(buffer.buffer_client_message(b"test message".to_vec()).is_ok());
    
    // Test server message buffering (without initialize)
    assert!(buffer.buffer_server_message(b"server response".to_vec()).is_ok());
    assert!(!buffer.is_server_ready());
    
    // Test server ready detection
    let init_msg = br#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#;
    assert!(buffer.buffer_server_message(init_msg.to_vec()).is_ok());
    assert!(buffer.is_server_ready());
    
    // Test draining buffers
    let client_messages = buffer.drain_client_buffer();
    assert_eq!(client_messages.len(), 1);
    assert_eq!(client_messages[0], b"test message");
    
    let server_messages = buffer.drain_server_buffer();
    assert_eq!(server_messages.len(), 2);
}

#[test]
fn test_mcp_buffer_overflow() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    
    let buffer = MCPBuffer::new(10, Duration::from_secs(5)); // Very small buffer
    
    // This should fail due to buffer overflow
    let large_message = vec![0u8; 20];
    assert!(buffer.buffer_client_message(large_message).is_err());
}

#[test]
fn test_mcp_buffer_timeout() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    
    let buffer = MCPBuffer::new(1024, Duration::from_millis(100)); // Very short timeout
    
    // Initially should be ok
    assert!(buffer.check_timeout().is_ok());
    
    // Wait for timeout
    thread::sleep(Duration::from_millis(150));
    
    // Should now timeout
    assert!(buffer.check_timeout().is_err());
}

#[test]
fn test_mcp_proxy_echo_server() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    use finch_mcp::mcp::proxy::StdioProxy;
    
    // Create a simple echo process
    let echo_child = Command::new("sh")
        .arg("-c")
        .arg("while read line; do echo \"$line\"; done")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn echo process");
    
    // Create buffer and proxy
    let buffer = Arc::new(MCPBuffer::new(1024 * 1024, Duration::from_secs(5)));
    let proxy = StdioProxy::new(buffer.clone(), echo_child).expect("Failed to create proxy");
    
    // Run proxy in a thread
    let proxy_thread = thread::spawn(move || {
        proxy.start().expect("Proxy failed");
    });
    
    // Give the proxy time to start
    thread::sleep(Duration::from_millis(100));
    
    // The test would need a way to send input and read output
    // This is a simplified test structure
    
    // Clean up
    proxy_thread.join().expect("Proxy thread failed");
}

#[test]
fn test_mcp_readiness_detection() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    
    // Test various message formats that should trigger readiness
    let test_cases = vec![
        br#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#.to_vec(),
        br#"{"jsonrpc":"2.0","result":{"capabilities":{}}}"#.to_vec(),
    ];
    
    for (i, msg) in test_cases.iter().enumerate() {
        let buffer = MCPBuffer::new(1024, Duration::from_secs(5));
        assert!(!buffer.is_server_ready());
        
        buffer.buffer_server_message(msg.clone()).expect("Failed to buffer message");
        assert!(buffer.is_server_ready(), "Test case {} should trigger readiness", i);
    }
}

#[test]
fn test_mcp_message_ordering() {
    use finch_mcp::mcp::buffer::MCPBuffer;
    
    let buffer = MCPBuffer::new(1024, Duration::from_secs(5));
    
    // Buffer multiple messages
    for i in 0..5 {
        let msg = format!("message {}", i).into_bytes();
        buffer.buffer_client_message(msg).expect("Failed to buffer");
    }
    
    // Drain and verify order
    let messages = buffer.drain_client_buffer();
    assert_eq!(messages.len(), 5);
    
    for (i, msg) in messages.iter().enumerate() {
        let expected = format!("message {}", i).into_bytes();
        assert_eq!(*msg, expected);
    }
}

#[test]
fn test_finch_config_mcp_settings() {
    use finch_mcp::core::finch_config::FinchConfig;
    
    let yaml = r#"
mcp:
  startupTimeout: 60
  bufferSize: 2097152
  readinessPattern: "custom_ready"
  enableBuffering: false
"#;
    
    let config: FinchConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.mcp.startup_timeout, 60);
    assert_eq!(config.mcp.buffer_size, 2097152);
    assert_eq!(config.mcp.readiness_pattern, "custom_ready");
    assert!(!config.mcp.enable_buffering);
}

#[test]
fn test_finch_config_mcp_defaults() {
    use finch_mcp::core::finch_config::FinchConfig;
    
    let config = FinchConfig::default();
    assert_eq!(config.mcp.startup_timeout, 30);
    assert_eq!(config.mcp.buffer_size, 1024 * 1024);
    assert_eq!(config.mcp.readiness_pattern, "initialize");
    assert!(config.mcp.enable_buffering);
}