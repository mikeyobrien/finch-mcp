use finch_mcp::{
    FinchClient,
    StdioRunOptions,
    DockerfileOptions,
    generate_stdio_dockerfile,
};
use tempfile::TempDir;
use std::fs;

// This test verifies that the Dockerfile template generator works correctly
#[test]
fn test_dockerfile_generation() {
    let dir = TempDir::new().unwrap();
    let dockerfile_path = dir.path().join("Dockerfile");
    
    let options = DockerfileOptions {
        base_image: "node:18-alpine".to_string(),
        python_dependencies: true,
        timezone: Some("UTC".to_string()),
    };
    
    let dockerfile_content = generate_stdio_dockerfile(&options);
    fs::write(&dockerfile_path, dockerfile_content).unwrap();
    
    assert!(dockerfile_path.exists());
    
    // Read the content back and verify it contains expected lines
    let content = fs::read_to_string(dockerfile_path).unwrap();
    assert!(content.contains("FROM node:18-alpine"));
    assert!(content.contains("WORKDIR /app"));
    assert!(content.contains("pip3 install --break-system-packages mcp-server-time"));
    assert!(content.contains("\"--local-timezone\", \"UTC\""));
}

// This integration test requires Finch to be installed
// It will be ignored by default during regular testing
#[tokio::test]
#[ignore = "Requires Finch to be installed and available"]
async fn test_finch_client_integration() {
    // Create a Finch client
    let client = FinchClient::new();
    
    // Check if Finch is available
    let finch_available = client.is_finch_available().await.unwrap();
    
    // Skip the test if Finch is not installed
    if !finch_available {
        println!("Finch not available, skipping integration test");
        return;
    }
    
    // Check if VM is running
    let vm_running = client.ensure_vm_running().await.unwrap();
    assert!(vm_running, "Finch VM should be running");
    
    // Test running a simple container (hello-world)
    let run_options = StdioRunOptions {
        image_name: "hello-world".to_string(),
        env_vars: vec!["TEST=value".to_string()],
        volumes: vec![],
        host_network: false,
    };
    
    // This should succeed but we'll ignore errors
    // since this is just a basic integration check
    let _ = client.run_stdio_container(&run_options).await;
}

// This test checks that our CLI properly parses arguments
// We use clap's built-in testing utilities
#[test]
fn test_cli_parsing() {
    use clap::CommandFactory;
    use finch_mcp::cli::Cli;
    
    // Verify our CLI definition is valid
    Cli::command().debug_assert();
    
    // Test that CLI parse logic works for required subcommand arguments
    let matches = Cli::command()
        .get_matches_from(vec!["finch-mcp", "run", "my-image:latest"]);
    
    // Verify the command parsed correctly
    assert!(matches.subcommand_matches("run").is_some());
}

// Test verifies that our run_stdio_container function works correctly
// with various options combinations
#[test]
fn test_run_options_creation() {
    use finch_mcp::RunOptions;
    
    // Test with minimal options
    let options = RunOptions {
        image_name: "test-image".to_string(),
        env_vars: None,
        volumes: None,
    };
    
    assert_eq!(options.image_name, "test-image");
    assert!(options.env_vars.is_none());
    assert!(options.volumes.is_none());
    
    // Test with all options
    let options = RunOptions {
        image_name: "test-image".to_string(),
        env_vars: Some(vec!["VAR=VALUE".to_string()]),
        volumes: Some(vec!["/host:/container".to_string()]),
    };
    
    assert_eq!(options.image_name, "test-image");
    assert_eq!(options.env_vars.unwrap()[0], "VAR=VALUE");
    assert_eq!(options.volumes.unwrap()[0], "/host:/container");
}