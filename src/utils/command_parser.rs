/// Parse a command string into command and arguments
/// Handles quoted strings and preserves spaces within quotes
pub fn parse_command_string(input: &str) -> (String, Vec<String>) {
    let input = input.trim();
    
    // Simple parsing - split on spaces but respect quoted strings
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();
    
    for ch in chars.by_ref() {
        match ch {
            '"' | '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_part.is_empty() {
                    parts.push(current_part.clone());
                    current_part.clear();
                }
            }
            _ => {
                current_part.push(ch);
            }
        }
    }
    
    // Add the last part if not empty
    if !current_part.is_empty() {
        parts.push(current_part);
    }
    
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let command = parts[0].clone();
    let args = parts[1..].to_vec();
    
    (command, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_command() {
        let (cmd, args) = parse_command_string("npx @modelcontextprotocol/server-filesystem");
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["@modelcontextprotocol/server-filesystem"]);
    }
    
    #[test]
    fn test_command_with_flags() {
        let (cmd, args) = parse_command_string("npx -y @modelcontextprotocol/server-filesystem /workspace");
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]);
    }
    
    #[test]
    fn test_quoted_arguments() {
        let (cmd, args) = parse_command_string("uvx mcp-server-time --local-timezone 'America/New York'");
        assert_eq!(cmd, "uvx");
        assert_eq!(args, vec!["mcp-server-time", "--local-timezone", "America/New York"]);
    }
    
    #[test]
    fn test_empty_command() {
        let (cmd, args) = parse_command_string("");
        assert_eq!(cmd, "");
        assert_eq!(args, Vec::<String>::new());
    }
}