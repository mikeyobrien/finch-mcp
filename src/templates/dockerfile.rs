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
            base_image: "node:20-alpine".to_string(),
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
        r#"# Multi-stage build for smaller final image
FROM {} AS builder

WORKDIR /app

# Copy package files first for better layer caching
COPY package*.json ./

# Install dependencies and build tools
{}

# Install application dependencies
RUN npm ci --omit=dev --no-audit --no-fund

# Copy application code
COPY . .

# Final runtime stage
FROM {} AS runtime

# Install dumb-init and runtime dependencies
RUN apk add --no-cache dumb-init{}

# Create non-root user
RUN addgroup -g 1000 -S mcp && \
    adduser -u 1000 -S mcp -G mcp

WORKDIR /app

# Copy built application from builder stage
COPY --from=builder --chown=mcp:mcp /app /app

# Set optimized environment variables
ENV NODE_ENV=production \
    NPM_CONFIG_CACHE=/tmp/.npm \
    NPM_CONFIG_LOGLEVEL=warn \
    MCP_ENABLED=true \
    MCP_STDIO=true

# Set non-root user for security
USER mcp

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--"]

# Run in STDIO mode
CMD ["python", "-m", "mcp_server_time", "--local-timezone", "{}"]
"#,
        options.base_image,
        if options.python_dependencies {
            "RUN apk add --no-cache --virtual .build-deps \\\n    python3-dev \\\n    py3-pip \\\n    gcc \\\n    musl-dev \\\n    && apk add --no-cache python3 py3-pip"
        } else {
            "# No Python dependencies needed"
        },
        options.base_image,
        if options.python_dependencies {
            " python3 py3-pip && \\\n    pip3 install --no-cache-dir --break-system-packages mcp-server-time"
        } else {
            ""
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
            base_image: "node:20-alpine".to_string(),
            python_dependencies: true,
            timezone: Some("Europe/London".to_string()),
        };
        
        let dockerfile = generate_stdio_dockerfile(&options);
        
        println!("Generated Dockerfile:\n{}", dockerfile);
        
        assert!(dockerfile.contains("FROM node:20-alpine"));
        assert!(dockerfile.contains("Multi-stage build"));
        assert!(dockerfile.contains("dumb-init"));
        assert!(dockerfile.contains("--local-timezone"));
        assert!(dockerfile.contains("Europe/London"));
    }
    
    #[test]
    fn test_generate_stdio_dockerfile_without_python() {
        let options = DockerfileOptions {
            base_image: "node:20-alpine".to_string(),
            python_dependencies: false,
            timezone: None,
        };
        
        let dockerfile = generate_stdio_dockerfile(&options);
        
        println!("Generated Dockerfile:\n{}", dockerfile);
        
        assert!(dockerfile.contains("FROM node:20-alpine"));
        assert!(dockerfile.contains("Multi-stage build"));
        assert!(!dockerfile.contains("python3-dev"));
        assert!(dockerfile.contains("America/Chicago"));
    }
}