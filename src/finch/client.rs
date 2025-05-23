use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use log::{info, warn, debug};
use console::style;
use crate::{status, output};

/// Options for running a container in STDIO mode
#[derive(Debug, Clone)]
pub struct StdioRunOptions {
    /// Name of the container image to run
    pub image_name: String,
    
    /// Environment variables to pass to the container
    pub env_vars: Vec<String>,
    
    /// Volume mounts for the container
    pub volumes: Vec<String>,
    
    /// Use host network for the container
    pub host_network: bool,
}

/// Client for interacting with Finch container CLI
#[derive(Default)]
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
        if !output::is_quiet_mode() {
            info!("🚀 Initializing Finch VM for first-time use...");
            info!("This may take a few minutes to download and set up the VM.");
        }
        
        let mut child = Command::new("finch")
            .args(["vm", "init"])
            .stdin(Stdio::null())
            .stdout(if output::is_quiet_mode() { Stdio::null() } else { Stdio::inherit() })
            .stderr(if output::is_quiet_mode() { Stdio::null() } else { Stdio::inherit() })
            .spawn()?;
            
        let status = child.wait().await?;
        
        if status.success() {
            if !output::is_quiet_mode() {
                info!("✅ Finch VM initialized successfully!");
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to initialize Finch VM: exit code {}", status))
        }
    }
    
    /// Fast VM check - assumes VM is likely already running
    pub async fn ensure_vm_running_fast(&self) -> Result<bool> {
        debug!("Fast VM check for direct container execution");
        
        // Try a quick status check first
        let status = Command::new("finch")
            .args(["vm", "status"])
            .output()
            .await?;
            
        let status_text = String::from_utf8_lossy(&status.stdout).to_lowercase();
        
        // If already running, return immediately
        if status_text.contains("running") {
            debug!("VM is already running");
            return Ok(true);
        }
        
        // If not running, fall back to full initialization flow
        debug!("VM not running, falling back to full ensure_vm_running");
        self.ensure_vm_running().await
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
        if !output::is_quiet_mode() {
            info!("🔄 Starting Finch VM...");
        }
        let mut start_child = Command::new("finch")
            .args(["vm", "start"])
            .stdout(Stdio::null())
            .stderr(if output::is_quiet_mode() { Stdio::null() } else { Stdio::inherit() })
            .spawn()?;
            
        let start_status = start_child.wait().await?;
        
        if start_status.success() {
            if !output::is_quiet_mode() {
                info!("✅ Finch VM started successfully");
            }
            Ok(true)
        } else {
            Err(anyhow::anyhow!("Failed to start Finch VM: exit code {}", start_status))
        }
    }
    
    /// Run a container in STDIO mode
    pub async fn run_stdio_container(&self, options: &StdioRunOptions) -> Result<()> {
        // For direct container mode, do a faster VM check
        debug!("Ensuring Finch VM is ready");
        self.ensure_vm_running_fast().await?;
        
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
        
        // Add host network if enabled
        if options.host_network {
            cmd.arg("--network").arg("host");
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
    
    /// List finch-mcp containers and images
    pub async fn list_resources(&self, show_all: bool) -> Result<()> {
        status!("\n{} Finch-MCP Resources", style("📋").blue().bold());
        status!("{}", "=".repeat(50));
        
        // List containers
        status!("\n{} Containers:", style("🐳").cyan());
        let container_args = if show_all {
            vec!["ps", "-a", "--filter", "name=mcp-", "--format", "table {{.Names}}\\t{{.Image}}\\t{{.Status}}\\t{{.CreatedAt}}"]
        } else {
            vec!["ps", "--filter", "name=mcp-", "--format", "table {{.Names}}\\t{{.Image}}\\t{{.Status}}\\t{{.CreatedAt}}"]
        };
        
        let container_output = Command::new("finch")
            .args(&container_args)
            .output()
            .await?;
            
        if container_output.status.success() {
            let output = String::from_utf8_lossy(&container_output.stdout);
            if output.trim().is_empty() || !output.contains("mcp-") {
                status!("  {}", style("No finch-mcp containers found").dim());
            } else {
                print!("{}", output);
            }
        } else {
            status!("  {}", style("Error listing containers").red());
        }
        
        // List images
        status!("\n{} Images:", style("💿").green());
        let image_output = Command::new("finch")
            .args(["images", "--filter", "reference=mcp-*", "--format", "table {{.Repository}}\\t{{.Tag}}\\t{{.Size}}\\t{{.CreatedAt}}"])
            .output()
            .await?;
            
        if image_output.status.success() {
            let output = String::from_utf8_lossy(&image_output.stdout);
            if output.trim().is_empty() || !output.contains("mcp-") {
                status!("  {}", style("No finch-mcp images found").dim());
            } else {
                print!("{}", output);
            }
        } else {
            status!("  {}", style("Error listing images").red());
        }
        
        status!();
        Ok(())
    }
    
    /// Cleanup finch-mcp containers and images
    pub async fn cleanup_resources(&self, cleanup_all: bool, cleanup_containers: bool, cleanup_images: bool, force: bool) -> Result<()> {
        status!("\n{} Cleaning up Finch-MCP resources...", style("🧹").yellow().bold());
        
        let mut cleaned_something = false;
        
        // Cleanup containers
        if cleanup_all || cleanup_containers {
            status!("\n{} Removing containers...", style("🐳").cyan());
            
            // Get list of finch-mcp containers
            let container_list = Command::new("finch")
                .args(["ps", "-a", "--filter", "name=mcp-", "--format", "{{.Names}}"])
                .output()
                .await?;
                
            if container_list.status.success() {
                let containers = String::from_utf8_lossy(&container_list.stdout);
                let container_names: Vec<&str> = containers.lines().filter(|l| !l.trim().is_empty()).collect();
                
                if container_names.is_empty() {
                    status!("  {}", style("No finch-mcp containers to remove").dim());
                } else {
                    if !force {
                        status!("  Found {} containers to remove:", container_names.len());
                        for name in &container_names {
                            status!("    • {}", name);
                        }
                        print!("  Continue? [y/N]: ");
                        use std::io::{self, Write};
                        io::stdout().flush().unwrap();
                        
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        
                        if !input.trim().to_lowercase().starts_with('y') {
                            status!("  Skipped container cleanup");
                            return Ok(());
                        }
                    }
                    
                    for container in &container_names {
                        let remove_result = Command::new("finch")
                            .args(["rm", "-f", container])
                            .output()
                            .await?;
                            
                        if remove_result.status.success() {
                            status!("  {} Removed container: {}", style("✓").green(), container);
                            cleaned_something = true;
                        } else {
                            status!("  {} Failed to remove container: {}", style("✗").red(), container);
                        }
                    }
                }
            }
        }
        
        // Cleanup images
        if cleanup_all || cleanup_images {
            status!("\n{} Removing images...", style("💿").green());
            
            // Get list of finch-mcp images
            let image_list = Command::new("finch")
                .args(["images", "--filter", "reference=mcp-*", "--format", "{{.Repository}}:{{.Tag}}"])
                .output()
                .await?;
                
            if image_list.status.success() {
                let images = String::from_utf8_lossy(&image_list.stdout);
                let image_names: Vec<&str> = images.lines().filter(|l| !l.trim().is_empty()).collect();
                
                if image_names.is_empty() {
                    status!("  {}", style("No finch-mcp images to remove").dim());
                } else {
                    if !force {
                        status!("  Found {} images to remove:", image_names.len());
                        for name in &image_names {
                            status!("    • {}", name);
                        }
                        print!("  Continue? [y/N]: ");
                        use std::io::{self, Write};
                        io::stdout().flush().unwrap();
                        
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        
                        if !input.trim().to_lowercase().starts_with('y') {
                            status!("  Skipped image cleanup");
                            return Ok(());
                        }
                    }
                    
                    for image in &image_names {
                        let remove_result = Command::new("finch")
                            .args(["rmi", "-f", image])
                            .output()
                            .await?;
                            
                        if remove_result.status.success() {
                            status!("  {} Removed image: {}", style("✓").green(), image);
                            cleaned_something = true;
                        } else {
                            status!("  {} Failed to remove image: {}", style("✗").red(), image);
                        }
                    }
                }
            }
        }
        
        if cleaned_something {
            status!("\n{} Cleanup completed!", style("✨").green().bold());
        } else {
            status!("\n{} Nothing to clean up", style("ℹ").blue());
        }
        
        Ok(())
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
            status!("VM initialized: {}", is_initialized);
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
                status!("VM running after ensure_vm_running: {}", is_running);
            }
        }
    }
}