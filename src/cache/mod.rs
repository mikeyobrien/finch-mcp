use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod content_hasher;
pub use content_hasher::ContentHasher;

/// Cache entry for a built container image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Content hash of the source code/project
    pub content_hash: String,
    
    /// Docker image name/tag
    pub image_name: String,
    
    /// Timestamp when the image was built
    pub created_at: u64,
    
    /// Last time this cache entry was accessed
    pub last_accessed: u64,
    
    /// Project type (NodeJs, Python, etc.)
    pub project_type: String,
    
    /// Original source path or URL
    pub source_path: String,
    
    /// Build options hash (for different build configurations)
    pub build_options_hash: String,
}

/// Cache manager for finch-mcp container images
pub struct CacheManager {
    cache_file: PathBuf,
    entries: HashMap<String, CacheEntry>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
        
        let cache_file = cache_dir.join("finch-mcp-cache.json");
        
        let mut manager = Self {
            cache_file,
            entries: HashMap::new(),
        };
        
        manager.load_cache()?;
        Ok(manager)
    }
    
    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            Ok(PathBuf::from(xdg_cache).join("finch-mcp"))
        } else if let Ok(home) = std::env::var("HOME") {
            Ok(PathBuf::from(home).join(".cache").join("finch-mcp"))
        } else {
            // Windows fallback
            Ok(PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string()))
                .join("finch-mcp")
                .join("cache"))
        }
    }
    
    /// Load cache from disk
    fn load_cache(&mut self) -> Result<()> {
        if self.cache_file.exists() {
            let content = fs::read_to_string(&self.cache_file)
                .context("Failed to read cache file")?;
            self.entries = serde_json::from_str(&content)
                .context("Failed to parse cache file")?;
        }
        Ok(())
    }
    
    /// Save cache to disk
    pub fn save_cache(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize cache")?;
        fs::write(&self.cache_file, content)
            .context("Failed to write cache file")?;
        Ok(())
    }
    
    /// Generate a cache key for a project
    pub fn generate_cache_key(&self, source_path: &str, content_hash: &str, build_options_hash: &str) -> String {
        format!("{}:{}:{}", source_path, content_hash, build_options_hash)
    }
    
    /// Check if a cached image exists and is valid
    pub async fn get_cached_image(&mut self, source_path: &str, content_hash: &str, build_options_hash: &str) -> Option<String> {
        let cache_key = self.generate_cache_key(source_path, content_hash, build_options_hash);
        
        if let Some(entry) = self.entries.get(&cache_key) {
            let image_name = entry.image_name.clone();
            // Check if the image still exists in finch
            if self.image_exists(&image_name).await {
                // Update last accessed time
                if let Some(entry) = self.entries.get_mut(&cache_key) {
                    entry.last_accessed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                }
                
                return Some(image_name);
            } else {
                // Image no longer exists, remove from cache
                self.entries.remove(&cache_key);
            }
        }
        
        None
    }
    
    /// Store a new cache entry
    pub fn store_cache_entry(
        &mut self,
        source_path: &str,
        content_hash: &str,
        build_options_hash: &str,
        image_name: &str,
        project_type: &str,
    ) -> Result<()> {
        let cache_key = self.generate_cache_key(source_path, content_hash, build_options_hash);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let entry = CacheEntry {
            content_hash: content_hash.to_string(),
            image_name: image_name.to_string(),
            created_at: now,
            last_accessed: now,
            project_type: project_type.to_string(),
            source_path: source_path.to_string(),
            build_options_hash: build_options_hash.to_string(),
        };
        
        self.entries.insert(cache_key, entry);
        self.save_cache()?;
        Ok(())
    }
    
    /// Check if a finch image exists
    async fn image_exists(&self, image_name: &str) -> bool {
        use tokio::process::Command;
        
        let output = Command::new("finch")
            .args(["image", "inspect", image_name])
            .output()
            .await;
        
        matches!(output, Ok(output) if output.status.success())
    }
    
    /// Clean up old cache entries
    pub async fn cleanup_old_entries(&mut self, max_age_days: u64) -> Result<usize> {
        let max_age_secs = max_age_days * 24 * 60 * 60;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut removed_count = 0;
        let mut to_remove = Vec::new();
        
        for (key, entry) in &self.entries {
            let age = now.saturating_sub(entry.last_accessed);
            if age > max_age_secs || !self.image_exists(&entry.image_name).await {
                to_remove.push(key.clone());
            }
        }
        
        for key in to_remove {
            self.entries.remove(&key);
            removed_count += 1;
        }
        
        if removed_count > 0 {
            self.save_cache()?;
        }
        
        Ok(removed_count)
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let total_entries = self.entries.len();
        let mut project_types = HashMap::new();
        let mut total_size_estimate = 0u64;
        
        for entry in self.entries.values() {
            *project_types.entry(entry.project_type.clone()).or_insert(0) += 1;
            // Estimate ~100MB per container image
            total_size_estimate += 100 * 1024 * 1024;
        }
        
        CacheStats {
            total_entries,
            project_types,
            estimated_size_bytes: total_size_estimate,
        }
    }
    
    /// Clear all cache entries
    pub fn clear_cache(&mut self) -> Result<()> {
        self.entries.clear();
        self.save_cache()?;
        Ok(())
    }
    
    /// Generate a deterministic image name from content hash
    pub fn generate_cached_image_name(&self, content_hash: &str, project_type: &str) -> String {
        // Take first 12 characters of hash for readability
        let short_hash = &content_hash[..12.min(content_hash.len())];
        format!("mcp-cache-{}-{}", project_type.to_lowercase(), short_hash)
    }
    
    /// Generate a smart, human-readable image name with tag
    pub fn generate_smart_image_name(&self, 
        _source_type: &str, 
        _project_type: &str, 
        identifier: &str, 
        content_hash: &str
    ) -> String {
        // Take first 8 characters of hash as tag
        let tag = &content_hash[..8.min(content_hash.len())];
        
        // Sanitize identifier to be Docker-safe
        let clean_identifier = Self::sanitize_docker_name(identifier);
        
        // Simple name: mcp-{identifier}:{tag}
        format!("mcp-{}:{}", clean_identifier, tag)
    }
    
    /// Sanitize a string to be safe for Docker image names
    /// Docker image names must be lowercase and can only contain: a-z, 0-9, -, _, .
    fn sanitize_docker_name(name: &str) -> String {
        let result = name.to_lowercase()
            .chars()
            .map(|c| match c {
                'a'..='z' | '0'..='9' | '-' | '_' | '.' => c,
                '/' | '\\' | ':' => '-',
                ' ' => '-',
                _ => '_'
            })
            .collect::<String>();
        
        // Trim leading and trailing special characters
        result
            .trim_start_matches(&['-', '_', '.'])
            .trim_end_matches(&['-', '_', '.'])
            .to_string()
    }
    
    /// Extract meaningful identifier from source path/URL
    pub fn extract_identifier(source_path: &str) -> String {
        if source_path.contains("github.com") || source_path.contains("gitlab.com") || source_path.contains(".git") {
            // Git repository - extract repo name
            if let Some(repo_name) = source_path.split('/').last() {
                return repo_name.trim_end_matches(".git").to_string();
            }
        } else if source_path.starts_with('/') || source_path.contains("\\") {
            // Local path - extract directory name
            if let Some(dir_name) = source_path.split(['/', '\\']).last() {
                return dir_name.to_string();
            }
        } else {
            // Command - extract command name
            if let Some(cmd_name) = source_path.split_whitespace().next() {
                return cmd_name.to_string();
            }
        }
        
        // Fallback - use a portion of the source path
        source_path.chars().take(20).collect()
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to create cache manager")
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub project_types: HashMap<String, usize>,
    pub estimated_size_bytes: u64,
}

/// Generate hash of build options for cache key
pub fn hash_build_options(host_network: bool, forward_registry: bool, env_vars: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(host_network.to_string().as_bytes());
    hasher.update(forward_registry.to_string().as_bytes());
    for env_var in env_vars {
        hasher.update(env_var.as_bytes());
    }
    format!("{:x}", hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_cache_key() {
        let manager = CacheManager::new().unwrap();
        let key = manager.generate_cache_key("/path/to/project", "abc123", "def456");
        assert_eq!(key, "/path/to/project:abc123:def456");
    }
    
    #[test]
    fn test_hash_build_options() {
        let hash1 = hash_build_options(true, false, &[]);
        let hash2 = hash_build_options(false, true, &[]);
        let hash3 = hash_build_options(true, false, &[]);
        
        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }
    
    #[test]
    fn test_generate_cached_image_name() {
        let manager = CacheManager::new().unwrap();
        let name = manager.generate_cached_image_name("abcdef123456789", "nodejs");
        assert_eq!(name, "mcp-cache-nodejs-abcdef123456");
    }
    
    #[test]
    fn test_generate_smart_image_name() {
        let manager = CacheManager::new().unwrap();
        
        // Test git repository
        let name = manager.generate_smart_image_name("git", "NodeJs", "my-server", "abcdef123456");
        assert_eq!(name, "mcp-git-nodejs-my-server-abcdef12");
        
        // Test with special characters
        let name = manager.generate_smart_image_name("local", "Python", "My App/Server", "123456789abc");
        assert_eq!(name, "mcp-local-python-my-app-server-12345678");
        
        // Test auto command
        let name = manager.generate_smart_image_name("auto", "UVX", "time-server", "fedcba987654");
        assert_eq!(name, "mcp-auto-uvx-time-server-fedcba98");
    }
    
    #[test]
    fn test_sanitize_docker_name() {
        assert_eq!(CacheManager::sanitize_docker_name("My-App"), "my-app");
        assert_eq!(CacheManager::sanitize_docker_name("server/name"), "server-name");
        assert_eq!(CacheManager::sanitize_docker_name("app:version"), "app-version");
        assert_eq!(CacheManager::sanitize_docker_name("test_123"), "test_123");
        assert_eq!(CacheManager::sanitize_docker_name("_-special-_"), "special");
    }
    
    #[test]
    fn test_extract_identifier() {
        // Git URLs
        assert_eq!(CacheManager::extract_identifier("https://github.com/user/my-repo.git"), "my-repo");
        assert_eq!(CacheManager::extract_identifier("git@github.com:user/server.git"), "server");
        
        // Local paths
        assert_eq!(CacheManager::extract_identifier("/home/user/my-project"), "my-project");
        assert_eq!(CacheManager::extract_identifier("C:\\Users\\user\\app"), "app");
        
        // Commands
        assert_eq!(CacheManager::extract_identifier("uvx mcp-server-time"), "uvx");
        assert_eq!(CacheManager::extract_identifier("npx create-app"), "npx");
    }
}