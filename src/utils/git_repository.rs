use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;

/// Represents a Git repository URL and its metadata  
#[derive(Debug)]
pub struct GitRepository {
    pub url: String,
    pub branch: Option<String>,
    pub local_path: Option<PathBuf>,
    pub _temp_dir: Option<TempDir>, // Keep temp dir alive
}

impl GitRepository {
    /// Create a new GitRepository from a URL
    pub fn new(url: &str) -> Self {
        // Parse URL for branch information (e.g., url#branch)
        let (clean_url, branch) = if let Some((url_part, branch_part)) = url.split_once('#') {
            (url_part.to_string(), Some(branch_part.to_string()))
        } else {
            (url.to_string(), None)
        };

        Self {
            url: clean_url,
            branch,
            local_path: None,
            _temp_dir: None,
        }
    }

    /// Check if the given string looks like a Git repository URL
    pub fn is_git_url(input: &str) -> bool {
        input.starts_with("http://") 
            || input.starts_with("https://")
            || input.starts_with("git@")
            || input.starts_with("ssh://")
            || input.contains("github.com")
            || input.contains("gitlab.com")
            || input.contains("bitbucket.org")
            || input.ends_with(".git")
    }

    /// Clone the repository to a temporary directory
    pub async fn clone_to_temp(&mut self) -> Result<PathBuf> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let clone_path = temp_dir.path().join("repo");
        
        info!("Cloning repository {} to {:?}", self.url, clone_path);
        
        let mut cmd = Command::new("git");
        cmd.arg("clone")
           .arg("--depth").arg("1"); // Shallow clone for faster downloads
        
        // Add branch specification if provided
        if let Some(ref branch) = self.branch {
            cmd.arg("--branch").arg(branch);
        }
        
        cmd.arg(&self.url)
           .arg(&clone_path)
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
        
        debug!("Running git command: {:?}", cmd);
        
        let status = cmd.status().context("Failed to execute git clone command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Git clone failed with status: {}", status));
        }
        
        // Keep the temp directory alive by storing it
        self.local_path = Some(clone_path.clone());
        self._temp_dir = Some(temp_dir);
        
        Ok(clone_path)
    }

    /// Clone the repository to a temporary directory with optional quiet mode
    pub async fn clone_to_temp_quiet(&mut self, quiet: bool) -> Result<PathBuf> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let clone_path = temp_dir.path().join("repo");
        
        info!("Cloning repository {} to {:?}", self.url, clone_path);
        
        let mut cmd = Command::new("git");
        cmd.arg("clone")
           .arg("--depth").arg("1"); // Shallow clone for faster downloads
        
        // Add branch specification if provided
        if let Some(ref branch) = self.branch {
            cmd.arg("--branch").arg(branch);
        }
        
        cmd.arg(&self.url)
           .arg(&clone_path);
        
        // Redirect output based on quiet mode
        if quiet {
            cmd.stdout(Stdio::null())
               .stderr(Stdio::null());
        } else {
            cmd.stdout(Stdio::inherit())
               .stderr(Stdio::inherit());
        }
        
        debug!("Running git command: {:?}", cmd);
        
        let status = cmd.status().context("Failed to execute git clone command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Git clone failed with status: {}", status));
        }
        
        // Keep the temp directory alive by storing it
        self.local_path = Some(clone_path.clone());
        self._temp_dir = Some(temp_dir);
        
        Ok(clone_path)
    }

    /// Get the local path of the cloned repository
    pub fn local_path(&self) -> Option<&Path> {
        self.local_path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_url() {
        assert!(GitRepository::is_git_url("https://github.com/user/repo"));
        assert!(GitRepository::is_git_url("http://github.com/user/repo"));
        assert!(GitRepository::is_git_url("git@github.com:user/repo.git"));
        assert!(GitRepository::is_git_url("ssh://git@github.com/user/repo.git"));
        assert!(GitRepository::is_git_url("https://gitlab.com/user/repo"));
        assert!(GitRepository::is_git_url("https://example.com/repo.git"));
        
        assert!(!GitRepository::is_git_url("uvx mcp-server-time"));
        assert!(!GitRepository::is_git_url("npx @package/name"));
        assert!(!GitRepository::is_git_url("some-docker-image"));
    }

    #[test]
    fn test_new_with_branch() {
        let repo = GitRepository::new("https://github.com/user/repo#main");
        assert_eq!(repo.url, "https://github.com/user/repo");
        assert_eq!(repo.branch, Some("main".to_string()));
    }

    #[test]
    fn test_new_without_branch() {
        let repo = GitRepository::new("https://github.com/user/repo");
        assert_eq!(repo.url, "https://github.com/user/repo");
        assert_eq!(repo.branch, None);
    }
}