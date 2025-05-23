use finch_mcp::cli::{Cli, Commands, CacheCommands, LogCommands};
use finch_mcp::run::run_stdio_container;
use finch_mcp::core::auto_containerize::{auto_containerize_and_run, auto_build};
use finch_mcp::core::git_containerize::{git_containerize_and_run, local_containerize_and_run, git_build, local_build};
use finch_mcp::finch::client::FinchClient;
use finch_mcp::cache::CacheManager;
use finch_mcp::logging::LogManager;
use finch_mcp::status;
use log::{info, error};

fn check_cached_image_sync(cli: &Cli) -> Option<String> {
    use std::path::PathBuf;
    use finch_mcp::cache::ContentHasher;
    
    let target = cli.get_target();
    let local_path = PathBuf::from(target);
    
    // This is a simplified check - just look for the expected image name pattern
    let content_hasher = ContentHasher::new();
    if let Ok(content_hash) = content_hasher.hash_directory(&local_path) {
        // Extract directory name for the image
        let dir_name = local_path.file_name()?.to_str()?;
        let image_name = format!("mcp-local-nodejs-{}-{}", 
            dir_name.to_lowercase().replace(" ", ""),
            &content_hash[0..8]
        );
        
        // Check if this image exists by trying to run a quick finch command
        let output = std::process::Command::new("finch")
            .args(&["images", "-q", &image_name])
            .output()
            .ok()?;
            
        if output.status.success() && !output.stdout.is_empty() {
            Some(image_name)
        } else {
            None
        }
    } else {
        None
    }
}

fn main() -> anyhow::Result<()> {
    // Parse CLI args and initialize logging
    let cli = Cli::parse_and_init();
    
    // Special handling for MCP mode - exec immediately before async runtime
    if cli.is_mcp_client_context() && cli.is_local_directory() {
        if let Commands::Run { .. } = &cli.command {
            // Try to check for cached image synchronously
            if let Some(image_name) = check_cached_image_sync(&cli) {
                use std::os::unix::process::CommandExt;
                
                let mut cmd = std::process::Command::new("finch");
                cmd.arg("run")
                   .arg("--rm")
                   .arg("-i")
                   .arg("-e")
                   .arg("MCP_ENABLED=true")
                   .arg("-e")
                   .arg("MCP_STDIO=true");
                
                if let Some(env_vars) = &cli.env {
                    for env in env_vars {
                        cmd.arg("-e").arg(env);
                    }
                }
                
                if let Some(volumes) = &cli.volume {
                    for volume in volumes {
                        cmd.arg("-v").arg(volume);
                    }
                }
                
                if cli.host_network {
                    cmd.arg("--network").arg("host");
                }
                
                cmd.arg(&image_name);
                
                // Exec immediately before any async runtime
                let _ = cmd.exec();
                // If we get here, exec failed
                eprintln!("Failed to exec finch");
                std::process::exit(1);
            }
        }
    }
    
    // Run the async main
    tokio::runtime::Runtime::new()?.block_on(async_main(cli))
}

async fn async_main(cli: Cli) -> anyhow::Result<()> {
    
    // Handle subcommands
    match &cli.command {
        Commands::List { all } => {
            let finch_client = FinchClient::new();
            if !finch_client.is_finch_available().await? {
                error!("Finch is not installed or not available");
                eprintln!("\n❌ Error: Finch is required but not found");
                eprintln!("📥 Please install Finch from: https://runfinch.com/");
                std::process::exit(1);
            }
            
            finch_client.list_resources(*all).await?;
            Ok(())
        }
        
        Commands::Cleanup { all, containers, images, force } => {
            let finch_client = FinchClient::new();
            if !finch_client.is_finch_available().await? {
                error!("Finch is not installed or not available");
                eprintln!("\n❌ Error: Finch is required but not found");
                eprintln!("📥 Please install Finch from: https://runfinch.com/");
                std::process::exit(1);
            }
            
            finch_client.cleanup_resources(*all, *containers, *images, *force).await?;
            Ok(())
        }
        
        Commands::Cache { action } => {
            handle_cache_command(action).await?;
            Ok(())
        }
        
        Commands::Logs { action } => {
            handle_log_command(action).await?;
            Ok(())
        }
        
        Commands::Build { .. } => {
            build_target(&cli).await
        }
        
        Commands::Run { .. } => {
            // For direct container mode or MCP STDIO mode, skip banner and do minimal setup
            if cli.is_direct_container() || cli.is_mcp_client_context() {
                let finch_client = FinchClient::new();
                if !finch_client.is_finch_available().await? {
                    error!("Finch is not installed or not available");
                    eprintln!("\n❌ Error: Finch is required but not found");
                    eprintln!("📥 Please install Finch from: https://runfinch.com/");
                    std::process::exit(1);
                }
                run_target(&cli).await
            } else {
                // Non-direct, non-MCP mode - show banner and full setup
                status!("Finch-MCP v{}", env!("CARGO_PKG_VERSION"));
                status!("-------------------------------");
                
                let finch_client = FinchClient::new();
                if !finch_client.is_finch_available().await? {
                    error!("Finch is not installed or not available");
                    eprintln!("\n❌ Error: Finch is required but not found");
                    eprintln!("📥 Please install Finch from: https://runfinch.com/");
                    eprintln!("💡 Finch is a container runtime that enables finch-mcp to run MCP servers");
                    std::process::exit(1);
                }
                run_target(&cli).await
            }
        }
    }
}


async fn build_target(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Build { target: _, args: _ } => {
            // Determine the type of target
            if cli.is_git_repository() {
                // Git repository - clone and build
                let git_options = cli.to_git_containerize_options();
                let image_name = git_build(git_options).await?;
                status!("\n✅ Build complete: {}", image_name);
            } else if cli.is_local_directory() {
                // Local directory - build from local source
                let local_options = cli.to_local_containerize_options();
                let image_name = local_build(local_options).await?;
                status!("\n✅ Build complete: {}", image_name);
            } else {
                // Command - auto-containerize
                let auto_options = cli.to_auto_containerize_options();
                let image_name = auto_build(auto_options).await?;
                status!("\n✅ Build complete: {}", image_name);
            }
            Ok(())
        }
        _ => unreachable!()
    }
}

async fn run_target(cli: &Cli) -> anyhow::Result<()> {
    let is_mcp_context = cli.is_mcp_client_context();
    
    if cli.is_direct_container() {
        // Direct container mode - run existing container
        let run_options = cli.to_run_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with direct container: {}", run_options.image_name);
        
        // Run the container
        run_stdio_container(run_options).await.map_err(|err| {
            error!("Error running MCP server container: {}", err);
            err
        })?;
        info!("MCP server container exited successfully");
        
    } else if cli.is_git_repository() {
        // Git repository mode - clone, build, and run
        let git_options = cli.to_git_containerize_options();
        
        info!("Starting MCP server in STDIO mode with git repository: {}", git_options.repo_url);
        
        // Always use the regular function to avoid stdio issues
        // The MCP-specific functions have stdio handling that interferes with the protocol
        git_containerize_and_run(git_options).await?;
        
    } else if cli.is_local_directory() {
        // Local directory mode - containerize and run from local path
        let local_options = cli.to_local_containerize_options();
        
        // Don't log in MCP mode as it corrupts the protocol
        if !is_mcp_context {
            info!("Starting MCP server in STDIO mode with local directory: {}", local_options.local_path);
        }
        
        // Always use the regular function
        local_containerize_and_run(local_options).await?;
        
    } else {
        // Auto-containerization mode
        let auto_options = cli.to_auto_containerize_options();
        
        info!("Starting MCP server in STDIO mode with auto-containerization: {} {}", 
             auto_options.command, auto_options.args.join(" "));
        
        // Always use the regular function to avoid stdio issues
        // The MCP-specific functions have stdio handling that interferes with the protocol
        auto_containerize_and_run(auto_options).await?;
    }
    
    Ok(())
}

/// Handle cache-related commands
async fn handle_cache_command(action: &CacheCommands) -> anyhow::Result<()> {
    use console::style;
    
    match action {
        CacheCommands::Stats => {
            let cache_manager = CacheManager::new()?;
            let stats = cache_manager.get_stats();
            
            println!("\n{} Cache Statistics", style("📊").blue());
            println!("Total cached images: {}", style(stats.total_entries).cyan());
            println!("Estimated disk usage: {:.1} MB", style(stats.estimated_size_bytes as f64 / 1024.0 / 1024.0).yellow());
            
            if !stats.project_types.is_empty() {
                println!("\nCached images by type:");
                for (project_type, count) in stats.project_types {
                    println!("  {}: {}", style(&project_type).green(), style(count).cyan());
                }
            }
            
            if stats.total_entries == 0 {
                println!("{} No cached images found", style("ℹ️").blue());
                println!("Run some projects to build up the cache!");
            }
        }
        
        CacheCommands::Clear { force } => {
            let mut cache_manager = CacheManager::new()?;
            let stats = cache_manager.get_stats();
            
            if stats.total_entries == 0 {
                println!("{} Cache is already empty", style("✅").green());
                return Ok(());
            }
            
            if !force {
                println!("{} This will remove {} cached images", style("⚠️").yellow(), stats.total_entries);
                println!("Run with {} to proceed", style("--force").cyan());
                return Ok(());
            }
            
            cache_manager.clear_cache()?;
            println!("{} Cleared all {} cached images", style("🗑️").green(), stats.total_entries);
            println!("Note: Container images may still exist in Finch. Use {} to remove them.", style("finch-mcp cleanup").cyan());
        }
        
        CacheCommands::Cleanup { max_age } => {
            let mut cache_manager = CacheManager::new()?;
            let removed_count = cache_manager.cleanup_old_entries(*max_age).await?;
            
            if removed_count > 0 {
                println!("{} Cleaned up {} old cache entries", style("🧹").green(), removed_count);
            } else {
                println!("{} No old cache entries to clean up", style("✅").green());
            }
        }
    }
    
    Ok(())
}

/// Handle log-related commands
async fn handle_log_command(action: &LogCommands) -> anyhow::Result<()> {
    use console::style;
    
    match action {
        LogCommands::List { limit } => {
            let log_manager = LogManager::new()?;
            let logs = log_manager.list_recent_logs(*limit)?;
            
            if logs.is_empty() {
                println!("{} No build logs found", style("ℹ️").blue());
                println!("Build logs will appear here after container builds");
                return Ok(());
            }
            
            println!("\n{} Recent Build Logs", style("📄").blue());
            println!();
            
            for log_entry in logs {
                let time_str = log_entry.created_at.format("%Y-%m-%d %H:%M:%S UTC");
                println!("{} {} {} ({})", 
                    style("📁").blue(),
                    style(&log_entry.filename).cyan(),
                    style(&log_entry.operation_type).green(),
                    style(time_str).dim()
                );
                println!("   {}", style(&log_entry.identifier).dim());
            }
            
            println!();
            println!("Use {} to view a specific log", style("finch-mcp logs show <filename>").cyan());
        }
        
        LogCommands::Show { filename } => {
            let log_manager = LogManager::new()?;
            let log_path = log_manager.get_logs_directory_path().join(filename);
            
            if !log_path.exists() {
                eprintln!("{} Log file not found: {}", style("❌").red(), filename);
                eprintln!("Use {} to see available logs", style("finch-mcp logs list").cyan());
                return Ok(());
            }
            
            let content = std::fs::read_to_string(&log_path)?;
            println!("{}", content);
        }
        
        LogCommands::Cleanup { max_age } => {
            let log_manager = LogManager::new()?;
            let removed_count = log_manager.cleanup_old_logs(*max_age)?;
            
            if removed_count > 0 {
                println!("{} Cleaned up {} old log files", style("🧹").green(), removed_count);
            } else {
                println!("{} No old log files to clean up", style("✅").green());
            }
        }
        
        LogCommands::Path => {
            let log_manager = LogManager::new()?;
            let log_dir = log_manager.get_logs_directory_path();
            println!("{}", log_dir.display());
        }
    }
    
    Ok(())
}