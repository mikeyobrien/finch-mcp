use finch_mcp::cli::{Cli, Commands};
use finch_mcp::run::run_stdio_container;
use finch_mcp::core::auto_containerize::auto_containerize_and_run;
use finch_mcp::core::git_containerize::{git_containerize_and_run, local_containerize_and_run};
use finch_mcp::finch::client::FinchClient;
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
        
        Commands::Run { .. } => {
            // Run command 
            
            // Print banner
            println!("Finch-MCP v{}", env!("CARGO_PKG_VERSION"));
            println!("-------------------------------");
            
            // Check if Finch is available early to provide helpful error messages
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