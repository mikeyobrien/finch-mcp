use finch_mcp_stdio::cli::Cli;
use finch_mcp_stdio::run::run_stdio_container;
use finch_mcp_stdio::core::auto_containerize::auto_containerize_and_run;
use finch_mcp_stdio::core::git_containerize::{git_containerize_and_run, local_containerize_and_run};
use finch_mcp_stdio::finch::client::FinchClient;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args and initialize logging
    let cli = Cli::parse_and_init();
    
    // Print banner
    println!("Finch-MCP STDIO v{}", env!("CARGO_PKG_VERSION"));
    println!("-------------------------------");
    
    // Check if Finch is available early to provide helpful error messages
    let finch_client = FinchClient::new();
    if !finch_client.is_finch_available().await? {
        error!("Finch is not installed or not available");
        eprintln!("\n❌ Error: Finch is required but not found");
        eprintln!("📥 Please install Finch from: https://runfinch.com/");
        eprintln!("💡 Finch is a container runtime that enables finch-mcp-stdio to run MCP servers");
        std::process::exit(1);
    }
    
    if cli.is_direct_container() {
        // Direct container mode - run existing container
        let run_options = cli.to_run_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with direct container: {}", run_options.image_name);
        
        // Run the container
        match run_stdio_container(run_options).await {
            Ok(_) => {
                info!("MCP server container exited successfully");
                Ok(())
            }
            Err(err) => {
                error!("Error running MCP server container: {}", err);
                Err(err)
            }
        }
    } else if cli.is_git_repository() {
        // Git repository mode - clone, build, and run
        let git_options = cli.to_git_containerize_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with git repository: {}", git_options.repo_url);
        
        // Run the git containerization process
        match git_containerize_and_run(git_options).await {
            Ok(_) => {
                info!("Git-containerized MCP server exited successfully");
                Ok(())
            }
            Err(err) => {
                error!("Error running git-containerized MCP server: {}", err);
                Err(err)
            }
        }
    } else if cli.is_local_directory() {
        // Local directory mode - containerize and run from local path
        let local_options = cli.to_local_containerize_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with local directory: {}", local_options.local_path);
        
        // Run the local containerization process
        match local_containerize_and_run(local_options).await {
            Ok(_) => {
                info!("Local-containerized MCP server exited successfully");
                Ok(())
            }
            Err(err) => {
                error!("Error running local-containerized MCP server: {}", err);
                Err(err)
            }
        }
    } else {
        // Auto-containerization mode
        let auto_options = cli.to_auto_containerize_options();
        
        // Log the start of execution
        info!("Starting MCP server in STDIO mode with auto-containerization: {} {}", 
             auto_options.command, auto_options.args.join(" "));
        
        // Run the auto-containerization process
        match auto_containerize_and_run(auto_options).await {
            Ok(_) => {
                info!("Auto-containerized MCP server exited successfully");
                Ok(())
            }
            Err(err) => {
                error!("Error running auto-containerized MCP server: {}", err);
                Err(err)
            }
        }
    }
}