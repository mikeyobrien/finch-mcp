use clap::{Parser, ArgAction};
use log::debug;

use crate::run::RunOptions;

/// Finch-MCP STDIO - Tool for running MCP servers in STDIO mode with Finch
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(disable_version_flag(true))]
pub struct Cli {
    /// MCP server image to run
    #[arg(required = true)]
    pub image: String,
    
    /// Environment variables to pass to the container
    /// Format: KEY=VALUE
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub env: Option<Vec<String>>,
    
    /// Mount volumes in the container
    /// Format: /host/path:/container/path
    #[arg(short, long, value_name = "HOST_PATH:CONTAINER_PATH")]
    pub volume: Option<Vec<String>>,
    
    /// Enable verbose logging
    #[arg(short = 'V', long, action = ArgAction::Count)]
    pub verbose: u8,
}

impl Cli {
    /// Parse CLI arguments and initialize logging
    pub fn parse_and_init() -> Self {
        let cli = Self::parse();
        
        // Initialize logging based on verbosity
        let log_level = match cli.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        
        env_logger::Builder::new()
            .filter_level(log_level)
            .format_timestamp(None)
            .init();
            
        debug!("CLI arguments: {:?}", cli);
        
        cli
    }
    
    /// Convert CLI args to RunOptions
    pub fn to_run_options(&self) -> RunOptions {
        RunOptions {
            image_name: self.image.clone(),
            env_vars: self.env.clone(),
            volumes: self.volume.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
    
    #[test]
    fn test_to_run_options() {
        let cli = Cli {
            image: "test-image:latest".to_string(),
            env: Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
        };
        
        let run_options = cli.to_run_options();
        
        assert_eq!(run_options.image_name, "test-image:latest");
        assert_eq!(run_options.env_vars, Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]));
        assert_eq!(run_options.volumes, Some(vec!["/host:/container".to_string()]));
    }
}