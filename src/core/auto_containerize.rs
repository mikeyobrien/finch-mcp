use std::fs;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;

use crate::utils::command_detector::{detect_command_type, generate_dockerfile_content, CommandType};
use crate::finch::client::{FinchClient, StdioRunOptions};
use crate::cache::{CacheManager, ContentHasher, hash_build_options};

pub struct AutoContainerizeOptions {
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
    pub host_network: bool,
    pub forward_registry: bool,
}

pub async fn auto_containerize_and_run(options: AutoContainerizeOptions) -> Result<()> {
    use console::style;
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the command
    let content_hash = content_hasher.hash_command(&options.command, &options.args)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    let command_key = format!("{} {}", options.command, options.args.join(" "));
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&command_key, &content_hash, &build_options_hash).await {
        println!("{} Using cached image: {}", style("⚡").yellow(), style(&cached_image).cyan());
        info!("Cache hit for command: {}", command_key);
        
        // Build extra args environment variable if needed
        let mut env_vars = options.env_vars;
        if !options.args.is_empty() {
            let extra_args = options.args.join(" ");
            env_vars.push(format!("EXTRA_ARGS={}", extra_args));
        }
        
        // Run the cached container
        println!("{} Starting server...\n", style("🚀").green());
        info!("Running cached auto-containerized command");
        let finch_client = FinchClient::new();
        let run_options = StdioRunOptions {
            image_name: cached_image,
            env_vars,
            volumes: options.volumes,
            host_network: options.host_network,
        };
        
        return finch_client.run_stdio_container(&run_options).await;
    }
    
    // Cache miss - need to build
    println!("{} Cache miss - building container...", style("🔨").blue());
    
    // Detect command type
    let command_details = detect_command_type(&options.command, &options.args);
    debug!("Detected command type: {:?}", command_details);
    
    // Generate deterministic image name based on content hash
    let image_name = cache_manager.generate_cached_image_name(&content_hash, &format!("{:?}", command_details.cmd_type));
    
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
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    // Add host network option if enabled
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(temp_dir.path());
    
    let build_status = build_command
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute finch build command")?;
    
    if !build_status.success() {
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &command_key,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", command_details.cmd_type),
    )?;
    
    println!("{} Image cached for future use", style("💾").green());
    
    // Build extra args environment variable if needed
    let mut env_vars = options.env_vars;
    
    // For some command types (like NPX), arguments are already built into the CMD
    // Only add EXTRA_ARGS for command types that use the ${EXTRA_ARGS} placeholder generically
    let should_add_extra_args = match command_details.cmd_type {
        CommandType::NodeNpx => false, // NPX args already built into CMD
        _ => !options.args.is_empty(), // Other commands can use EXTRA_ARGS
    };
    
    if should_add_extra_args {
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
        host_network: options.host_network,
    };
    
    finch_client.run_stdio_container(&run_options).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
            host_network: false,
            forward_registry: false,
        };
        
        let result = auto_containerize_and_run(options).await;
        assert!(result.is_ok());
    }
}