use clap::{Parser, ArgAction};
use log::debug;

use crate::run::RunOptions;
use crate::core::auto_containerize::AutoContainerizeOptions;

/// Finch-MCP STDIO - Tool for running MCP servers in STDIO mode with Finch
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(disable_version_flag(true))]
pub struct Cli {
    /// MCP server image or command to run
    #[arg(required = true)]
    pub command: String,
    
    /// Arguments for the command (when containerizing a command)
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
    
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
    
    /// Skip auto-containerization (treat the command as a Docker image directly)
    #[arg(long)]
    pub direct: bool,
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
    
    /// Convert CLI args to RunOptions (for direct container mode)
    pub fn to_run_options(&self) -> RunOptions {
        RunOptions {
            image_name: self.command.clone(),
            env_vars: self.env.clone(),
            volumes: self.volume.clone(),
        }
    }
    
    /// Convert CLI args to AutoContainerizeOptions
    pub fn to_auto_containerize_options(&self) -> AutoContainerizeOptions {
        AutoContainerizeOptions {
            command: self.command.clone(),
            args: self.args.clone(),
            env_vars: self.env.clone().unwrap_or_default(),
            volumes: self.volume.clone().unwrap_or_default(),
        }
    }
    
    /// Determine if we should use direct container mode or auto-containerization
    pub fn is_direct_container(&self) -> bool {
        self.direct || !self.command.contains(' ') && self.command.contains('/')
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
            command: "test-image:latest".to_string(),
            args: vec![],
            env: Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
            direct: true,
        };
        
        let run_options = cli.to_run_options();
        
        assert_eq!(run_options.image_name, "test-image:latest");
        assert_eq!(run_options.env_vars, Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]));
        assert_eq!(run_options.volumes, Some(vec!["/host:/container".to_string()]));
    }
    
    #[test]
    fn test_to_auto_containerize_options() {
        let cli = Cli {
            command: "uvx".to_string(),
            args: vec!["mcp-server-time".to_string()],
            env: Some(vec!["DEBUG=true".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
            direct: false,
        };
        
        let options = cli.to_auto_containerize_options();
        
        assert_eq!(options.command, "uvx");
        assert_eq!(options.args, vec!["mcp-server-time"]);
        assert_eq!(options.env_vars, vec!["DEBUG=true"]);
        assert_eq!(options.volumes, vec!["/host:/container"]);
    }
    
    #[test]
    fn test_is_direct_container() {
        // Direct flag overrides
        let cli1 = Cli {
            command: "uvx".to_string(),
            args: vec![],
            env: None,
            volume: None,
            verbose: 0,
            direct: true,
        };
        assert!(cli1.is_direct_container());
        
        // Docker-like image path
        let cli2 = Cli {
            command: "ghcr.io/user/image:tag".to_string(),
            args: vec![],
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
        };
        assert!(cli2.is_direct_container());
        
        // Regular command
        let cli3 = Cli {
            command: "uvx".to_string(),
            args: vec!["mcp-server-time".to_string()],
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
        };
        assert!(!cli3.is_direct_container());
    }
}