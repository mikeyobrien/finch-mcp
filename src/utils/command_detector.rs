
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
                r#"FROM python:3.11-slim

# Install uv for package management
RUN pip install uv

# Install required package
RUN uv pip install --system {}

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
                r#"FROM python:3.11-slim

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
        CommandType::NodeNpm => {
            format!(
                r#"FROM node:20-slim

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
        CommandType::NodeNpx => {
            // Special handling for NPX - separate the package from its arguments
            let (package_and_flags, package_args) = if details.args.len() >= 1 {
                // Find the package name (first non-flag argument)
                let mut package_idx = 0;
                let mut flags = Vec::new();
                
                for (i, arg) in details.args.iter().enumerate() {
                    if arg.starts_with('-') {
                        flags.push(arg.clone());
                    } else {
                        package_idx = i;
                        break;
                    }
                }
                
                if package_idx < details.args.len() {
                    let package_name = &details.args[package_idx];
                    let remaining_args = if package_idx + 1 < details.args.len() {
                        details.args[package_idx + 1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    let mut full_flags = flags;
                    full_flags.push(package_name.clone());
                    (full_flags.join(" "), remaining_args)
                } else {
                    (details.args.join(" "), Vec::new())
                }
            } else {
                (details.args.join(" "), Vec::new())
            };
            
            let cmd_args = if !package_args.is_empty() {
                format!(" {}", package_args.join(" "))
            } else {
                String::new()
            };
            
            format!(
                r#"FROM node:20-slim

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the npx command
CMD ["sh", "-c", "npx {}{} ${{EXTRA_ARGS:+$EXTRA_ARGS}}"]
"#,
                package_and_flags,
                cmd_args
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