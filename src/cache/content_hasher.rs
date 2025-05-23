use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use log::debug;

/// Content hasher for projects to detect changes
pub struct ContentHasher {
    ignore_patterns: Vec<String>,
}

impl ContentHasher {
    /// Create a new content hasher with default ignore patterns
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                "target".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".cache".to_string(),
                "coverage".to_string(),
                ".nyc_output".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".tmp".to_string(),
                ".temp".to_string(),
            ],
        }
    }
    
    /// Hash the contents of a directory
    pub fn hash_directory(&self, dir_path: &Path) -> Result<String> {
        debug!("Hashing directory: {:?}", dir_path);
        
        let mut file_hashes = BTreeSet::new();
        self.collect_file_hashes(dir_path, &mut file_hashes)?;
        
        // Create final hash from sorted file hashes
        let mut hasher = Sha256::new();
        for file_hash in &file_hashes {
            hasher.update(file_hash.as_bytes());
        }
        
        let result = format!("{:x}", hasher.finalize());
        debug!("Directory hash: {} (from {} files)", result, file_hashes.len());
        Ok(result)
    }
    
    /// Hash content of a git repository URL
    pub fn hash_git_repository(&self, repo_url: &str, branch: Option<&str>) -> Result<String> {
        debug!("Hashing git repository: {}", repo_url);
        
        // For git repos, we use the URL + branch as the content identifier
        // In a real implementation, you might want to fetch the latest commit hash
        let mut hasher = Sha256::new();
        hasher.update(repo_url.as_bytes());
        if let Some(branch) = branch {
            hasher.update(b":");
            hasher.update(branch.as_bytes());
        }
        
        let result = format!("{:x}", hasher.finalize());
        debug!("Git repository hash: {}", result);
        Ok(result)
    }
    
    /// Hash a command for auto-containerization
    pub fn hash_command(&self, command: &str, args: &[String]) -> Result<String> {
        debug!("Hashing command: {} with args: {:?}", command, args);
        
        let mut hasher = Sha256::new();
        hasher.update(command.as_bytes());
        for arg in args {
            hasher.update(b":");
            hasher.update(arg.as_bytes());
        }
        
        let result = format!("{:x}", hasher.finalize());
        debug!("Command hash: {}", result);
        Ok(result)
    }
    
    /// Recursively collect file hashes from a directory
    fn collect_file_hashes(&self, dir_path: &Path, file_hashes: &mut BTreeSet<String>) -> Result<()> {
        let entries = fs::read_dir(dir_path)
            .with_context(|| format!("Failed to read directory: {:?}", dir_path))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            
            // Skip ignored files/directories
            if self.should_ignore(&file_name.to_string_lossy()) {
                debug!("Ignoring: {:?}", path);
                continue;
            }
            
            if path.is_file() {
                if let Ok(hash) = self.hash_file(&path) {
                    // Include relative path in hash to detect file moves
                    let relative_path = path.strip_prefix(dir_path)
                        .unwrap_or(&path)
                        .to_string_lossy();
                    let file_entry = format!("{}:{}", relative_path, hash);
                    file_hashes.insert(file_entry);
                }
            } else if path.is_dir() {
                self.collect_file_hashes(&path, file_hashes)?;
            }
        }
        
        Ok(())
    }
    
    /// Hash a single file
    fn hash_file(&self, file_path: &Path) -> Result<String> {
        let content = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize())[..16].to_string())
    }
    
    /// Check if a file/directory should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        for pattern in &self.ignore_patterns {
            if pattern.contains('*') {
                // Simple glob matching for patterns with *
                if pattern.starts_with("*.") {
                    let ext = &pattern[2..];
                    if name.ends_with(ext) {
                        return true;
                    }
                }
            } else if name == pattern {
                return true;
            }
        }
        false
    }
    
    /// Add custom ignore pattern
    pub fn add_ignore_pattern(&mut self, pattern: String) {
        self.ignore_patterns.push(pattern);
    }
    
    /// Set custom ignore patterns
    pub fn set_ignore_patterns(&mut self, patterns: Vec<String>) {
        self.ignore_patterns = patterns;
    }
}

impl Default for ContentHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_hash_command() {
        let hasher = ContentHasher::new();
        let hash1 = hasher.hash_command("uvx", &["mcp-server-time".to_string()]).unwrap();
        let hash2 = hasher.hash_command("uvx", &["mcp-server-time".to_string()]).unwrap();
        let hash3 = hasher.hash_command("uvx", &["different-package".to_string()]).unwrap();
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_hash_git_repository() {
        let hasher = ContentHasher::new();
        let hash1 = hasher.hash_git_repository("https://github.com/user/repo", None).unwrap();
        let hash2 = hasher.hash_git_repository("https://github.com/user/repo", Some("main")).unwrap();
        let hash3 = hasher.hash_git_repository("https://github.com/user/repo", Some("dev")).unwrap();
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
    }
    
    #[test]
    fn test_should_ignore() {
        let hasher = ContentHasher::new();
        
        assert!(hasher.should_ignore("node_modules"));
        assert!(hasher.should_ignore(".git"));
        assert!(hasher.should_ignore("test.log"));
        assert!(!hasher.should_ignore("src"));
        assert!(!hasher.should_ignore("package.json"));
    }
    
    #[test]
    fn test_hash_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("file1.txt"), "content1").unwrap();
        fs::write(temp_path.join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();
        fs::write(temp_path.join("subdir/file3.txt"), "content3").unwrap();
        
        let hasher = ContentHasher::new();
        let hash1 = hasher.hash_directory(temp_path).unwrap();
        
        // Hash should be consistent
        let hash2 = hasher.hash_directory(temp_path).unwrap();
        assert_eq!(hash1, hash2);
        
        // Adding a file should change the hash
        fs::write(temp_path.join("file4.txt"), "content4").unwrap();
        let hash3 = hasher.hash_directory(temp_path).unwrap();
        assert_ne!(hash1, hash3);
    }
}