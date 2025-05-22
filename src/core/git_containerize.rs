use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{debug, info};
use tempfile::TempDir;
use uuid::Uuid;

use crate::utils::git_repository::GitRepository;
use crate::utils::project_detector::{detect_project_type, ProjectType, ProjectInfo};
use crate::finch::client::{FinchClient, StdioRunOptions};

pub struct GitContainerizeOptions {
    pub repo_url: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
}

pub struct LocalContainerizeOptions {
    pub local_path: String,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
}

pub async fn git_containerize_and_run(options: GitContainerizeOptions) -> Result<()> {
    // Parse and clone the repository
    let mut git_repo = GitRepository::new(&options.repo_url);
    
    info!("Cloning repository: {}", git_repo.url);
    let repo_path = git_repo.clone_to_temp().await?;
    
    // Detect the project type
    let project_info = detect_project_type(&repo_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in repository"));
    }
    
    // Generate unique image name
    let image_name = format!("mcp-git-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("default"));
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args)?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Copy repository contents to build context
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy repository files to build context
    copy_dir_all(&repo_path, &build_context).context("Failed to copy repository to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image
    info!("Building container image: {}", image_name);
    let build_status = Command::new("finch")
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg(&build_context)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute finch build command")?;
    
    if !build_status.success() {
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    // Prepare environment variables
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    // Add extra arguments if provided
    if !options.args.is_empty() {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
    // Run the container
    info!("Running containerized git repository");
    let finch_client = FinchClient::new();
    let run_options = StdioRunOptions {
        image_name,
        env_vars,
        volumes: options.volumes,
    };
    
    finch_client.run_stdio_container(&run_options).await
}

pub async fn local_containerize_and_run(options: LocalContainerizeOptions) -> Result<()> {
    let local_path = PathBuf::from(&options.local_path);
    
    // Validate that the path exists and is a directory
    if !local_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", options.local_path));
    }
    
    if !local_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", options.local_path));
    }
    
    info!("Containerizing local directory: {}", local_path.display());
    
    // Detect the project type
    let project_info = detect_project_type(&local_path)?;
    debug!("Detected project: {:?}", project_info);
    
    if project_info.project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!("Could not detect project type in directory"));
    }
    
    // Generate unique image name
    let image_name = format!("mcp-local-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("default"));
    
    // Create temp directory for Dockerfile
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Generate Dockerfile content based on project type
    let dockerfile_content = generate_dockerfile_for_project(&project_info, &options.args)?;
    debug!("Generated Dockerfile:\n{}", dockerfile_content);
    
    // Write Dockerfile
    fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;
    info!("Created Dockerfile at: {:?}", dockerfile_path);
    
    // Create build context and copy local directory contents
    let build_context = temp_dir.path().join("context");
    fs::create_dir_all(&build_context).context("Failed to create build context directory")?;
    
    // Copy local directory files to build context
    copy_dir_all(&local_path, &build_context).context("Failed to copy local directory to build context")?;
    
    // Copy Dockerfile to build context
    fs::copy(&dockerfile_path, build_context.join("Dockerfile"))?;
    
    // Build the container image
    info!("Building container image: {}", image_name);
    let build_status = Command::new("finch")
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg(&build_context)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute finch build command")?;
    
    if !build_status.success() {
        return Err(anyhow::anyhow!("Container build failed with status: {}", build_status));
    }
    
    // Prepare environment variables
    let mut env_vars = options.env_vars;
    env_vars.push("MCP_ENABLED=true".to_string());
    env_vars.push("MCP_STDIO=true".to_string());
    
    // Add extra arguments if provided
    if !options.args.is_empty() {
        let extra_args = options.args.join(" ");
        env_vars.push(format!("EXTRA_ARGS={}", extra_args));
    }
    
    // Run the container
    info!("Running containerized local directory");
    let finch_client = FinchClient::new();
    let run_options = StdioRunOptions {
        image_name,
        env_vars,
        volumes: options.volumes,
    };
    
    finch_client.run_stdio_container(&run_options).await
}

fn generate_dockerfile_for_project(project_info: &ProjectInfo, args: &[String]) -> Result<String> {
    match project_info.project_type {
        ProjectType::PythonPoetry => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if let Some(ref entry_point) = project_info.entry_point {
                format!("poetry run {}", entry_point)
            } else if !args.is_empty() {
                format!("poetry run python {}", args.join(" "))
            } else {
                "poetry run python -m src".to_string()
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app

# Install poetry
RUN pip install poetry

# Copy project files
COPY . .

# Configure poetry
RUN poetry config virtualenvs.create false

# Install dependencies
RUN poetry install

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command
            ))
        }
        
        ProjectType::PythonUv => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if let Some(ref entry_point) = project_info.entry_point {
                entry_point.clone()
            } else if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python -m src".to_string()
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app

# Install uv
RUN pip install uv

# Copy project files
COPY . .

# Install dependencies
RUN uv pip install --system -e .

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command
            ))
        }
        
        ProjectType::PythonSetupPy => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python setup.py".to_string()
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app

# Copy project files
COPY . .

# Install dependencies
RUN pip install -e .

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command
            ))
        }
        
        ProjectType::PythonRequirements => {
            let python_version = project_info.python_version.as_deref().unwrap_or("3.11");
            let entry_command = if !args.is_empty() {
                format!("python {}", args.join(" "))
            } else {
                "python main.py".to_string()
            };
            
            Ok(format!(
                r#"FROM python:{}-slim

WORKDIR /app

# Copy project files
COPY . .

# Install dependencies
RUN pip install -r requirements.txt

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                python_version,
                entry_command
            ))
        }
        
        ProjectType::NodeJs => {
            let node_version = project_info.node_version.as_deref().unwrap_or("20");
            let entry_command = if let Some(ref run_cmd) = project_info.run_command {
                run_cmd.clone()
            } else if let Some(ref entry_point) = project_info.entry_point {
                format!("node {}", entry_point)
            } else if !args.is_empty() {
                format!("node {}", args.join(" "))
            } else {
                "npm start".to_string()
            };
            
            Ok(format!(
                r#"FROM node:{}-slim

WORKDIR /app

# Copy project files
COPY . .

# Install dependencies
RUN npm install

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                node_version,
                entry_command
            ))
        }
        
        ProjectType::NodeJsMonorepo => {
            let node_version = project_info.node_version.as_deref().unwrap_or("20");
            let package_manager = project_info.package_manager.as_deref().unwrap_or("npm");
            
            let install_command = match package_manager {
                "pnpm" => "pnpm install",
                "yarn" => "yarn install",
                _ => "npm install",
            };
            
            let entry_command = if let Some(ref run_cmd) = project_info.run_command {
                run_cmd.clone()
            } else if let Some(ref entry_point) = project_info.entry_point {
                format!("node {}", entry_point)
            } else if !args.is_empty() {
                format!("node {}", args.join(" "))
            } else {
                match package_manager {
                    "pnpm" => "pnpm start".to_string(),
                    "yarn" => "yarn start".to_string(),
                    _ => "npm start".to_string(),
                }
            };
            
            // For monorepo, we need to install the package manager first
            let pm_install = match package_manager {
                "pnpm" => "RUN npm install -g pnpm",
                "yarn" => "RUN npm install -g yarn", 
                _ => "",
            };
            
            Ok(format!(
                r#"FROM node:{}-slim

WORKDIR /app

# Install package manager if needed
{}

# Copy project files
COPY . .

# Install dependencies
RUN {}

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the application
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                node_version,
                pm_install,
                install_command,
                entry_command
            ))
        }
        
        ProjectType::Rust => {
            Err(anyhow::anyhow!("Rust projects are not yet supported for git containerization"))
        }
        
        ProjectType::Unknown => {
            Err(anyhow::anyhow!("Unknown project type cannot be containerized"))
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        
        // Skip hidden files and directories, and common build/cache directories
        if let Some(file_name) = name.to_str() {
            if file_name.starts_with('.') 
                || file_name == "node_modules" 
                || file_name == "__pycache__" 
                || file_name == "target" 
                || file_name == "dist" 
                || file_name == "build" {
                continue;
            }
        }
        
        let dst_path = dst.join(&name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::project_detector::ProjectInfo;

    #[test]
    fn test_generate_dockerfile_python_poetry() {
        let project_info = ProjectInfo {
            project_type: ProjectType::PythonPoetry,
            name: Some("test-server".to_string()),
            entry_point: Some("test-server".to_string()),
            install_command: Some("poetry install".to_string()),
            run_command: None,
            python_version: Some("3.11".to_string()),
            node_version: None,
            is_monorepo: false,
            package_manager: None,
        };
        
        let dockerfile = generate_dockerfile_for_project(&project_info, &[]).unwrap();
        assert!(dockerfile.contains("FROM python:3.11-slim"));
        assert!(dockerfile.contains("RUN pip install poetry"));
        assert!(dockerfile.contains("poetry run test-server"));
    }

    #[test]
    fn test_generate_dockerfile_nodejs() {
        let project_info = ProjectInfo {
            project_type: ProjectType::NodeJs,
            name: Some("test-server".to_string()),
            entry_point: Some("index.js".to_string()),
            install_command: Some("npm install".to_string()),
            run_command: None,
            python_version: None,
            node_version: Some("20".to_string()),
            is_monorepo: false,
            package_manager: None,
        };
        
        let dockerfile = generate_dockerfile_for_project(&project_info, &[]).unwrap();
        assert!(dockerfile.contains("FROM node:20-slim"));
        assert!(dockerfile.contains("RUN npm install"));
        assert!(dockerfile.contains("node index.js"));
    }
}