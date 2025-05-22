use clap::{Parser, Subcommand, ArgAction};
use log::debug;
use std::path::Path;

use crate::run::RunOptions;
use crate::core::auto_containerize::AutoContainerizeOptions;
use crate::core::git_containerize::{GitContainerizeOptions, LocalContainerizeOptions};
use crate::utils::git_repository::GitRepository;

/// Finch-MCP - Tool for running MCP servers using Finch containers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(disable_version_flag(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Environment variables to pass to the container
    /// Format: KEY=VALUE
    #[arg(short, long, value_name = "KEY=VALUE", global = true)]
    pub env: Option<Vec<String>>,
    
    /// Mount volumes in the container
    /// Format: /host/path:/container/path
    #[arg(short, long, value_name = "HOST_PATH:CONTAINER_PATH", global = true)]
    pub volume: Option<Vec<String>>,
    
    /// Enable verbose logging
    #[arg(short = 'V', long, action = ArgAction::Count, global = true)]
    pub verbose: u8,
    
    /// Skip auto-containerization (treat the command as a Docker image directly)
    #[arg(long, global = true)]
    pub direct: bool,
    
    /// Use host network for package registry access
    #[arg(long, global = true)]
    pub host_network: bool,
    
    /// Forward registry configuration from host
    /// Supports: npmrc, pip.conf, poetry config, requirements.txt with --index-url
    #[arg(long, global = true)]
    pub forward_registry: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run an MCP server
    Run {
        /// MCP server image, command, git repository URL, or local directory to run
        target: String,
        
        /// Arguments for the command (when containerizing a command)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List finch-mcp containers and images
    List {
        /// Show all containers (including stopped ones)
        #[arg(short, long)]
        all: bool,
    },
    /// Clean up finch-mcp containers and images
    Cleanup {
        /// Remove all finch-mcp containers and images
        #[arg(short, long)]
        all: bool,
        
        /// Remove only stopped containers
        #[arg(short, long)]
        containers: bool,
        
        /// Remove only unused images
        #[arg(short, long)]
        images: bool,
        
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Manage build cache
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },
    
    /// Manage build logs
    Logs {
        #[command(subcommand)]
        action: LogCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Show cache statistics
    Stats,
    
    /// Clear all cached images
    Clear {
        /// Force clearing without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Clean up old cache entries
    Cleanup {
        /// Maximum age in days for cache entries (default: 7)
        #[arg(short, long, default_value = "7")]
        max_age: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    /// List recent build logs
    List {
        /// Number of logs to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Show contents of a specific log file
    Show {
        /// Log filename to display
        filename: String,
    },
    
    /// Clean up old log files
    Cleanup {
        /// Maximum age in days for log files (default: 30)
        #[arg(short, long, default_value = "30")]
        max_age: u32,
    },
    
    /// Show logs directory path
    Path,
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
    
    /// Get the target string (for run operations)
    pub fn get_target(&self) -> &str {
        match &self.command {
            Commands::Run { target, .. } => target,
            _ => unreachable!("Only run command should call this"),
        }
    }
    
    /// Get the args (for run operations)  
    pub fn get_args(&self) -> &[String] {
        match &self.command {
            Commands::Run { args, .. } => args,
            _ => unreachable!("Only run command should call this"),
        }
    }
    
    /// Convert CLI args to RunOptions (for direct container mode)
    pub fn to_run_options(&self) -> RunOptions {
        RunOptions {
            image_name: self.get_target().to_string(),
            env_vars: self.env.clone(),
            volumes: self.volume.clone(),
        }
    }
    
    /// Convert CLI args to AutoContainerizeOptions
    pub fn to_auto_containerize_options(&self) -> AutoContainerizeOptions {
        let target = self.get_target();
        let args = self.get_args();
        
        // Check if the command looks like a quoted command string
        // (contains spaces and common command patterns)
        if args.is_empty() && (
            target.contains(" -") || 
            target.contains(" @") || 
            target.starts_with("npx ") ||
            target.starts_with("uvx ")
        ) {
            // Parse as a quoted command string
            let (parsed_command, parsed_args) = crate::utils::command_parser::parse_command_string(target);
            AutoContainerizeOptions {
                command: parsed_command,
                args: parsed_args,
                env_vars: self.env.clone().unwrap_or_default(),
                volumes: self.volume.clone().unwrap_or_default(),
                host_network: self.host_network,
                forward_registry: self.forward_registry,
            }
        } else {
            // Use as separate command and args
            AutoContainerizeOptions {
                command: target.to_string(),
                args: args.to_vec(),
                env_vars: self.env.clone().unwrap_or_default(),
                volumes: self.volume.clone().unwrap_or_default(),
                host_network: self.host_network,
                forward_registry: self.forward_registry,
            }
        }
    }
    
    /// Convert CLI args to GitContainerizeOptions
    pub fn to_git_containerize_options(&self) -> GitContainerizeOptions {
        GitContainerizeOptions {
            repo_url: self.get_target().to_string(),
            args: self.get_args().to_vec(),
            env_vars: self.env.clone().unwrap_or_default(),
            volumes: self.volume.clone().unwrap_or_default(),
            host_network: self.host_network,
            forward_registry: self.forward_registry,
        }
    }
    
    /// Convert CLI args to LocalContainerizeOptions
    pub fn to_local_containerize_options(&self) -> LocalContainerizeOptions {
        LocalContainerizeOptions {
            local_path: self.get_target().to_string(),
            args: self.get_args().to_vec(),
            env_vars: self.env.clone().unwrap_or_default(),
            volumes: self.volume.clone().unwrap_or_default(),
            host_network: self.host_network,
            forward_registry: self.forward_registry,
        }
    }
    
    /// Determine if we should use direct container mode or auto-containerization
    pub fn is_direct_container(&self) -> bool {
        let target = self.get_target();
        
        // Explicit --direct flag
        if self.direct {
            return true;
        }
        
        // Auto-detect container image patterns (but not for MCP clients)
        if Self::looks_like_container_image(target) {
            return true;
        }
        
        // Fallback to existing logic
        !target.contains(' ') && target.contains('/') && !GitRepository::is_git_url(target) && !self.is_local_directory()
    }
    
    /// Check if we're running in an MCP client context
    pub fn is_mcp_client_context(&self) -> bool {
        Self::is_mcp_client_context_static()
    }
    
    /// Static version for use in auto-detection
    fn is_mcp_client_context_static() -> bool {
        // MCP_STDIO environment variable (set by MCP clients)
        if std::env::var("MCP_STDIO").is_ok() {
            return true;
        }
        
        // Check if parent process looks like an MCP client
        if let Ok(parent) = std::env::var("_") {
            if parent.contains("claude") || parent.contains("mcp") {
                return true;
            }
        }
        
        // Check for common MCP client environment indicators
        std::env::var("MCP_CLIENT").is_ok() || 
        std::env::var("CLAUDE_DESKTOP").is_ok()
    }
    
    /// Check if the target looks like a container image
    fn looks_like_container_image(target: &str) -> bool {
        // Standard Docker image patterns
        if target.contains(':') && !target.starts_with("http") && !target.contains(' ') {
            // registry.com/namespace/image:tag
            if target.matches('/').count() >= 1 && target.contains(':') {
                return true;
            }
            
            // image:tag (simple case)
            if !target.contains('/') && target.contains(':') && target.split(':').count() == 2 {
                let parts: Vec<&str> = target.split(':').collect();
                // Ensure tag doesn't look like a port number or URL
                if !parts[1].chars().all(|c| c.is_ascii_digit()) && !parts[1].contains('.') {
                    return true;
                }
            }
        }
        
        // Registry patterns (no tag, implies :latest)
        if target.contains('/') && !target.contains(' ') && !target.contains(':') {
            // Skip obvious file paths
            if !target.starts_with('.') && !target.starts_with('/') && !target.starts_with('~') {
                return true;
            }
        }
        
        false
    }
    
    /// Determine if the command is a git repository URL
    pub fn is_git_repository(&self) -> bool {
        let target = self.get_target();
        GitRepository::is_git_url(target)
    }
    
    /// Determine if the command is a local directory path
    pub fn is_local_directory(&self) -> bool {
        let target = self.get_target();
        let path = Path::new(target);
        path.exists() && path.is_dir()
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
            command: Commands::Run {
                target: "test-image:latest".to_string(),
                args: vec![],
            },
            env: Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
            direct: true,
            host_network: false,
            forward_registry: false,
        };
        
        let run_options = cli.to_run_options();
        
        assert_eq!(run_options.image_name, "test-image:latest");
        assert_eq!(run_options.env_vars, Some(vec!["KEY=VALUE".to_string(), "DEBUG=true".to_string()]));
        assert_eq!(run_options.volumes, Some(vec!["/host:/container".to_string()]));
    }
    
    #[test]
    fn test_to_auto_containerize_options() {
        let cli = Cli {
            command: Commands::Run {
                target: "uvx".to_string(),
                args: vec!["mcp-server-time".to_string()],
            },
            env: Some(vec!["DEBUG=true".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
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
            command: Commands::Run {
                target: "uvx".to_string(),
                args: vec![],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: true,
            host_network: false,
            forward_registry: false,
        };
        assert!(cli1.is_direct_container());
        
        // Docker-like image path
        let cli2 = Cli {
            command: Commands::Run {
                target: "ghcr.io/user/image:tag".to_string(),
                args: vec![],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        assert!(cli2.is_direct_container());
        
        // Regular command
        let cli3 = Cli {
            command: Commands::Run {
                target: "uvx".to_string(),
                args: vec!["mcp-server-time".to_string()],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        assert!(!cli3.is_direct_container());
    }
    
    #[test]
    fn test_is_local_directory() {
        // Test with existing directory (current directory should exist)
        let cli1 = Cli {
            command: Commands::Run {
                target: ".".to_string(),
                args: vec![],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        assert!(cli1.is_local_directory());
        
        // Test with non-existent directory
        let cli2 = Cli {
            command: Commands::Run {
                target: "./non-existent-dir-12345".to_string(),
                args: vec![],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        assert!(!cli2.is_local_directory());
        
        // Test with regular command
        let cli3 = Cli {
            command: Commands::Run {
                target: "uvx".to_string(),
                args: vec![],
            },
            env: None,
            volume: None,
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        assert!(!cli3.is_local_directory());
    }
    
    #[test]
    fn test_to_local_containerize_options() {
        let cli = Cli {
            command: Commands::Run {
                target: "./test-dir".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
            },
            env: Some(vec!["KEY=VALUE".to_string()]),
            volume: Some(vec!["/host:/container".to_string()]),
            verbose: 0,
            direct: false,
            host_network: false,
            forward_registry: false,
        };
        
        let options = cli.to_local_containerize_options();
        
        assert_eq!(options.local_path, "./test-dir");
        assert_eq!(options.args, vec!["arg1", "arg2"]);
        assert_eq!(options.env_vars, vec!["KEY=VALUE"]);
        assert_eq!(options.volumes, vec!["/host:/container"]);
    }
}