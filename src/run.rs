use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, error};
use std::path::Path;
use tokio::signal::ctrl_c;

use crate::finch::client::{FinchClient, StdioRunOptions};

/// Options for running an MCP server container in STDIO mode
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Name of the image to run
    pub image_name: String,
    
    /// Environment variables to pass to the container
    pub env_vars: Option<Vec<String>>,
    
    /// Volume mounts for the container
    pub volumes: Option<Vec<String>>,
}

/// Spinner helper for console output
struct Spinner {
    progress: ProgressBar,
}

impl Spinner {
    fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        
        Self { progress: pb }
    }
    
    fn succeed(&self, message: &str) {
        self.progress.finish_with_message(format!("✓ {}", message));
    }
    
    fn fail(&self, message: &str) {
        self.progress.finish_with_message(format!("✗ {}", message));
    }
}

/// Run an MCP server container in STDIO mode
pub async fn run_stdio_container(options: RunOptions) -> Result<()> {
    let spinner = Spinner::new("Preparing to run MCP server container in STDIO mode...");
    
    // Create Finch client
    let finch_client = FinchClient::new();
    
    // Check if Finch is available
    if !finch_client.is_finch_available().await? {
        spinner.fail("Finch is not installed or not available");
        return Err(anyhow::anyhow!("Finch is not installed or not available. Please install Finch from https://runfinch.com/"));
    }
    
    // Log the MCP server we're about to run
    info!("Running MCP server from image: {}", options.image_name);
    
    spinner.succeed("Starting MCP server in STDIO mode...");
    println!("Connecting to MCP Server...");
    
    // Prepare run options
    let run_options = StdioRunOptions {
        image_name: options.image_name,
        env_vars: options.env_vars.unwrap_or_default(),
        volumes: options.volumes.unwrap_or_default(),
    };

    // Setup signal handler for ctrl+c
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    // Handle Ctrl+C for clean container termination
    tokio::spawn(async move {
        if let Err(e) = ctrl_c().await {
            error!("Failed to listen for Ctrl+C: {}", e);
            return;
        }
        
        println!("\nReceived interrupt signal, shutting down...");
        let _ = tx.send(());
    });
    
    // Run the container in a separate task so we can handle interrupts
    let container_task = tokio::spawn(async move {
        finch_client.run_stdio_container(&run_options).await
    });
    
    // Wait for either the container to finish or a ctrl+c signal
    tokio::select! {
        result = container_task => {
            match result {
                Ok(container_result) => container_result,
                Err(e) => Err(anyhow::anyhow!("Container task failed: {}", e))
            }
        }
        _ = rx => {
            // Ctrl+C received
            // The container will be terminated automatically because the process
            // is terminating and the stdio streams will be closed
            println!("MCP server container terminated");
            Ok(())
        }
    }
}

// Helper function for converting relative paths to absolute
pub fn to_absolute_path(path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_string_lossy().to_string()
    } else {
        let current_dir = std::env::current_dir().unwrap_or_default();
        current_dir.join(path).to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // No imports needed here
    
    // This test is a basic example and would need to be expanded
    // with proper mocking in an actual implementation
    #[tokio::test]
    #[ignore = "Requires Finch to be installed"]
    async fn test_run_stdio_container() {
        // Only run this test if Finch is installed
        let finch_client = FinchClient::new();
        if !finch_client.is_finch_available().await.unwrap_or(false) {
            println!("Finch not available, skipping test");
            return;
        }
        
        let run_options = RunOptions {
            image_name: "hello-world".to_string(), // Use a simple public image
            env_vars: None,
            volumes: None,
        };
        
        let result = run_stdio_container(run_options).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_to_absolute_path() {
        // Relative path
        let rel_result = to_absolute_path("test/path");
        assert!(rel_result.contains("test/path"));
        assert!(Path::new(&rel_result).is_absolute());
        
        // Absolute path
        let abs_path = if cfg!(windows) { "C:\\absolute\\path" } else { "/absolute/path" };
        let abs_result = to_absolute_path(abs_path);
        assert_eq!(abs_result, abs_path);
    }
}