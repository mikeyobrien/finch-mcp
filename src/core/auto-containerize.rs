use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;
use uuid::Uuid;

use crate::utils::command_detector::{detect_command_type, generate_dockerfile_content, CommandDetails};
use crate::finch::client::{FinchClient, StdioRunOptions};

pub struct AutoContainerizeOptions {
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
}

pub async fn auto_containerize_and_run(options: AutoContainerizeOptions) -> Result<()> {
    // Detect command type
    let command_details = detect_command_type(&options.command, &options.args);
    debug!("Detected command type: {:?}", command_details);
    
    // Generate unique image name
    let image_name = format!("mcp-auto-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("default"));
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content
    let dockerfile_content = generate_dockerfile_content(&command_details);
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Build the container image
    info!("Building container image: {}", image_name);
    let build_status = Command::new("finch")
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(temp_dir.path())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute finch build command")?;
    
    if !build_status.success() {
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    // Build extra args environment variable if needed
    let mut env_vars = options.env_vars;
    
    // Check if we need to pass any arguments that weren't included in the Dockerfile's CMD
    if !options.args.is_empty() {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
    // Run the container
    info!("Running containerized command");
    let finch_client = FinchClient::new();
    let run_options = StdioRunOptions {
        image_name,
        env_vars,
        volumes: options.volumes,
    };
    
    finch_client.run_stdio_container(&run_options).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    // These tests would require finch installed to run
    // so we'll mark them as ignore for automated testing
    
    #[tokio::test]
    #[ignore]
    async fn test_auto_containerize_uvx_command() {
        let options = AutoContainerizeOptions {
            command: "uvx".to_string(),
            args: vec!["mcp-server-time".to_string(), "--local-timezone".to_string(), "UTC".to_string()],
            env_vars: vec![],
            volumes: vec![],
        };
        
        let result = auto_containerize_and_run(options).await;
        assert!(result.is_ok());
    }
}