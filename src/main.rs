use finch_mcp_stdio::cli::Cli;
use finch_mcp_stdio::run::run_stdio_container;
use finch_mcp_stdio::core::auto_containerize::auto_containerize_and_run;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args and initialize logging
    let cli = Cli::parse_and_init();
    
    // Print banner
    println!("Finch-MCP STDIO v{}", env!("CARGO_PKG_VERSION"));
    println!("-------------------------------");
    
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