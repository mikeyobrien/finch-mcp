use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

/// Configuration for finch-mcp containerization
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinchConfig {
    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,
    
    /// Runtime configuration
    #[serde(default)]
    pub runtime: RuntimeConfig,
    
    /// Dependencies configuration
    #[serde(default)]
    pub dependencies: DependenciesConfig,
    
    /// MCP-specific configuration
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildConfig {
    /// Custom build command (overrides auto-detection)
    pub command: Option<String>,
    
    /// Skip build step entirely
    #[serde(default)]
    pub skip: bool,
    
    /// Additional build arguments
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    /// Custom start command (overrides auto-detection)
    pub command: Option<String>,
    
    /// Working directory
    pub working_dir: Option<String>,
    
    /// Additional environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependenciesConfig {
    /// Install all dependencies (including devDependencies)
    #[serde(default)]
    pub install_all: bool,
    
    /// Auto-detect build dependencies (default: true)
    #[serde(default = "default_true")]
    pub auto_detect: bool,
    
    /// Additional dependencies to include beyond auto-detection
    #[serde(default)]
    pub include: Vec<String>,
    
    /// Dependencies to skip (overrides auto-detection)
    #[serde(default)]
    pub skip: Vec<String>,
    
    /// Custom install command
    pub install_command: Option<String>,
    
    /// Pre-install commands (e.g., for global tools)
    #[serde(default)]
    pub pre_install: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    /// Maximum time to wait for server startup (in seconds)
    #[serde(default = "default_startup_timeout")]
    pub startup_timeout: u64,
    
    /// Maximum buffer size for client messages (in bytes)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    /// Pattern to detect server readiness
    #[serde(default = "default_readiness_pattern")]
    pub readiness_pattern: String,
    
    /// Enable message buffering (default: true)
    #[serde(default = "default_true")]
    pub enable_buffering: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            startup_timeout: default_startup_timeout(),
            buffer_size: default_buffer_size(),
            readiness_pattern: default_readiness_pattern(),
            enable_buffering: true,
        }
    }
}

fn default_startup_timeout() -> u64 {
    30 // 30 seconds
}

fn default_buffer_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_readiness_pattern() -> String {
    "initialize".to_string()
}

impl FinchConfig {
    /// Load config from a directory
    pub fn load_from_dir(dir: &Path) -> Result<Option<Self>> {
        let config_path = dir.join(".finch-mcp");
        if !config_path.exists() {
            let yaml_path = dir.join(".finch-mcp.yaml");
            if !yaml_path.exists() {
                let yml_path = dir.join(".finch-mcp.yml");
                if !yml_path.exists() {
                    return Ok(None);
                }
                return Self::load_from_file(&yml_path).map(Some);
            }
            return Self::load_from_file(&yaml_path).map(Some);
        }
        Self::load_from_file(&config_path).map(Some)
    }
    
    /// Load config from a specific file
    fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        
        // Try to parse as YAML first (also handles JSON)
        if let Ok(config) = serde_yaml::from_str::<Self>(&content) {
            return Ok(config);
        }
        
        // Try to parse as TOML
        if let Ok(config) = toml::from_str::<Self>(&content) {
            return Ok(config);
        }
        
        Err(anyhow::anyhow!("Failed to parse .finch-mcp config file"))
    }
    
    /// Generate npm install command based on config
    pub fn get_install_command(&self, package_manager: &str) -> String {
        // If custom command specified, use it
        if let Some(ref custom_command) = self.dependencies.install_command {
            return custom_command.clone();
        }
        
        // If install_all is true, install everything
        if self.dependencies.install_all {
            return match package_manager {
                "pnpm" => "pnpm install".to_string(),
                "yarn" => "yarn install".to_string(),
                _ => "npm install".to_string(),
            };
        }
        
        // If auto-detect is disabled, use production install
        if !self.dependencies.auto_detect {
            return match package_manager {
                "pnpm" => "pnpm install --prod".to_string(),
                "yarn" => "yarn install --production".to_string(),
                _ => "npm install --production".to_string(),
            };
        }
        
        // If we have includes or skips, we'll modify package.json and then install
        // So we can use regular install (which will install modified deps)
        if !self.dependencies.include.is_empty() || !self.dependencies.skip.is_empty() {
            return match package_manager {
                "pnpm" => "pnpm install".to_string(),
                "yarn" => "yarn install".to_string(),
                _ => "npm install".to_string(),
            };
        }
        
        // Default to production install
        match package_manager {
            "pnpm" => "pnpm install --prod".to_string(),
            "yarn" => "yarn install --production".to_string(),
            _ => "npm install --production".to_string(),
        }
    }
    
    /// Check if we need build dependencies
    pub fn needs_build_dependencies(&self) -> bool {
        // If we have a build command or don't skip build, we likely need devDependencies
        !self.build.skip || self.dependencies.install_all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = FinchConfig::default();
        assert!(!config.build.skip);
        assert!(!config.dependencies.install_all);
        assert!(config.dependencies.include.is_empty());
    }
    
    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
dependencies:
  installAll: true
  include:
    - typescript
    - "@types/node"
build:
  command: "npm run custom-build"
"#;
        let config: FinchConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dependencies.install_all);
        assert_eq!(config.dependencies.include.len(), 2);
        assert_eq!(config.build.command, Some("npm run custom-build".to_string()));
    }
}