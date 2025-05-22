use std::path::Path;
use std::fs;
use anyhow::{Context, Result};
use log::debug;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    PythonPoetry,     // pyproject.toml with poetry
    PythonSetupPy,    // setup.py
    PythonRequirements, // requirements.txt
    PythonUv,         // pyproject.toml with uv
    NodeJs,           // package.json
    NodeJsMonorepo,   // package.json with workspaces (pnpm/npm)
    Rust,             // Cargo.toml
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub name: Option<String>,
    pub entry_point: Option<String>,
    pub install_command: Option<String>,
    pub run_command: Option<String>,
    pub python_version: Option<String>,
    pub node_version: Option<String>,
    pub is_monorepo: bool,
    pub package_manager: Option<String>,
}

pub fn detect_project_type(repo_path: &Path) -> Result<ProjectInfo> {
    debug!("Detecting project type in: {:?}", repo_path);
    
    // Check for Python projects first
    if let Some(info) = detect_python_project(repo_path)? {
        return Ok(info);
    }
    
    // Check for Node.js projects
    if let Some(info) = detect_nodejs_project(repo_path)? {
        return Ok(info);
    }
    
    // Check for Rust projects
    if let Some(info) = detect_rust_project(repo_path)? {
        return Ok(info);
    }
    
    // Default to unknown
    Ok(ProjectInfo {
        project_type: ProjectType::Unknown,
        name: None,
        entry_point: None,
        install_command: None,
        run_command: None,
        python_version: None,
        node_version: None,
        is_monorepo: false,
        package_manager: None,
    })
}

fn detect_python_project(repo_path: &Path) -> Result<Option<ProjectInfo>> {
    let pyproject_path = repo_path.join("pyproject.toml");
    let setup_py_path = repo_path.join("setup.py");
    let requirements_path = repo_path.join("requirements.txt");
    
    // Check for pyproject.toml (modern Python projects)
    if pyproject_path.exists() {
        debug!("Found pyproject.toml");
        let content = fs::read_to_string(&pyproject_path)
            .context("Failed to read pyproject.toml")?;
        
        let info = parse_pyproject_toml(&content)?;
        return Ok(Some(info));
    }
    
    // Check for setup.py (legacy Python projects)
    if setup_py_path.exists() {
        debug!("Found setup.py");
        return Ok(Some(ProjectInfo {
            project_type: ProjectType::PythonSetupPy,
            name: extract_setup_py_name(repo_path)?,
            entry_point: None,
            install_command: Some("pip install -e .".to_string()),
            run_command: None,
            python_version: Some("3.11".to_string()),
            node_version: None,
            is_monorepo: false,
            package_manager: None,
        }));
    }
    
    // Check for requirements.txt
    if requirements_path.exists() {
        debug!("Found requirements.txt");
        return Ok(Some(ProjectInfo {
            project_type: ProjectType::PythonRequirements,
            name: None,
            entry_point: None,
            install_command: Some("pip install -r requirements.txt".to_string()),
            run_command: None,
            python_version: Some("3.11".to_string()),
            node_version: None,
            is_monorepo: false,
            package_manager: None,
        }));
    }
    
    Ok(None)
}

fn detect_nodejs_project(repo_path: &Path) -> Result<Option<ProjectInfo>> {
    let package_json_path = repo_path.join("package.json");
    
    if package_json_path.exists() {
        debug!("Found package.json");
        let content = fs::read_to_string(&package_json_path)
            .context("Failed to read package.json")?;
        
        let package_json: Value = serde_json::from_str(&content)
            .context("Failed to parse package.json")?;
        
        let name = package_json.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Check for monorepo indicators
        let is_monorepo = detect_nodejs_monorepo(repo_path, &package_json)?;
        let (project_type, package_manager, install_command) = if is_monorepo {
            let pm = detect_package_manager(repo_path)?;
            let install_cmd = match pm.as_deref() {
                Some("pnpm") => "pnpm install".to_string(),
                Some("yarn") => "yarn install".to_string(),
                _ => "npm install".to_string(),
            };
            (ProjectType::NodeJsMonorepo, pm, install_cmd)
        } else {
            (ProjectType::NodeJs, None, "npm install".to_string())
        };
        
        // Look for MCP server entry point
        let entry_point = package_json.get("bin")
            .and_then(|bin| {
                if let Some(bin_str) = bin.as_str() {
                    Some(bin_str.to_string())
                } else if let Some(bin_obj) = bin.as_object() {
                    // Get the first binary entry
                    bin_obj.values().next()
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .or_else(|| {
                package_json.get("main")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        
        // Check for start script
        let run_command = package_json.get("scripts")
            .and_then(|scripts| scripts.get("start"))
            .and_then(|v| v.as_str())
            .map(|_s| {
                if is_monorepo {
                    match package_manager.as_deref() {
                        Some("pnpm") => "pnpm run start".to_string(),
                        Some("yarn") => "yarn start".to_string(),
                        _ => "npm run start".to_string(),
                    }
                } else {
                    "npm run start".to_string()
                }
            });
        
        // Check for Node.js version requirement
        let raw_node_version = package_json.get("engines")
            .and_then(|engines| engines.get("node"))
            .and_then(|v| v.as_str());
        
        let node_version = raw_node_version
            .map(normalize_node_version)
            .or_else(|| Some("20".to_string())); // Default to Node 20
        
        return Ok(Some(ProjectInfo {
            project_type,
            name,
            entry_point,
            install_command: Some(install_command),
            run_command,
            python_version: None,
            node_version,
            is_monorepo,
            package_manager,
        }));
    }
    
    Ok(None)
}

fn detect_rust_project(repo_path: &Path) -> Result<Option<ProjectInfo>> {
    let cargo_path = repo_path.join("Cargo.toml");
    
    if cargo_path.exists() {
        debug!("Found Cargo.toml");
        // For now, we don't support Rust projects but we can detect them
        return Ok(Some(ProjectInfo {
            project_type: ProjectType::Rust,
            name: None,
            entry_point: None,
            install_command: Some("cargo build --release".to_string()),
            run_command: Some("cargo run".to_string()),
            python_version: None,
            node_version: None,
            is_monorepo: false,
            package_manager: None,
        }));
    }
    
    Ok(None)
}

fn parse_pyproject_toml(content: &str) -> Result<ProjectInfo> {
    // Simple TOML parsing for key information
    // For a full implementation, we'd use a proper TOML parser
    
    let mut project_type = ProjectType::PythonUv; // Default to uv
    let mut name = None;
    let mut entry_point = None;
    let mut python_version = Some("3.11".to_string());
    
    // Check for Poetry
    if content.contains("[tool.poetry]") {
        project_type = ProjectType::PythonPoetry;
    }
    
    // Extract project name
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name = ") {
            if let Some(name_str) = line.strip_prefix("name = ") {
                name = Some(name_str.trim_matches('"').to_string());
            }
        }
        
        // Look for Python version requirement
        if line.contains("python = ") || line.contains("requires-python = ") {
            if let Some(version_part) = line.split('=').nth(1) {
                let version = version_part.trim().trim_matches('"');
                if let Some(stripped) = version.strip_prefix(">=") {
                    python_version = Some(stripped.to_string());
                } else if let Some(stripped) = version.strip_prefix("^") {
                    python_version = Some(stripped.to_string());
                }
            }
        }
    }
    
    // Try to find entry points
    if content.contains("[project.scripts]") || content.contains("[tool.poetry.scripts]") {
        // Look for script definitions
        let mut in_scripts = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("[project.scripts]") || line.starts_with("[tool.poetry.scripts]") {
                in_scripts = true;
                continue;
            }
            if in_scripts && line.starts_with('[') {
                break; // End of scripts section
            }
            if in_scripts && line.contains('=') {
                if let Some(script_name) = line.split('=').next() {
                    entry_point = Some(script_name.trim().to_string());
                    break; // Use the first script as entry point
                }
            }
        }
    }
    
    let install_command = match project_type {
        ProjectType::PythonPoetry => Some("poetry install".to_string()),
        ProjectType::PythonUv => Some("uv pip install -e .".to_string()),
        _ => Some("pip install -e .".to_string()),
    };
    
    Ok(ProjectInfo {
        project_type,
        name,
        entry_point,
        install_command,
        run_command: None,
        python_version,
        node_version: None,
        is_monorepo: false,
        package_manager: None,
    })
}

fn extract_setup_py_name(repo_path: &Path) -> Result<Option<String>> {
    // Try to extract name from setup.py
    // This is a simplified approach - a full parser would be more robust
    let setup_py_path = repo_path.join("setup.py");
    let content = fs::read_to_string(&setup_py_path)
        .context("Failed to read setup.py")?;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name=") {
            if let Some(name_part) = line.strip_prefix("name=") {
                let name = name_part.trim_matches('"').trim_matches('\'').trim_matches(',');
                return Ok(Some(name.to_string()));
            }
        }
    }
    
    Ok(None)
}

fn detect_nodejs_monorepo(repo_path: &Path, package_json: &Value) -> Result<bool> {
    // Check for workspace configuration in package.json
    if package_json.get("workspaces").is_some() {
        debug!("Found workspaces in package.json");
        return Ok(true);
    }
    
    // Check for pnpm-workspace.yaml
    if repo_path.join("pnpm-workspace.yaml").exists() {
        debug!("Found pnpm-workspace.yaml");
        return Ok(true);
    }
    
    // Check for lerna.json
    if repo_path.join("lerna.json").exists() {
        debug!("Found lerna.json");
        return Ok(true);
    }
    
    // Check for rush.json
    if repo_path.join("rush.json").exists() {
        debug!("Found rush.json");
        return Ok(true);
    }
    
    // Check for nx.json
    if repo_path.join("nx.json").exists() {
        debug!("Found nx.json");
        return Ok(true);
    }
    
    Ok(false)
}

fn detect_package_manager(repo_path: &Path) -> Result<Option<String>> {
    // Check for lock files to determine package manager
    if repo_path.join("pnpm-lock.yaml").exists() {
        return Ok(Some("pnpm".to_string()));
    }
    
    if repo_path.join("yarn.lock").exists() {
        return Ok(Some("yarn".to_string()));
    }
    
    if repo_path.join("package-lock.json").exists() {
        return Ok(Some("npm".to_string()));
    }
    
    // Check for packageManager field in package.json
    let package_json_path = repo_path.join("package.json");
    if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path)
            .context("Failed to read package.json")?;
        
        let package_json: Value = serde_json::from_str(&content)
            .context("Failed to parse package.json")?;
            
        if let Some(package_manager) = package_json.get("packageManager")
            .and_then(|v| v.as_str()) {
            if package_manager.starts_with("pnpm") {
                return Ok(Some("pnpm".to_string()));
            } else if package_manager.starts_with("yarn") {
                return Ok(Some("yarn".to_string()));
            }
        }
    }
    
    Ok(None)
}

/// Normalize Node.js version strings to valid Docker tag format
/// Converts version ranges like ">=20", "^18.0.0", "~16.14" to specific versions
fn normalize_node_version(version_str: &str) -> String {
    let version = version_str.trim();
    
    // Remove common version range operators and extract the base version
    let normalized = if let Some(base_version) = version.strip_prefix(">=") {
        // >=18 -> 18, >=18.0.0 -> 18
        extract_major_version(base_version)
    } else if let Some(base_version) = version.strip_prefix("^") {
        // ^18.0.0 -> 18
        extract_major_version(base_version)
    } else if let Some(base_version) = version.strip_prefix("~") {
        // ~18.14 -> 18 
        extract_major_version(base_version)
    } else if let Some(base_version) = version.strip_prefix("=") {
        // =18.0.0 -> 18
        extract_major_version(base_version)
    } else if version.contains(" || ") {
        // ">=16 || >=18" -> take the first range and normalize it
        let first_range = version.split(" || ").next().unwrap_or(version);
        return normalize_node_version(first_range);
    } else if version.contains(" - ") {
        // "16.0.0 - 18.0.0" -> use the lower bound
        let lower_bound = version.split(" - ").next().unwrap_or(version);
        extract_major_version(lower_bound)
    } else if version.contains("-") && !version.starts_with('-') {
        // Handle ranges like "16-18" -> use first number
        let first_part = version.split('-').next().unwrap_or(version);
        extract_major_version(first_part)
    } else {
        // Already a clean version like "18" or "18.0.0"
        extract_major_version(version)
    };
    
    // Validate that we have a reasonable Node.js version
    if let Ok(major_version) = normalized.parse::<u32>() {
        // Node.js major versions typically range from 14-22 (as of 2024)
        // If it's within reasonable bounds, use it; otherwise default to 20
        if (14..=30).contains(&major_version) {
            major_version.to_string()
        } else {
            "20".to_string()
        }
    } else {
        // If parsing fails, default to Node 20
        "20".to_string()
    }
}

/// Extract major version number from a version string
fn extract_major_version(version: &str) -> String {
    let cleaned = version.trim();
    
    // Handle empty or whitespace-only strings
    if cleaned.is_empty() {
        return "20".to_string();
    }
    
    // Split by dot and take the first part (major version)
    let major_part = cleaned.split('.').next().unwrap_or(cleaned);
    
    // Remove any non-numeric characters from the end
    let mut major_version = String::new();
    for ch in major_part.chars() {
        if ch.is_ascii_digit() {
            major_version.push(ch);
        } else {
            break; // Stop at first non-digit
        }
    }
    
    // Return the major version or default to 20
    if major_version.is_empty() {
        "20".to_string()
    } else {
        major_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_python_poetry_project() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject_content = r#"
[tool.poetry]
name = "test-mcp-server"
version = "0.1.0"

[tool.poetry.dependencies]
python = "^3.11"

[tool.poetry.scripts]
test-server = "test_mcp_server:main"
"#;
        
        fs::write(temp_dir.path().join("pyproject.toml"), pyproject_content).unwrap();
        
        let project_info = detect_project_type(temp_dir.path()).unwrap();
        assert_eq!(project_info.project_type, ProjectType::PythonPoetry);
        assert_eq!(project_info.name, Some("test-mcp-server".to_string()));
        assert_eq!(project_info.entry_point, Some("test-server".to_string()));
    }

    #[test]
    fn test_detect_nodejs_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_content = r#"
{
  "name": "test-mcp-server",
  "version": "1.0.0",
  "main": "index.js",
  "bin": {
    "test-server": "./bin/server.js"
  },
  "scripts": {
    "start": "node index.js"
  }
}
"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json_content).unwrap();
        
        let project_info = detect_project_type(temp_dir.path()).unwrap();
        assert_eq!(project_info.project_type, ProjectType::NodeJs);
        assert_eq!(project_info.name, Some("test-mcp-server".to_string()));
        assert_eq!(project_info.entry_point, Some("./bin/server.js".to_string()));
    }
    
    #[test]
    fn test_normalize_node_version() {
        // Test range operators
        assert_eq!(normalize_node_version(">=20"), "20");
        assert_eq!(normalize_node_version(">=18.0.0"), "18");
        assert_eq!(normalize_node_version("^18.0.0"), "18");
        assert_eq!(normalize_node_version("~16.14"), "16");
        assert_eq!(normalize_node_version("=18.0.0"), "18");
        
        // Test exact versions
        assert_eq!(normalize_node_version("20"), "20");
        assert_eq!(normalize_node_version("18.0.0"), "18");
        assert_eq!(normalize_node_version("16.14.2"), "16");
        
        // Test complex ranges
        assert_eq!(normalize_node_version(">=16 || >=18"), "16");
        assert_eq!(normalize_node_version("16.0.0 - 18.0.0"), "16");
        assert_eq!(normalize_node_version("16-18"), "16");
        
        // Test edge cases
        assert_eq!(normalize_node_version(""), "20");
        assert_eq!(normalize_node_version("   "), "20");
        assert_eq!(normalize_node_version("invalid"), "20");
        assert_eq!(normalize_node_version("abc"), "20");
        
        // Test out of range versions (should default to 20)
        assert_eq!(normalize_node_version(">=50"), "20");
        assert_eq!(normalize_node_version("5"), "20");
        
        // Test edge of valid range
        assert_eq!(normalize_node_version("14"), "14");
        assert_eq!(normalize_node_version("22"), "22");
    }
    
    #[test] 
    fn test_extract_major_version() {
        assert_eq!(extract_major_version("18.0.0"), "18");
        assert_eq!(extract_major_version("20"), "20");
        assert_eq!(extract_major_version("16.14.2"), "16");
        assert_eq!(extract_major_version("18x"), "18");
        assert_eq!(extract_major_version("20.1.0-alpha"), "20");
        assert_eq!(extract_major_version(""), "20");
        assert_eq!(extract_major_version("abc"), "20");
    }
    
    #[test]
    fn test_nodejs_project_with_engine_versions() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test with range version
        let package_json_content = r#"
{
  "name": "test-server",
  "version": "1.0.0",
  "engines": {
    "node": ">=18"
  },
  "scripts": {
    "start": "node index.js"
  }
}
"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json_content).unwrap();
        
        let project_info = detect_project_type(temp_dir.path()).unwrap();
        assert_eq!(project_info.project_type, ProjectType::NodeJs);
        assert_eq!(project_info.node_version, Some("18".to_string()));
    }
}