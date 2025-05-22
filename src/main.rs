use finch_mcp::cli::{Cli, Commands, CacheCommands, LogCommands};
use finch_mcp::run::run_stdio_container;
use finch_mcp::core::auto_containerize::auto_containerize_and_run;
use finch_mcp::core::git_containerize::{git_containerize_and_run, local_containerize_and_run};
use finch_mcp::finch::client::FinchClient;
use finch_mcp::cache::CacheManager;
use finch_mcp::logging::LogManager;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args and initialize logging
    let cli = Cli::parse_and_init();
    
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
        
        Commands::Run { .. } => {
            // For direct container mode, skip banner and do minimal setup
            if cli.is_direct_container() {
                let finch_client = FinchClient::new();
                if !finch_client.is_finch_available().await? {
                    error!("Finch is not installed or not available");
                    eprintln!("\n❌ Error: Finch is required but not found");
                    eprintln!("📥 Please install Finch from: https://runfinch.com/");
                    std::process::exit(1);
                }
                run_target(&cli).await
            } else {
                // Non-direct mode - show banner and full setup
                println!("Finch-MCP v{}", env!("CARGO_PKG_VERSION"));
                println!("-------------------------------");
                
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

async fn run_target(cli: &Cli) -> anyhow::Result<()> {
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
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with git repository: {}", git_options.repo_url);
        
        // Run the git containerization process
        git_containerize_and_run(git_options).await.map_err(|err| {
            error!("Error running git-containerized MCP server: {}", err);
            err
        })?;
        info!("Git-containerized MCP server exited successfully");
        
    } else if cli.is_local_directory() {
        // Local directory mode - containerize and run from local path
        let local_options = cli.to_local_containerize_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with local directory: {}", local_options.local_path);
        
        // Run the local containerization process
        local_containerize_and_run(local_options).await.map_err(|err| {
            error!("Error running local-containerized MCP server: {}", err);
            err
        })?;
        info!("Local-containerized MCP server exited successfully");
        
    } else {
        // Auto-containerization mode
        let auto_options = cli.to_auto_containerize_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with auto-containerization: {} {}", 
             auto_options.command, auto_options.args.join(" "));
        
        // Run the auto-containerization process
        auto_containerize_and_run(auto_options).await.map_err(|err| {
            error!("Error running auto-containerized MCP server: {}", err);
            err
        })?;
        info!("Auto-containerized MCP server exited successfully");
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