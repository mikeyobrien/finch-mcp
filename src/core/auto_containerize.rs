use std::fs;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;

use crate::utils::command_detector::{detect_command_type, generate_dockerfile_content, CommandType};
use crate::finch::client::{FinchClient, StdioRunOptions};
use crate::cache::{CacheManager, ContentHasher, hash_build_options};
use crate::logging::LogManager;
use crate::status;

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
        status!("⚡ Using cached image: {}", style(&cached_image).cyan());
        info!("Cache hit for command: {}", command_key);
        
        // Build extra args environment variable if needed
        let mut env_vars = options.env_vars;
        if !options.args.is_empty() {
            let extra_args = options.args.join(" ");
            env_vars.push(format!("EXTRA_ARGS={}", extra_args));
        }
        
        // Run the cached container
        status!("🚀 Starting server...\n");
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
    status!("🔨 Cache miss - building container...");
    
    // Initialize logging
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("auto", &command_key)?;
    let build_start = std::time::Instant::now();
    
    // Detect command type
    let command_details = detect_command_type(&options.command, &options.args);
    debug!("Detected command type: {:?}", command_details);
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&command_key);
    let image_name = cache_manager.generate_smart_image_name(
        "auto",
        &format!("{:?}", command_details.cmd_type),
        &identifier,
        &content_hash
    );
    
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
    
    // Log build command
    log_manager.append_to_log(&log_filename, &format!("Build command: {:?}", build_command))?;
    
    let build_status = build_command
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute finch build command")?;
    
    let build_duration = build_start.elapsed().as_secs();
    
    if !build_status.success() {
        log_manager.append_to_log(&log_filename, &format!("Build failed with status: {}", build_status))?;
        log_manager.finish_build_log(&log_filename, false, build_duration)?;
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    log_manager.append_to_log(&log_filename, "Build completed successfully")?;
    log_manager.finish_build_log(&log_filename, true, build_duration)?;
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &command_key,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", command_details.cmd_type),
    )?;
    
    status!("💾 Image cached for future use");
    
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

/// Auto-containerize and run for MCP clients (build-then-run in one step)
pub async fn auto_containerize_and_run_mcp(options: AutoContainerizeOptions) -> Result<()> {
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the command
    let content_hash = content_hasher.hash_command(&options.command, &options.args)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    let command_key = format!("{} {}", options.command, options.args.join(" "));
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&command_key, &content_hash, &build_options_hash).await {
        // Run the cached container directly in MCP mode
        let mut env_vars = options.env_vars;
        if !options.args.is_empty() {
            let extra_args = options.args.join(" ");
            env_vars.push(format!("EXTRA_ARGS={}", extra_args));
        }
        
        let finch_client = FinchClient::new();
        let run_options = StdioRunOptions {
            image_name: cached_image,
            env_vars,
            volumes: options.volumes,
            host_network: options.host_network,
        };
        
        return finch_client.run_stdio_container(&run_options).await;
    }
    
    // Build the image first (with suppressed output for MCP)
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("auto-mcp", &command_key)?;
    let build_start = std::time::Instant::now();
    
    // Detect command type
    let command_details = detect_command_type(&options.command, &options.args);
    debug!("Detected command type: {:?}", command_details);
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&command_key);
    let image_name = cache_manager.generate_smart_image_name(
        "auto-mcp",
        &format!("{:?}", command_details.cmd_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content
    let dockerfile_content = generate_dockerfile_content(&command_details);
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    
    // Build the container image (suppress output for MCP)
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(temp_dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    
    let build_status = build_command.status().context("Failed to execute finch build command")?;
    let build_duration = build_start.elapsed().as_secs();
    
    if !build_status.success() {
        log_manager.append_to_log(&log_filename, &format!("Build failed with status: {}", build_status))?;
        log_manager.finish_build_log(&log_filename, false, build_duration)?;
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    log_manager.append_to_log(&log_filename, "Build completed successfully")?;
    log_manager.finish_build_log(&log_filename, true, build_duration)?;
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &command_key,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", command_details.cmd_type),
    )?;
    
    // Run the container directly
    let mut env_vars = options.env_vars;
    
    let should_add_extra_args = match command_details.cmd_type {
        CommandType::NodeNpx => false,
        _ => !options.args.is_empty(),
    };
    
    if should_add_extra_args {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
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