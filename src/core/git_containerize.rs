use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;
use serde_json::json;

use crate::utils::git_repository::GitRepository;
use crate::utils::project_detector::{detect_project_type, ProjectType, ProjectInfo};
use crate::utils::progress::run_build_with_progress;
use crate::finch::client::{FinchClient, StdioRunOptions};
use crate::cache::{CacheManager, ContentHasher, hash_build_options};
use crate::logging::LogManager;
use crate::core::finch_config::FinchConfig;
use crate::status;

pub struct GitContainerizeOptions {
    pub repo_url: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
    pub host_network: bool,
    pub forward_registry: bool,
    pub force_rebuild: bool,
}

pub struct LocalContainerizeOptions {
    pub local_path: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
    pub host_network: bool,
    pub forward_registry: bool,
    pub force_rebuild: bool,
}

pub async fn git_containerize_and_run(options: GitContainerizeOptions) -> Result<()> {
    use console::style;
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the git repository
    let content_hash = content_hasher.hash_git_repository(&options.repo_url, None)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.repo_url, &content_hash, &build_options_hash).await {
        if options.force_rebuild {
            status!("🔨 Force rebuild requested, ignoring cached image: {}", style(&cached_image).cyan());
            info!("Force rebuild for git repository: {}", options.repo_url);
        } else {
            status!("⚡ Using cached image: {}", style(&cached_image).cyan());
            status!("💡 To rebuild, use: {}", style("finch-mcp run --force <target>").yellow());
            info!("Cache hit for git repository: {}", options.repo_url);
            
            // Prepare environment variables (MCP env vars are added by finch client)
            let mut env_vars = options.env_vars;
            
            // Add extra arguments if provided
            if !options.args.is_empty() {
                let extra_args = options.args.join(" ");
                env_vars.push(format!("EXTRA_ARGS={}", extra_args));
            }
            
            // Run the cached container
            status!("🚀 Starting server...\n");
            info!("Running cached containerized git repository");
            let finch_client = FinchClient::new();
            let run_options = StdioRunOptions {
                image_name: cached_image,
                env_vars,
                volumes: options.volumes,
                host_network: options.host_network,
            };
            
            return finch_client.run_stdio_container(&run_options, None).await;
        }
    }
    
    // Cache miss - need to build
    status!("🔨 Cache miss - building container...");
    
    // Initialize logging
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("git", &options.repo_url)?;
    let build_start = std::time::Instant::now();
    
    // Parse and clone the repository
    let mut git_repo = GitRepository::new(&options.repo_url);
    
    status!("\n🔄 Cloning repository...");
    info!("Cloning repository: {}", git_repo.url);
    let repo_path = git_repo.clone_to_temp_quiet(crate::output::is_quiet_mode()).await?;
    
    // Detect the project type
    let project_info = detect_project_type(&repo_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in repository"));
    }
    
    // Load finch-mcp config if present
    let finch_config = FinchConfig::load_from_dir(&repo_path)?;
    if finch_config.is_some() {
        info!("Found .finch-mcp configuration file");
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.repo_url);
    let image_name = cache_manager.generate_smart_image_name(
        "git",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, None)?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Copy repository contents to build context
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy repository files to build context
    copy_dir_all(&repo_path, &build_context).context("Failed to copy repository to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image with progress tracking
    let project_type_str = match project_info.project_type {
        ProjectType::NodeJs | ProjectType::NodeJsMonorepo => "Node.js",
        ProjectType::PythonPoetry => "Python (Poetry)",
        ProjectType::PythonUv => "Python (uv)",
        ProjectType::PythonSetupPy => "Python (setup.py)",
        ProjectType::PythonRequirements => "Python (requirements.txt)",
        ProjectType::Rust => "Rust",
        ProjectType::Unknown => "Unknown",
    };
    
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    // Add host network option if enabled
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command.arg(&build_context);
    
    // Log build command
    log_manager.append_to_log(&log_filename, &format!("Build command: {:?}", build_command))?;
    
    let build_result = run_build_with_progress(&mut build_command, &image_name, project_type_str);
    
    let build_duration = build_start.elapsed().as_secs();
    
    match &build_result {
        Ok(_) => {
            log_manager.append_to_log(&log_filename, "Build completed successfully")?;
            log_manager.finish_build_log(&log_filename, true, build_duration)?;
        }
        Err(e) => {
            log_manager.append_to_log(&log_filename, &format!("Build failed: {}", e))?;
            log_manager.finish_build_log(&log_filename, false, build_duration)?;
        }
    }
    
    build_result?;
    
    // Tag the image with 'latest' as well
    let base_name = image_name.split(':').next().unwrap_or(&image_name);
    let latest_tag = format!("{}:latest", base_name);
    
    let tag_command = Command::new("finch")
        .args(["tag", &image_name, &latest_tag])
        .status()
        .context("Failed to tag image with latest")?;
    
    if !tag_command.success() {
        log::warn!("Failed to tag image with latest");
    }
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &options.repo_url,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    status!("💾 Image cached for future use");
    
    // Output MCP configuration
    output_mcp_config(&options.repo_url, &image_name, &options.env_vars)?;
    
    // Prepare environment variables
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    // Add extra arguments if provided
    if !options.args.is_empty() {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
    // Run the container
    status!("🚀 Starting server...\n");
    info!("Running containerized git repository");
    let finch_client = FinchClient::new();
    let run_options = StdioRunOptions {
        image_name,
        env_vars,
        volumes: options.volumes,
        host_network: options.host_network,
    };
    
    finch_client.run_stdio_container(&run_options, Some(temp_dir.path())).await
}

pub async fn local_containerize_and_run(options: LocalContainerizeOptions) -> Result<()> {
    use console::style;
    
    // Debug: Log that we're using the regular function
    log::info!("🚨 DEBUG: Using local_containerize_and_run REGULAR function");
    
    let local_path = PathBuf::from(&options.local_path);
    
    // Validate that the path exists and is a directory
    if !local_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", options.local_path));
    }
    
    if !local_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", options.local_path));
    }
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the local directory
    let content_hash = content_hasher.hash_directory(&local_path)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.local_path, &content_hash, &build_options_hash).await {
        if options.force_rebuild {
            if !crate::output::is_quiet_mode() {
                status!("🔨 Force rebuild requested, ignoring cached image: {}", style(&cached_image).cyan());
            }
            info!("Force rebuild for local directory: {}", options.local_path);
        } else {
            // In MCP mode, suppress all output to avoid corrupting the protocol
            if !crate::output::is_quiet_mode() {
                status!("⚡ Using cached image: {}", style(&cached_image).cyan());
                status!("💡 To rebuild, use: {}", style("finch-mcp run --force <target>").yellow());
            }
            info!("Cache hit for local directory: {}", options.local_path);
            
            // Prepare environment variables (MCP env vars are added by finch client)
            let mut env_vars = options.env_vars;
            
            // Add extra arguments if provided
            if !options.args.is_empty() {
                let extra_args = options.args.join(" ");
                env_vars.push(format!("EXTRA_ARGS={}", extra_args));
            }
            
            // Run the cached container
            if !crate::output::is_quiet_mode() {
                status!("🚀 Starting server...\n");
            }
            info!("Running cached containerized local directory");
            let finch_client = FinchClient::new();
            let run_options = StdioRunOptions {
                image_name: cached_image,
                env_vars,
                volumes: options.volumes,
                host_network: options.host_network,
            };
            
            return finch_client.run_stdio_container(&run_options, None).await;
        }
    }
    
    // Cache miss - need to build
    status!("🔨 Cache miss - building container...");
    
    // Initialize logging
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("local", &options.local_path)?;
    let build_start = std::time::Instant::now();
    
    status!("\n🔍 Analyzing project...");
    info!("Containerizing local directory: {}", local_path.display());
    
    // Detect the project type
    let project_info = detect_project_type(&local_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in directory"));
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.local_path);
    let image_name = cache_manager.generate_smart_image_name(
        "local",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Load finch-mcp config if present
    let finch_config = FinchConfig::load_from_dir(&local_path)?;
    if finch_config.is_some() {
        info!("Found .finch-mcp configuration file");
    }
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, finch_config.as_ref())?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Create build context and copy local directory contents
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy local directory files to build context
    copy_dir_all(&local_path, &build_context).context("Failed to copy local directory to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image with progress tracking
    let project_type_str = match project_info.project_type {
        ProjectType::NodeJs | ProjectType::NodeJsMonorepo => "Node.js",
        ProjectType::PythonPoetry => "Python (Poetry)",
        ProjectType::PythonUv => "Python (uv)",
        ProjectType::PythonSetupPy => "Python (setup.py)",
        ProjectType::PythonRequirements => "Python (requirements.txt)",
        ProjectType::Rust => "Rust",
        ProjectType::Unknown => "Unknown",
    };
    
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    // Add host network option if enabled
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command.arg(&build_context);
    
    // Log build command
    log_manager.append_to_log(&log_filename, &format!("Build command: {:?}", build_command))?;
    
    let build_result = run_build_with_progress(&mut build_command, &image_name, project_type_str);
    
    let build_duration = build_start.elapsed().as_secs();
    
    match &build_result {
        Ok(_) => {
            log_manager.append_to_log(&log_filename, "Build completed successfully")?;
            log_manager.finish_build_log(&log_filename, true, build_duration)?;
        }
        Err(e) => {
            log_manager.append_to_log(&log_filename, &format!("Build failed: {}", e))?;
            log_manager.finish_build_log(&log_filename, false, build_duration)?;
        }
    }
    
    build_result?;
    
    // Tag the image with 'latest' as well
    let base_name = image_name.split(':').next().unwrap_or(&image_name);
    let latest_tag = format!("{}:latest", base_name);
    
    let tag_command = Command::new("finch")
        .args(["tag", &image_name, &latest_tag])
        .status()
        .context("Failed to tag image with latest")?;
    
    if !tag_command.success() {
        log::warn!("Failed to tag image with latest");
    }
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &options.local_path,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    status!("💾 Image cached for future use");
    
    // Output MCP configuration
    output_mcp_config(&options.local_path, &image_name, &options.env_vars)?;
    
    // Prepare environment variables
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    // Add extra arguments if provided
    if !options.args.is_empty() {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
    // Run the container
    status!("🚀 Starting server...\n");
    info!("Running containerized local directory");
    let finch_client = FinchClient::new();
    let run_options = StdioRunOptions {
        image_name,
        env_vars,
        volumes: options.volumes,
        host_network: options.host_network,
    };
    
    finch_client.run_stdio_container(&run_options, Some(temp_dir.path())).await
}

/// Git containerize and run for MCP clients (build-then-run in one step)
pub async fn git_containerize_and_run_mcp(options: GitContainerizeOptions) -> Result<()> {
    use std::process::Stdio;
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the git repository
    let content_hash = content_hasher.hash_git_repository(&options.repo_url, None)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.repo_url, &content_hash, &build_options_hash).await {
        // Run the cached container directly in MCP mode (MCP env vars are added by finch client)
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
        
        return finch_client.run_stdio_container(&run_options, None).await;
    }
    
    // Build the image first (with suppressed output for MCP)
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("git-mcp", &options.repo_url)?;
    let build_start = std::time::Instant::now();
    
    // Parse and clone the repository
    let mut git_repo = GitRepository::new(&options.repo_url);
    let repo_path = git_repo.clone_to_temp_quiet(true).await?; // Always quiet for MCP
    
    // Detect the project type
    let project_info = detect_project_type(&repo_path)?;
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in repository"));
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.repo_url);
    let image_name = cache_manager.generate_smart_image_name(
        "git-mcp",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, None)?;
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    
    // Copy repository contents to build context
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    copy_dir_all(&repo_path, &build_context).context("Failed to copy repository to build context")?;
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
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
        .arg(&build_context);
    
    // Don't suppress output in MCP mode as it can interfere with stdio setup
    // Instead, let stderr show build progress while keeping stdout clean
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
        &options.repo_url,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    // Run the container directly
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    if !options.args.is_empty() {
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
    
    finch_client.run_stdio_container(&run_options, Some(temp_dir.path())).await
}

/// Local containerize and run for MCP clients (build-then-run in one step)
pub async fn local_containerize_and_run_mcp(options: LocalContainerizeOptions) -> Result<()> {
    use std::process::Stdio;
    
    // Debug: Log that we're using the MCP function
    log::info!("🚨 DEBUG: Using local_containerize_and_run_MCP function");
    
    let local_path = PathBuf::from(&options.local_path);
    
    // Validate that the path exists and is a directory
    if !local_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", options.local_path));
    }
    
    if !local_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", options.local_path));
    }
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the local directory
    let content_hash = content_hasher.hash_directory(&local_path)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.local_path, &content_hash, &build_options_hash).await {
        // Run the cached container directly in MCP mode (MCP env vars are added by finch client)
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
        
        return finch_client.run_stdio_container(&run_options, None).await;
    }
    
    // Build the image first (with suppressed output for MCP)
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("local-mcp", &options.local_path)?;
    let build_start = std::time::Instant::now();
    
    // Detect the project type
    let project_info = detect_project_type(&local_path)?;
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in directory"));
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.local_path);
    let image_name = cache_manager.generate_smart_image_name(
        "local-mcp",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Load finch-mcp config if present
    let finch_config = FinchConfig::load_from_dir(&local_path)?;
    if finch_config.is_some() {
        info!("Found .finch-mcp configuration file");
    }
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, finch_config.as_ref())?;
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    
    // Create build context and copy local directory contents
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    copy_dir_all(&local_path, &build_context).context("Failed to copy local directory to build context")?;
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
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
        .arg(&build_context);
    
    // Don't suppress output in MCP mode as it can interfere with stdio setup
    // Instead, let stderr show build progress while keeping stdout clean
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
        &options.local_path,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    // Run the container directly
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    if !options.args.is_empty() {
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
    
    finch_client.run_stdio_container(&run_options, Some(temp_dir.path())).await
}

fn get_registry_config(forward_registry: bool, project_type: &ProjectType) -> Vec<String> {
    if !forward_registry {
        return Vec::new();
    }
    
    let mut config_lines = Vec::new();
    
    match project_type {
        ProjectType::NodeJs | ProjectType::NodeJsMonorepo => {
            // Check for .npmrc in home directory
            if let Ok(home) = std::env::var("HOME") {
                let npmrc_path = format!("{}/.npmrc", home);
                if std::path::Path::new(&npmrc_path).exists() {
                    config_lines.push(format!("COPY --from=host {} /root/.npmrc", npmrc_path));
                }
            }
            
            // Forward common npm registry environment variables
            if let Ok(registry) = std::env::var("NPM_CONFIG_REGISTRY") {
                config_lines.push(format!("ENV NPM_CONFIG_REGISTRY={}", registry));
            }
            if let Ok(token) = std::env::var("NPM_TOKEN") {
                config_lines.push(format!("ENV NPM_TOKEN={}", token));
            }
        }
        
        ProjectType::PythonPoetry | ProjectType::PythonUv | 
        ProjectType::PythonSetupPy | ProjectType::PythonRequirements => {
            // Check for pip.conf
            if let Ok(home) = std::env::var("HOME") {
                let pip_conf_path = format!("{}/.pip/pip.conf", home);
                if std::path::Path::new(&pip_conf_path).exists() {
                    config_lines.push(format!("COPY --from=host {} /root/.pip/pip.conf", pip_conf_path));
                }
            }
            
            // Forward common pip registry environment variables
            if let Ok(index_url) = std::env::var("PIP_INDEX_URL") {
                config_lines.push(format!("ENV PIP_INDEX_URL={}", index_url));
            }
            if let Ok(extra_index_url) = std::env::var("PIP_EXTRA_INDEX_URL") {
                config_lines.push(format!("ENV PIP_EXTRA_INDEX_URL={}", extra_index_url));
            }
            if let Ok(trusted_host) = std::env::var("PIP_TRUSTED_HOST") {
                config_lines.push(format!("ENV PIP_TRUSTED_HOST={}", trusted_host));
            }
        }
        
        _ => {}
    }
    
    config_lines
}

/// Generate Dockerfile steps to modify package.json for selective dependency installation
fn generate_package_json_modification_steps(config: &FinchConfig, _package_manager: &str) -> Result<String> {
    if config.dependencies.install_all {
        // If installing all, don't modify
        return Ok(String::new());
    }
    
    let mut steps = Vec::new();
    
    steps.push("\n# Selectively install dependencies".to_string());
    steps.push("# Save original package.json".to_string());
    steps.push("RUN cp package.json package.json.original".to_string());
    
    // Create a Node.js script to modify package.json
    let script = format!(r#"
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));

// Keep track of what we're doing
console.log('Original devDependencies:', Object.keys(pkg.devDependencies || {{}}));

// Handle includes
const includes = {includes};
const skips = {skips};

// Helper to check if a dependency matches a pattern
function matchesPattern(dep, pattern) {{
    if (pattern.includes('*')) {{
        // Convert glob pattern to regex
        const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
        return regex.test(dep);
    }}
    return dep === pattern;
}}

if (pkg.devDependencies) {{
    const originalDevDeps = {{...pkg.devDependencies}};
    
    // If we have specific includes, keep only those
    if (includes.length > 0) {{
        pkg.devDependencies = {{}};
        for (const dep of includes) {{
            if (originalDevDeps[dep]) {{
                pkg.devDependencies[dep] = originalDevDeps[dep];
            }}
        }}
    }}
    
    // Remove any skips (supporting glob patterns)
    for (const pattern of skips) {{
        for (const dep of Object.keys(pkg.devDependencies)) {{
            if (matchesPattern(dep, pattern)) {{
                delete pkg.devDependencies[dep];
                console.log('Removing:', dep, '(matched pattern:', pattern, ')');
            }}
        }}
    }}
    
    console.log('Modified devDependencies:', Object.keys(pkg.devDependencies));
}}

fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2));
"#,
        includes = serde_json::to_string(&config.dependencies.include).unwrap_or_else(|_| "[]".to_string()),
        skips = serde_json::to_string(&config.dependencies.skip).unwrap_or_else(|_| "[]".to_string())
    );
    
    // Write the script to a file
    steps.push(format!(
        "RUN echo '{}' > /tmp/modify-deps.js",
        script.replace('\'', r"'\''").replace('\n', r"\n")
    ));
    
    // Run the script
    steps.push("RUN node /tmp/modify-deps.js".to_string());
    
    // Clean up
    steps.push("RUN rm /tmp/modify-deps.js".to_string());
    
    steps.push("".to_string()); // Empty line for readability
    
    Ok(steps.join("\n"))
}

fn generate_dockerfile_for_project(project_info: &ProjectInfo, args: &[String], forward_registry: bool, config: Option<&FinchConfig>) -> Result<String> {
    let registry_config = get_registry_config(forward_registry, &project_info.project_type);
    
    match project_info.project_type {
        ProjectType::PythonPoetry => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if let Some(ref entry_point) = project_info.entry_point {
                format!("poetry run {}", entry_point)
            } else if !args.is_empty() {
                format!("poetry run python {}", args.join(" "))
            } else {
                "poetry run python -m src".to_string()
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app
{registry_section}
# Install poetry
RUN pip install poetry

# Copy project files
COPY . .

# Configure poetry
RUN poetry config virtualenvs.create false

# Install dependencies
RUN poetry install

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command,
                registry_section = registry_section
            ))
        }
        
        ProjectType::PythonUv => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if let Some(ref entry_point) = project_info.entry_point {
                entry_point.clone()
            } else if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python -m src".to_string()
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app
{registry_section}
# Install uv
RUN pip install uv

# Copy project files
COPY . .

# Install dependencies
RUN uv pip install --system -e .

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command,
                registry_section = registry_section
            ))
        }
        
        ProjectType::PythonSetupPy => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python setup.py".to_string()
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app
{registry_section}
# Copy project files
COPY . .

# Install dependencies
RUN pip install -e .

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command,
                registry_section = registry_section
            ))
        }
        
        ProjectType::PythonRequirements => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python main.py".to_string()
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app
{registry_section}
# Copy project files
COPY . .

# Install dependencies
RUN pip install -r requirements.txt

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command,
                registry_section = registry_section
            ))
        }
        
        ProjectType::NodeJs => {
            let node_version = project_info.node_version.as_deref().unwrap_or("20");
            
            // Determine if this package has bin entries that need global installation
            let has_bin_command = project_info.bin_command.is_some();
            
            // Determine install command based on config
            let package_manager = project_info.package_manager.as_deref().unwrap_or("npm");
            let install_command = if let Some(cfg) = config {
                cfg.get_install_command(package_manager)
            } else {
                match package_manager {
                    "pnpm" => "pnpm install --prod",
                    "yarn" => "yarn install --production", 
                    _ => "npm install --production",
                }.to_string()
            };
            
            let entry_command = if let Some(ref run_cmd) = project_info.run_command {
                run_cmd.clone()
            } else if let Some(ref bin_cmd) = project_info.bin_command {
                // Use the bin command name directly
                bin_cmd.clone()
            } else if let Some(ref entry_point) = project_info.entry_point {
                format!("node {}", entry_point)
            } else if !args.is_empty() {
                format!("node {}", args.join(" "))
            } else {
                "npm start".to_string()
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            // Generate package.json modification steps if needed
            let package_json_steps = if let Some(cfg) = config {
                if !cfg.dependencies.install_all && 
                   (!cfg.dependencies.include.is_empty() || !cfg.dependencies.skip.is_empty()) {
                    // We need to selectively install dependencies
                    generate_package_json_modification_steps(cfg, package_manager)?
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            // Generate appropriate build and install steps
            let (build_steps, install_steps) = if has_bin_command {
                (
                    "# Build the package if needed\nRUN npm run build 2>/dev/null || echo \"No build script found, skipping...\"\n\n".to_string(),
                    "# Install the package globally to create bin symlinks\nRUN npm install -g .\n\n".to_string()
                )
            } else {
                ("".to_string(), "".to_string())
            };
            
            // Add pre-install commands if configured
            let pre_install_section = if let Some(cfg) = config {
                if !cfg.dependencies.pre_install.is_empty() {
                    format!("# Pre-install commands\n{}\n\n", 
                        cfg.dependencies.pre_install.iter()
                            .map(|cmd| format!("RUN {}", cmd))
                            .collect::<Vec<_>>()
                            .join("\n"))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            Ok(format!(
                r#"FROM node:{}-slim

WORKDIR /app
{}{}
# Copy project files
COPY . .
{}
# Install dependencies
RUN {}

{}{}# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                node_version,
                registry_section,
                pre_install_section,
                package_json_steps,
                install_command,
                build_steps,
                install_steps,
                entry_command
            ))
        }
        
        ProjectType::NodeJsMonorepo => {
            let node_version = project_info.node_version.as_deref().unwrap_or("20");
            let package_manager = project_info.package_manager.as_deref().unwrap_or("npm");
            
            let install_command = match package_manager {
                "pnpm" => "pnpm install --prod",
                "yarn" => "yarn install --production",
                _ => "npm install --production",
            };
            
            // Determine if this package has bin entries that need global installation
            let has_bin_command = project_info.bin_command.is_some();
            
            let entry_command = if let Some(ref run_cmd) = project_info.run_command {
                run_cmd.clone()
            } else if let Some(ref bin_cmd) = project_info.bin_command {
                // Use the bin command name directly
                bin_cmd.clone()
            } else if let Some(ref entry_point) = project_info.entry_point {
                format!("node {}", entry_point)
            } else if !args.is_empty() {
                format!("node {}", args.join(" "))
            } else {
                match package_manager {
                    "pnpm" => "pnpm start".to_string(),
                    "yarn" => "yarn start".to_string(),
                    _ => "npm start".to_string(),
                }
            };
            
            // For monorepo, we need to install the package manager first
            let pm_install = match package_manager {
                "pnpm" => "RUN npm install -g pnpm",
                "yarn" => "RUN npm install -g yarn", 
                _ => "",
            };
            
            let registry_section = if registry_config.is_empty() {
                String::new()
            } else {
                format!("\n# Registry configuration\n{}\n", registry_config.join("\n"))
            };
            
            // Generate appropriate build and install steps for monorepos
            let (build_steps, install_steps) = if has_bin_command {
                let build_cmd = match package_manager {
                    "pnpm" => "pnpm run build",
                    "yarn" => "yarn build",
                    _ => "npm run build",
                };
                let install_cmd = match package_manager {
                    "pnpm" => "pnpm install -g .",
                    "yarn" => "yarn global add file:.",
                    _ => "npm install -g .",
                };
                (
                    format!("# Build the package if needed\nRUN {} 2>/dev/null || echo \"No build script found, skipping...\"\n\n", build_cmd),
                    format!("# Install the package globally to create bin symlinks\nRUN {}\n\n", install_cmd)
                )
            } else {
                ("".to_string(), "".to_string())
            };
            
            Ok(format!(
                r#"FROM node:{}-slim

WORKDIR /app
{}
# Install package manager if needed
{}

# Copy project files
COPY . .

# Install dependencies
RUN {}

{}{}# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                node_version,
                registry_section,
                pm_install,
                install_command,
                build_steps,
                install_steps,
                entry_command
            ))
        }
        
        ProjectType::Rust => {
            Err(anyhow::anyhow!("Rust projects are not yet supported for git containerization"))
        }
        
        ProjectType::Unknown => {
            Err(anyhow::anyhow!("Unknown project type cannot be containerized"))
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        
        // Skip hidden files and directories, and common build/cache directories
        if let Some(file_name) = name.to_str() {
            if file_name.starts_with('.') 
                || file_name == "node_modules" 
                || file_name == "__pycache__" 
                || file_name == "target" 
                || file_name == "dist" 
                || file_name == "build" {
                continue;
            }
        }
        
        let dst_path = dst.join(&name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::project_detector::ProjectInfo;

    #[test]
    fn test_generate_dockerfile_python_poetry() {
        let project_info = ProjectInfo {
            project_type: ProjectType::PythonPoetry,
            name: Some("test-server".to_string()),
            entry_point: Some("test-server".to_string()),
            bin_command: None,
            install_command: Some("poetry install".to_string()),
            run_command: None,
            python_version: Some("3.11".to_string()),
            node_version: None,
            is_monorepo: false,
            package_manager: None,
        };
        
        let dockerfile = generate_dockerfile_for_project(&project_info, &[], false, None).unwrap();
        assert!(dockerfile.contains("FROM python:3.11-slim"));
        assert!(dockerfile.contains("RUN pip install poetry"));
        assert!(dockerfile.contains("poetry run test-server"));
    }

    #[test]
    fn test_generate_dockerfile_nodejs() {
        let project_info = ProjectInfo {
            project_type: ProjectType::NodeJs,
            name: Some("test-server".to_string()),
            entry_point: Some("index.js".to_string()),
            bin_command: None,
            install_command: Some("npm install".to_string()),
            run_command: None,
            python_version: None,
            node_version: Some("20".to_string()),
            is_monorepo: false,
            package_manager: None,
        };
        
        let dockerfile = generate_dockerfile_for_project(&project_info, &[], false, None).unwrap();
        assert!(dockerfile.contains("FROM node:20-slim"));
        assert!(dockerfile.contains("RUN npm install --production"));
        assert!(dockerfile.contains("node index.js"));
    }

    #[test]
    fn test_generate_dockerfile_nodejs_with_bin_command() {
        let project_info = ProjectInfo {
            project_type: ProjectType::NodeJs,
            name: Some("my-mcp-server".to_string()),
            entry_point: Some("./bin/server.js".to_string()),
            bin_command: Some("my-server".to_string()),
            install_command: Some("npm install".to_string()),
            run_command: None,
            python_version: None,
            node_version: Some("18".to_string()),
            is_monorepo: false,
            package_manager: None,
        };
        
        let dockerfile = generate_dockerfile_for_project(&project_info, &[], false, None).unwrap();
        assert!(dockerfile.contains("FROM node:18-slim"));
        assert!(dockerfile.contains("RUN npm install --production"));
        assert!(dockerfile.contains("npm run build"));
        assert!(dockerfile.contains("npm install -g ."));
        assert!(dockerfile.contains("my-server"));
        assert!(!dockerfile.contains("node ./bin/server.js")); // Should use bin command, not direct file
    }
}

/// Build a container from a git repository without running it
pub async fn git_build(options: GitContainerizeOptions) -> Result<String> {
    use console::style;
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the git repository
    let content_hash = content_hasher.hash_git_repository(&options.repo_url, None)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.repo_url, &content_hash, &build_options_hash).await {
        if options.force_rebuild {
            status!("🔨 Force rebuild requested, ignoring cached image: {}", style(&cached_image).cyan());
            info!("Force rebuild for git repository: {}", options.repo_url);
        } else {
            status!("⚡ Image already built: {}", style(&cached_image).cyan());
            status!("💡 To rebuild, use: {}", style("finch-mcp build --force <target>").yellow());
            info!("Cache hit for git repository: {}", options.repo_url);
            
            // Output MCP configuration
            output_mcp_config(&options.repo_url, &cached_image, &options.env_vars)?;
            
            return Ok(cached_image);
        }
    }
    
    // Cache miss or force rebuild - need to build
    status!("🔨 Building container...");
    
    // Initialize logging
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("git", &options.repo_url)?;
    let build_start = std::time::Instant::now();
    
    // Parse and clone the repository
    let mut git_repo = GitRepository::new(&options.repo_url);
    
    status!("\n🔄 Cloning repository...");
    info!("Cloning repository: {}", git_repo.url);
    let repo_path = git_repo.clone_to_temp_quiet(crate::output::is_quiet_mode()).await?;
    
    // Detect the project type
    let project_info = detect_project_type(&repo_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in repository"));
    }
    
    // Load finch-mcp config if present
    let finch_config = FinchConfig::load_from_dir(&repo_path)?;
    if finch_config.is_some() {
        info!("Found .finch-mcp configuration file");
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.repo_url);
    let image_name = cache_manager.generate_smart_image_name(
        "git",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, None)?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Copy repository contents to build context
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy repository files to build context
    copy_dir_all(&repo_path, &build_context).context("Failed to copy repository to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image with progress tracking
    let project_type_str = match project_info.project_type {
        ProjectType::NodeJs | ProjectType::NodeJsMonorepo => "Node.js",
        ProjectType::PythonPoetry => "Python (Poetry)",
        ProjectType::PythonUv => "Python (uv)",
        ProjectType::PythonSetupPy => "Python (setup.py)",
        ProjectType::PythonRequirements => "Python (requirements.txt)",
        ProjectType::Rust => "Rust",
        ProjectType::Unknown => "Unknown",
    };
    
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    // Add host network option if enabled
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command.arg(&build_context);
    
    // Log build command
    log_manager.append_to_log(&log_filename, &format!("Build command: {:?}", build_command))?;
    
    let build_result = run_build_with_progress(&mut build_command, &image_name, project_type_str);
    
    let build_duration = build_start.elapsed().as_secs();
    
    match &build_result {
        Ok(_) => {
            log_manager.append_to_log(&log_filename, "Build completed successfully")?;
            log_manager.finish_build_log(&log_filename, true, build_duration)?;
        }
        Err(e) => {
            log_manager.append_to_log(&log_filename, &format!("Build failed: {}", e))?;
            log_manager.finish_build_log(&log_filename, false, build_duration)?;
        }
    }
    
    build_result?;
    
    // Tag the image with 'latest' as well
    let base_name = image_name.split(':').next().unwrap_or(&image_name);
    let latest_tag = format!("{}:latest", base_name);
    
    let tag_command = Command::new("finch")
        .args(["tag", &image_name, &latest_tag])
        .status()
        .context("Failed to tag image with latest")?;
    
    if !tag_command.success() {
        log::warn!("Failed to tag image with latest");
    }
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &options.repo_url,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    status!("💾 Image cached for future use");
    
    // Output MCP configuration
    output_mcp_config(&options.repo_url, &image_name, &options.env_vars)?;
    
    Ok(image_name)
}

/// Build a container from a local directory without running it
pub async fn local_build(options: LocalContainerizeOptions) -> Result<String> {
    use console::style;
    
    let local_path = PathBuf::from(&options.local_path);
    
    // Validate that the path exists and is a directory
    if !local_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", options.local_path));
    }
    
    if !local_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", options.local_path));
    }
    
    // Initialize cache and content hasher
    let mut cache_manager = CacheManager::new()?;
    let content_hasher = ContentHasher::new();
    
    // Generate content hash for the local directory
    let content_hash = content_hasher.hash_directory(&local_path)?;
    let build_options_hash = hash_build_options(options.host_network, options.forward_registry, &options.env_vars);
    
    // Check if we have a cached image
    if let Some(cached_image) = cache_manager.get_cached_image(&options.local_path, &content_hash, &build_options_hash).await {
        if options.force_rebuild {
            status!("🔨 Force rebuild requested, ignoring cached image: {}", style(&cached_image).cyan());
            info!("Force rebuild for local directory: {}", options.local_path);
        } else {
            status!("⚡ Image already built: {}", style(&cached_image).cyan());
            status!("💡 To rebuild, use: {}", style("finch-mcp build --force <target>").yellow());
            info!("Cache hit for local directory: {}", options.local_path);
            
            // Output MCP configuration
            output_mcp_config(&options.local_path, &cached_image, &options.env_vars)?;
            
            return Ok(cached_image);
        }
    }
    
    // Cache miss or force rebuild - need to build
    status!("🔨 Building container...");
    
    // Initialize logging
    let log_manager = LogManager::new()?;
    let log_filename = log_manager.log_build_start("local", &options.local_path)?;
    let build_start = std::time::Instant::now();
    
    status!("\n🔍 Analyzing project...");
    info!("Containerizing local directory: {}", local_path.display());
    
    // Detect the project type
    let project_info = detect_project_type(&local_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in directory"));
    }
    
    // Generate smart, human-readable image name
    let identifier = CacheManager::extract_identifier(&options.local_path);
    let image_name = cache_manager.generate_smart_image_name(
        "local",
        &format!("{:?}", project_info.project_type),
        &identifier,
        &content_hash
    );
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Load finch-mcp config if present
    let finch_config = FinchConfig::load_from_dir(&local_path)?;
    if finch_config.is_some() {
        info!("Found .finch-mcp configuration file");
    }
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args, options.forward_registry, finch_config.as_ref())?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Create build context and copy local directory contents
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy local directory files to build context
    copy_dir_all(&local_path, &build_context).context("Failed to copy local directory to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image with progress tracking
    let project_type_str = match project_info.project_type {
        ProjectType::NodeJs | ProjectType::NodeJsMonorepo => "Node.js",
        ProjectType::PythonPoetry => "Python (Poetry)",
        ProjectType::PythonUv => "Python (uv)",
        ProjectType::PythonSetupPy => "Python (setup.py)",
        ProjectType::PythonRequirements => "Python (requirements.txt)",
        ProjectType::Rust => "Rust",
        ProjectType::Unknown => "Unknown",
    };
    
    let mut build_command = Command::new("finch");
    build_command
        .arg("build")
        .arg("-t")
        .arg(&image_name);
    
    // Add host network option if enabled
    if options.host_network {
        build_command.arg("--network").arg("host");
    }
    
    build_command.arg(&build_context);
    
    // Log build command
    log_manager.append_to_log(&log_filename, &format!("Build command: {:?}", build_command))?;
    
    let build_result = run_build_with_progress(&mut build_command, &image_name, project_type_str);
    
    let build_duration = build_start.elapsed().as_secs();
    
    match &build_result {
        Ok(_) => {
            log_manager.append_to_log(&log_filename, "Build completed successfully")?;
            log_manager.finish_build_log(&log_filename, true, build_duration)?;
        }
        Err(e) => {
            log_manager.append_to_log(&log_filename, &format!("Build failed: {}", e))?;
            log_manager.finish_build_log(&log_filename, false, build_duration)?;
        }
    }
    
    build_result?;
    
    // Tag the image with 'latest' as well
    let base_name = image_name.split(':').next().unwrap_or(&image_name);
    let latest_tag = format!("{}:latest", base_name);
    
    let tag_command = Command::new("finch")
        .args(["tag", &image_name, &latest_tag])
        .status()
        .context("Failed to tag image with latest")?;
    
    if !tag_command.success() {
        log::warn!("Failed to tag image with latest");
    }
    
    // Store in cache after successful build
    cache_manager.store_cache_entry(
        &options.local_path,
        &content_hash,
        &build_options_hash,
        &image_name,
        &format!("{:?}", project_info.project_type),
    )?;
    
    status!("💾 Image cached for future use");
    
    // Output MCP configuration
    output_mcp_config(&options.local_path, &image_name, &options.env_vars)?;
    
    Ok(image_name)
}

/// Output MCP configuration for MCP clients
fn output_mcp_config(source_path: &str, image_name: &str, env_vars: &[String]) -> Result<()> {
    use console::style;
    
    // Extract the server name from the path
    let server_name = CacheManager::extract_identifier(source_path)
        .to_lowercase()
        .replace('_', "-");
    
    // Parse environment variables into a map
    let mut env_map = serde_json::Map::new();
    for env_var in env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.insert(key.to_string(), json!(value));
        }
    }
    
    // Build the configuration object
    let config = json!({
        server_name: {
            "command": "finch-mcp",
            "args": [
                "run",
                image_name
            ],
            "env": env_map
        }
    });
    
    // Pretty print the configuration
    let config_str = serde_json::to_string_pretty(&config)?;
    
    println!("\n{} MCP Server Configuration:", style("📋").blue());
    println!("{}", style("Add this to your MCP client configuration:").dim());
    println!("{}", style("─".repeat(60)).dim());
    println!("{}", config_str);
    println!("{}", style("─".repeat(60)).dim());
    
    // Add helpful notes about environment variables and arguments
    println!("\n{} Configuration Notes:", style("💡").yellow());
    println!("• Environment variables: Check the MCP server's documentation for supported env vars");
    println!("• Server arguments: Pass additional args via EXTRA_ARGS environment variable");
    println!("  Example: \"env\": {{ \"EXTRA_ARGS\": \"--port 8080 --verbose\" }}");
    
    println!("\n{} Container image: {}", style("🐳").cyan(), style(image_name).green());
    println!("{} Latest tag: {}", style("🏷️").yellow(), style(format!("{}:latest", image_name.split(':').next().unwrap_or(image_name))).green());
    
    Ok(())
}