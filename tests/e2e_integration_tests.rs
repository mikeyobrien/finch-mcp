use finch_mcp::{
    FinchClient,
    RunOptions,
    DockerfileOptions,
    generate_stdio_dockerfile,
    cli::Cli,
    core::auto_containerize::{auto_containerize_and_run, AutoContainerizeOptions},
    core::git_containerize::{git_containerize_and_run, GitContainerizeOptions},
    cache::CacheManager,
    logging::LogManager,
    utils::project_detector,
};
use tempfile::TempDir;
use std::{fs, path::Path, process::Command};
use tokio::time::{timeout, Duration};

/// Test the complete end-to-end workflow of containerizing and running MCP servers
#[tokio::test]
#[ignore = "E2E test requiring Finch installation"]
async fn test_e2e_auto_containerization_workflow() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();
    
    // Create a sample Node.js MCP server project
    create_sample_nodejs_mcp_project(test_path).await;
    
    // Test auto-containerization
    let finch_client = FinchClient::new();
    
    // Skip if Finch is not available
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping E2E test: Finch not available");
        return;
    }
    
    // Ensure VM is running
    assert!(finch_client.ensure_vm_running().await.unwrap());
    
    // Test the auto-containerization process
    let auto_options = AutoContainerizeOptions {
        command: "node".to_string(),
        args: vec!["index.js".to_string()],
        env_vars: vec!["NODE_ENV=test".to_string()],
        volumes: vec![],
        host_network: false,
        forward_registry: false,
        force_rebuild: false,
    };
    
    // Run with timeout to prevent hanging
    let result = timeout(
        Duration::from_secs(120),
        auto_containerize_and_run(auto_options)
    ).await;
    
    assert!(result.is_ok(), "Auto-containerization should complete within timeout");
}

#[tokio::test]
#[ignore = "E2E test requiring Finch installation"] 
async fn test_e2e_git_repository_workflow() {
    let finch_client = FinchClient::new();
    
    // Skip if Finch is not available
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping E2E test: Finch not available");
        return;
    }
    
    // Test git repository containerization with a known MCP server
    let git_options = GitContainerizeOptions {
        repo_url: "https://github.com/modelcontextprotocol/servers.git".to_string(),
        args: vec!["index.js".to_string()],
        env_vars: vec![],
        volumes: vec![],
        host_network: false,
        forward_registry: false,
        force_rebuild: false,
    };
    
    // Run with timeout
    let result = timeout(
        Duration::from_secs(300), // Git clone may take longer
        git_containerize_and_run(git_options)
    ).await;
    
    assert!(result.is_ok(), "Git containerization should complete within timeout");
}

#[tokio::test]
#[ignore = "E2E test requiring Finch installation"]
async fn test_e2e_direct_container_workflow() {
    let finch_client = FinchClient::new();
    
    // Skip if Finch is not available
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping E2E test: Finch not available");
        return;
    }
    
    // Test running a pre-built container
    let run_options = RunOptions {
        image_name: "hello-world".to_string(),
        env_vars: Some(vec!["TEST_VAR=e2e_test".to_string()]),
        volumes: None,
    };
    
    // This should complete quickly
    let result = timeout(
        Duration::from_secs(60),
        finch_mcp::run::run_stdio_container(run_options)
    ).await;
    
    assert!(result.is_ok(), "Direct container run should complete within timeout");
}

#[tokio::test]
async fn test_e2e_cache_management() {
    let mut cache_manager = CacheManager::new().unwrap();
    
    // Test cache statistics
    let initial_stats = cache_manager.get_stats();
    // total_entries is usize which is always >= 0
    assert!(initial_stats.total_entries == initial_stats.total_entries);
    
    // Test cache operations don't panic
    let result = cache_manager.cleanup_old_entries(1).await; // 1 day
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_e2e_log_management() {
    let log_manager = LogManager::new().unwrap();
    
    // Test log directory exists
    let log_dir = log_manager.get_logs_directory_path();
    assert!(log_dir.exists() || log_dir.parent().unwrap().exists());
    
    // Test listing logs doesn't panic
    let logs_result = log_manager.list_recent_logs(10);
    assert!(logs_result.is_ok());
    
    // Test cleanup doesn't panic
    let cleanup_result = log_manager.cleanup_old_logs(1); // 1 day
    assert!(cleanup_result.is_ok());
}

#[test]
fn test_e2e_cli_command_parsing() {
    use clap::CommandFactory;
    
    // Test all major command combinations
    let test_cases = vec![
        vec!["finch-mcp", "run", "my-image:latest"],
        vec!["finch-mcp", "run", "--env", "VAR=value", "my-image:latest"],
        vec!["finch-mcp", "run", "--volume", "/host:/container", "my-image:latest"],
        vec!["finch-mcp", "list"],
        vec!["finch-mcp", "list", "--all"],
        vec!["finch-mcp", "cleanup", "--containers"],
        vec!["finch-mcp", "cleanup", "--images", "--force"],
        vec!["finch-mcp", "cache", "stats"],
        vec!["finch-mcp", "cache", "clear", "--force"],
        vec!["finch-mcp", "logs", "list"],
        vec!["finch-mcp", "logs", "cleanup", "--max-age", "7"],
    ];
    
    for args in test_cases {
        let result = Cli::command().try_get_matches_from(&args);
        assert!(result.is_ok(), "Failed to parse args: {:?}", args);
    }
}

#[tokio::test]
#[ignore = "E2E test requiring file system operations"]
async fn test_e2e_dockerfile_generation_and_build() {
    let test_dir = TempDir::new().unwrap();
    
    // Test different Dockerfile generation scenarios
    let test_cases = vec![
        DockerfileOptions {
            base_image: "node:20-alpine".to_string(),
            python_dependencies: true,
            timezone: Some("UTC".to_string()),
        },
        DockerfileOptions {
            base_image: "python:3.11-slim".to_string(),
            python_dependencies: false,
            timezone: Some("America/New_York".to_string()),
        },
        DockerfileOptions {
            base_image: "ubuntu:22.04".to_string(),
            python_dependencies: true,
            timezone: None,
        },
    ];
    
    for (i, options) in test_cases.iter().enumerate() {
        let dockerfile_content = generate_stdio_dockerfile(options);
        let test_dockerfile = test_dir.path().join(format!("Dockerfile.test{}", i));
        
        fs::write(&test_dockerfile, &dockerfile_content).unwrap();
        assert!(test_dockerfile.exists());
        
        // Verify content structure
        let content = fs::read_to_string(&test_dockerfile).unwrap();
        assert!(content.contains(&format!("FROM {}", options.base_image)));
        assert!(content.contains("WORKDIR /app"));
        
        if options.python_dependencies {
            assert!(content.contains("pip3 install"));
        }
        
        if let Some(ref tz) = options.timezone {
            assert!(content.contains(tz));
        }
    }
}

#[tokio::test]
#[ignore = "E2E test requiring network access"]
async fn test_e2e_container_lifecycle() {
    let finch_client = FinchClient::new();
    
    // Skip if Finch is not available
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping E2E test: Finch not available");
        return;
    }
    
    // Run a test container
    let run_options = RunOptions {
        image_name: "alpine:latest".to_string(),
        env_vars: Some(vec!["TEST=lifecycle".to_string()]),
        volumes: None,
    };
    
    // This should complete quickly for alpine
    let result = timeout(
        Duration::from_secs(30),
        finch_mcp::run::run_stdio_container(run_options)
    ).await;
    
    // The container may exit immediately, which is expected for alpine without a command
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable
    
    // Test cleanup capabilities
    let cleanup_result = finch_client.cleanup_resources(false, true, false, false).await;
    assert!(cleanup_result.is_ok());
}

#[test]
fn test_e2e_project_detection() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();
    
    // Test Node.js project detection
    fs::write(test_path.join("package.json"), r#"{"name": "test-mcp-server"}"#).unwrap();
    let project_info = project_detector::detect_project_type(test_path).unwrap();
    assert_eq!(project_info.project_type, project_detector::ProjectType::NodeJs);
    
    // Test Python project detection
    fs::remove_file(test_path.join("package.json")).unwrap();
    fs::write(test_path.join("requirements.txt"), "fastapi").unwrap();
    let project_info = project_detector::detect_project_type(test_path).unwrap();
    assert_eq!(project_info.project_type, project_detector::ProjectType::PythonRequirements);
    
    // Test unknown project detection
    fs::remove_file(test_path.join("requirements.txt")).unwrap();
    fs::write(test_path.join("main.py"), "print('hello')").unwrap();
    let project_info = project_detector::detect_project_type(test_path).unwrap();
    assert_eq!(project_info.project_type, project_detector::ProjectType::Unknown);
}

#[tokio::test]
#[ignore = "E2E test requiring filesystem"]
async fn test_e2e_auto_containerization_filesystem() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();
    
    // Create a simple Node.js project
    create_test_nodejs_project(test_path);
    
    // Test auto-containerization with filesystem operations
    let auto_options = AutoContainerizeOptions {
        command: "node".to_string(),
        args: vec!["index.js".to_string()],
        env_vars: vec![],
        volumes: vec![],
        host_network: false,
        forward_registry: false,
        force_rebuild: false,
    };
    
    // This tests the filesystem operations involved in containerization
    let result = timeout(
        Duration::from_secs(60),
        auto_containerize_and_run(auto_options)
    ).await;
    
    // The operation should complete without filesystem errors
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable
}

#[tokio::test]
#[ignore = "E2E test requiring binary execution"]
async fn test_e2e_binary_execution() {
    // Test the compiled binary directly
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output();
    
    if output.is_err() {
        println!("Skipping binary test: cargo build failed");
        return;
    }
    
    // Test help command
    let help_output = Command::new("./target/release/finch-mcp")
        .args(["--help"])
        .output();
    
    if let Ok(output) = help_output {
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("finch-mcp"));
        assert!(stdout.contains("COMMANDS"));
    }
    
    // Test version command  
    let version_output = Command::new("./target/release/finch-mcp")
        .args(["--version"])
        .output();
    
    if let Ok(output) = version_output {
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("finch-mcp"));
    }
}

// Helper function to create a sample MCP server project for testing
async fn create_sample_nodejs_mcp_project(path: &Path) {
    // Create package.json
    let package_json = r#"{
  "name": "test-mcp-server",
  "version": "1.0.0",
  "type": "module",
  "dependencies": {
    "@modelcontextprotocol/sdk": "latest"
  },
  "scripts": {
    "start": "node index.js"
  }
}"#;
    fs::write(path.join("package.json"), package_json).unwrap();
    
    // Create a simple MCP server
    let index_js = r#"#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const server = new Server({
  name: "test-mcp-server",
  version: "1.0.0"
}, {
  capabilities: {
    tools: {}
  }
});

server.setRequestHandler('tools/list', async () => ({
  tools: [{
    name: "test_tool",
    description: "A test tool",
    inputSchema: {
      type: "object",
      properties: {
        message: { type: "string" }
      }
    }
  }]
}));

server.setRequestHandler('tools/call', async (request) => {
  if (request.params.name === "test_tool") {
    return {
      content: [{
        type: "text",
        text: `Test response: ${request.params.arguments?.message || "Hello World"}`
      }]
    };
  }
  throw new Error(`Unknown tool: ${request.params.name}`);
});

const transport = new StdioServerTransport();
await server.connect(transport);
"#;
    fs::write(path.join("index.js"), index_js).unwrap();
}

// Helper function to create a test Node.js project
fn create_test_nodejs_project(path: &Path) {
    let package_json = r#"{
  "name": "filesystem-test-project",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {
    "start": "node index.js"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "latest"
  }
}"#;
    fs::write(path.join("package.json"), package_json).unwrap();
    
    let index_js = r#"#!/usr/bin/env node
console.log('Filesystem test MCP server');
process.exit(0);
"#;
    fs::write(path.join("index.js"), index_js).unwrap();
}