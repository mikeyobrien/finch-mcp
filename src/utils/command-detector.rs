use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    PythonUvx,
    PythonPip,
    NodeNpm,
    NodeNpx,
    Generic,
}

#[derive(Debug, Clone)]
pub struct CommandDetails {
    pub cmd_type: CommandType,
    pub command: String,
    pub args: Vec<String>,
    pub package_name: Option<String>,
}

pub fn detect_command_type(command: &str, args: &[String]) -> CommandDetails {
    let command = command.to_lowercase();
    
    // Check for uvx command (Python)
    if command == "uvx" {
        return CommandDetails {
            cmd_type: CommandType::PythonUvx,
            command: command.to_string(),
            args: args.to_vec(),
            package_name: args.first().cloned(),
        };
    }
    
    // Check for pip command (Python)
    if command == "pip" || command == "pip3" {
        let package_name = if args.len() >= 2 && args[0] == "install" {
            Some(args[1].clone())
        } else {
            None
        };
        
        return CommandDetails {
            cmd_type: CommandType::PythonPip,
            command: command.to_string(),
            args: args.to_vec(),
            package_name,
        };
    }
    
    // Check for npm command (Node.js)
    if command == "npm" {
        let package_name = if args.len() >= 2 && (args[0] == "install" || args[0] == "exec") {
            Some(args[1].clone())
        } else {
            None
        };
        
        return CommandDetails {
            cmd_type: CommandType::NodeNpm,
            command: command.to_string(),
            args: args.to_vec(),
            package_name,
        };
    }
    
    // Check for npx command (Node.js)
    if command == "npx" {
        return CommandDetails {
            cmd_type: CommandType::NodeNpx,
            command: command.to_string(),
            args: args.to_vec(),
            package_name: args.first().cloned(),
        };
    }
    
    // Default to generic
    CommandDetails {
        cmd_type: CommandType::Generic,
        command: command.to_string(),
        args: args.to_vec(),
        package_name: None,
    }
}

pub fn generate_dockerfile_content(details: &CommandDetails) -> String {
    match details.cmd_type {
        CommandType::PythonUvx => {
            let package_name = details.package_name.clone().unwrap_or_default();
            format!(
                r#"FROM python:3.9-slim

# Install uv for package management
RUN pip install uv

# Install required package
RUN uv pip install {package_name}

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the command with arguments
CMD ["sh", "-c", "{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                package_name,
                format!("{} {}", details.command, details.args.join(" "))
            )
        }
        CommandType::PythonPip => {
            format!(
                r#"FROM python:3.9-slim

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Install and run the command
CMD ["sh", "-c", "{} {} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                details.command,
                details.args.join(" ")
            )
        }
        CommandType::NodeNpm | CommandType::NodeNpx => {
            format!(
                r#"FROM node:18-slim

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the command with arguments
CMD ["sh", "-c", "{} {} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                details.command,
                details.args.join(" ")
            )
        }
        CommandType::Generic => {
            format!(
                r#"FROM debian:bullseye-slim

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the command with arguments
CMD ["sh", "-c", "{} {} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                details.command,
                details.args.join(" ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_uvx_command() {
        let args = vec!["mcp-server-time".to_string()];
        let details = detect_command_type("uvx", &args);
        
        assert_eq!(details.cmd_type, CommandType::PythonUvx);
        assert_eq!(details.package_name, Some("mcp-server-time".to_string()));
    }
    
    #[test]
    fn test_detect_npm_command() {
        let args = vec!["install".to_string(), "@modelcontextprotocol/inspector".to_string()];
        let details = detect_command_type("npm", &args);
        
        assert_eq!(details.cmd_type, CommandType::NodeNpm);
        assert_eq!(details.package_name, Some("@modelcontextprotocol/inspector".to_string()));
    }
    
    #[test]
    fn test_dockerfile_generation_uvx() {
        let details = CommandDetails {
            cmd_type: CommandType::PythonUvx,
            command: "uvx".to_string(),
            args: vec!["mcp-server-time".to_string(), "--local-timezone".to_string(), "UTC".to_string()],
            package_name: Some("mcp-server-time".to_string()),
        };
        
        let dockerfile = generate_dockerfile_content(&details);
        assert!(dockerfile.contains("FROM python:3.9-slim"));
        assert!(dockerfile.contains("RUN pip install uv"));
        assert!(dockerfile.contains("RUN uv pip install mcp-server-time"));
        assert!(dockerfile.contains("uvx mcp-server-time --local-timezone UTC"));
    }
}