use finch_mcp_stdio::cli::Cli;
use finch_mcp_stdio::run::run_stdio_container;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args and initialize logging
    let cli = Cli::parse_and_init();
    
    // Print banner
    println!("Finch-MCP STDIO v{}", env!("CARGO_PKG_VERSION"));
    println!("-------------------------------");
    
    // Convert CLI args to run options
    let run_options = cli.to_run_options();
    
    // Log the start of execution
    info!("Starting MCP server in STDIO mode: {}", run_options.image_name);
    
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
}