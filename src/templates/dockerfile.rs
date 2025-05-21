/// Options for generating a Dockerfile for STDIO mode
#[derive(Debug, Clone)]
pub struct DockerfileOptions {
    /// Base Docker image
    pub base_image: String,
    
    /// Whether to include Python and MCP time server dependencies
    pub python_dependencies: bool,
    
    /// Local timezone for the MCP time server
    pub timezone: Option<String>,
}

impl Default for DockerfileOptions {
    fn default() -> Self {
        Self {
            base_image: "node:18-alpine".to_string(),
            python_dependencies: true,
            timezone: Some("America/Chicago".to_string()),
        }
    }
}

/// Generate a Dockerfile for STDIO mode
pub fn generate_stdio_dockerfile(options: &DockerfileOptions) -> String {
    // Set default timezone if not provided
    let timezone = options.timezone.as_deref().unwrap_or("America/Chicago");
    
    format!(
        r#"FROM {}

WORKDIR /app

# Copy package files first for better layer caching
COPY package*.json ./

# Install dependencies{}

# Copy application code
COPY . .

# Set non-root user for security
USER node

# Run in STDIO mode
CMD ["python", "-m", "mcp_server_time", "--local-timezone", "{}"]
"#,
        options.base_image,
        if options.python_dependencies {
            "\nRUN npm install --omit=dev && \\\n    apk add --no-cache python3 py3-pip && \\\n    pip3 install --break-system-packages mcp-server-time"
        } else {
            "\nRUN npm install --omit=dev"
        },
        timezone
    )
}

/// Write Dockerfile to a specified path
pub async fn write_dockerfile_to_file(
    path: &str, 
    options: &DockerfileOptions
) -> anyhow::Result<()> {
    let content = generate_stdio_dockerfile(options);
    tokio::fs::write(path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_stdio_dockerfile_with_python() {
        let options = DockerfileOptions {
            base_image: "node:18-alpine".to_string(),
            python_dependencies: true,
            timezone: Some("Europe/London".to_string()),
        };
        
        let dockerfile = generate_stdio_dockerfile(&options);
        
        println!("Generated Dockerfile:\n{}", dockerfile);
        
        assert!(dockerfile.contains("FROM node:18-alpine"));
        assert!(dockerfile.contains("apk add --no-cache python3 py3-pip"));
        assert!(dockerfile.contains("pip3 install --break-system-packages mcp-server-time"));
        assert!(dockerfile.contains("\"--local-timezone\", \"Europe/London\"")); // Fixed assertion
    }
    
    #[test]
    fn test_generate_stdio_dockerfile_without_python() {
        let options = DockerfileOptions {
            base_image: "node:16-slim".to_string(),
            python_dependencies: false,
            timezone: None,
        };
        
        let dockerfile = generate_stdio_dockerfile(&options);
        
        println!("Generated Dockerfile:\n{}", dockerfile);
        
        assert!(dockerfile.contains("FROM node:16-slim"));
        assert!(!dockerfile.contains("apk add --no-cache python3 py3-pip"));
        assert!(dockerfile.contains("\"--local-timezone\", \"America/Chicago\"")); // Fixed assertion
    }
}