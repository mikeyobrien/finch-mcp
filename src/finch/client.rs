use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use log::{info, warn, debug};

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
    
    /// Check if Finch VM is initialized (exists)
    pub async fn is_vm_initialized(&self) -> Result<bool> {
        debug!("Checking if Finch VM is initialized");
        let output = Command::new("finch")
            .args(["vm", "status"])
            .output()
            .await?;
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        debug!("VM status stdout: {}", stdout);
        debug!("VM status stderr: {}", stderr);
        
        // If the VM doesn't exist, status command will fail or return specific messages
        // Common indicators that VM needs initialization:
        // - "does not exist" 
        // - "nonexistent"
        // - "not found"
        // - Command fails with specific exit codes
        
        if !output.status.success() {
            // Command failed, likely VM doesn't exist
            let error_msg = stderr.to_lowercase();
            if error_msg.contains("does not exist") || 
               error_msg.contains("nonexistent") || 
               error_msg.contains("not found") ||
               error_msg.contains("no such") {
                return Ok(false);
            }
        }
        
        let status_text = stdout.to_lowercase();
        // If we get any status (Running, Stopped, etc.), VM exists
        Ok(status_text.contains("running") || 
           status_text.contains("stopped") || 
           status_text.contains("stopping"))
    }
    
    /// Initialize Finch VM for first-time users
    pub async fn initialize_vm(&self) -> Result<()> {
        info!("🚀 Initializing Finch VM for first-time use...");
        info!("This may take a few minutes to download and set up the VM.");
        
        let mut child = Command::new("finch")
            .args(["vm", "init"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
            
        let status = child.wait().await?;
        
        if status.success() {
            info!("✅ Finch VM initialized successfully!");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to initialize Finch VM: exit code {}", status))
        }
    }
    
    /// Ensure Finch VM is running (with automatic initialization if needed)
    pub async fn ensure_vm_running(&self) -> Result<bool> {
        debug!("Ensuring Finch VM is running");
        
        // First check if VM is initialized
        if !self.is_vm_initialized().await? {
            warn!("Finch VM not found. Initializing for first-time use...");
            self.initialize_vm().await?;
        }
        
        // Check current VM status
        let status = Command::new("finch")
            .args(["vm", "status"])
            .output()
            .await?;
            
        let status_text = String::from_utf8_lossy(&status.stdout).to_lowercase();
        debug!("VM status: {}", status_text);
        
        // If output contains "Running", VM is already running
        if status_text.contains("running") {
            debug!("VM is already running");
            return Ok(true);
        }
        
        // Try to start VM
        info!("🔄 Starting Finch VM...");
        let mut start_child = Command::new("finch")
            .args(["vm", "start"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
            
        let start_status = start_child.wait().await?;
        
        if start_status.success() {
            info!("✅ Finch VM started successfully");
            Ok(true)
        } else {
            Err(anyhow::anyhow!("Failed to start Finch VM: exit code {}", start_status))
        }
    }
    
    /// Run a container in STDIO mode
    pub async fn run_stdio_container(&self, options: &StdioRunOptions) -> Result<()> {
        // Ensure VM is running (with auto-initialization if needed)
        debug!("Ensuring Finch VM is ready");
        self.ensure_vm_running().await?;
        
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
    
    #[tokio::test]
    async fn test_is_vm_initialized() {
        let client = FinchClient::new();
        
        // This test will only run meaningfully if Finch is installed
        if client.is_finch_available().await.unwrap_or(false) {
            let result = client.is_vm_initialized().await;
            assert!(result.is_ok());
            
            // If Finch is available, the VM should typically be initialized
            // (unless this is a completely fresh install)
            let is_initialized = result.unwrap();
            println!("VM initialized: {}", is_initialized);
        }
    }
    
    #[tokio::test]
    async fn test_ensure_vm_running() {
        let client = FinchClient::new();
        
        // This test will only run meaningfully if Finch is installed
        if client.is_finch_available().await.unwrap_or(false) {
            let result = client.ensure_vm_running().await;
            assert!(result.is_ok());
            
            // If successful, VM should be running
            if let Ok(is_running) = result {
                assert!(is_running);
                println!("VM running after ensure_vm_running: {}", is_running);
            }
        }
    }
}