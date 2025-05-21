use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

/// Options for running a container in STDIO mode
#[derive(Debug, Clone)]
pub struct StdioRunOptions {
    /// Name of the container image to run
    pub image_name: String,
    
    /// Environment variables to pass to the container
    pub env_vars: Vec<String>,
    
    /// Volume mounts for the container
    pub volumes: Vec<String>,
}

/// Client for interacting with Finch container CLI
pub struct FinchClient {}

impl FinchClient {
    /// Create a new Finch client
    pub fn new() -> Self {
        Self {}
    }
    
    /// Check if Finch CLI is available on the system
    pub async fn is_finch_available(&self) -> Result<bool> {
        let output = Command::new("finch")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
            
        match output {
            Ok(mut child) => {
                let status = child.wait().await;
                Ok(status.is_ok())
            },
            Err(_) => Ok(false)
        }
    }
    
    /// Ensure Finch VM is running
    pub async fn ensure_vm_running(&self) -> Result<bool> {
        // Check current VM status
        let status = Command::new("finch")
            .args(["vm", "status"])
            .output()
            .await?;
            
        // If output contains "Running", VM is already running
        if String::from_utf8_lossy(&status.stdout).to_lowercase().contains("running") {
            return Ok(true);
        }
        
        // Try to start VM
        let start_cmd = Command::new("finch")
            .args(["vm", "start"])
            .stdout(Stdio::null())
            .spawn();
            
        match start_cmd {
            Ok(mut child) => {
                let status = child.wait().await;
                Ok(status.is_ok())
            },
            Err(_) => Ok(false)
        }
    }
    
    /// Run a container in STDIO mode
    pub async fn run_stdio_container(&self, options: &StdioRunOptions) -> Result<()> {
        // Ensure VM is running
        if !self.ensure_vm_running().await? {
            return Err(anyhow::anyhow!("Failed to start Finch VM"));
        }
        
        // Build command
        let mut cmd = Command::new("finch");
        cmd.arg("run")
           .arg("--rm")
           .arg("-i")
           .arg("-e")
           .arg("MCP_ENABLED=true")
           .arg("-e")
           .arg("MCP_STDIO=true");
        
        // Add custom environment variables
        for env in &options.env_vars {
            cmd.arg("-e").arg(env);
        }
        
        // Add volume mounts
        for volume in &options.volumes {
            cmd.arg("-v").arg(volume);
        }
        
        // Add image name
        cmd.arg(&options.image_name);
        
        // Run with stdio inheritance for piping
        log::debug!("Running finch command: {:?}", cmd);
        
        let mut child = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        
        // Wait for the process to complete
        let status = child.wait().await?;
        
        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Container exited with non-zero status code: {}", status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_is_finch_available() {
        // This is a basic test - it will only pass if finch is actually installed,
        // which is fine for our purposes
        let client = FinchClient::new();
        let result = client.is_finch_available().await;
        
        // Just assert that the function runs without errors
        assert!(result.is_ok());
    }
}