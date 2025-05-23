# Contributing to Finch-MCP

Thank you for your interest in contributing to Finch-MCP! This guide will help you get started with development and contribution.

## Getting Started

### Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Finch** (for testing)
   ```bash
   # macOS
   brew install --cask finch
   
   # Linux - see https://runfinch.com/
   ```

3. **Development Tools**
   ```bash
   # Install development dependencies
   cargo install cargo-watch cargo-edit
   ```

### Setting Up Development Environment

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/finch-mcp.git
   cd finch-mcp
   ```

2. **Build the Project**
   ```bash
   cargo build
   ```

3. **Run Tests**
   ```bash
   # Unit tests
   cargo test
   
   # Integration tests (requires Finch)
   cargo test -- --ignored
   ```

4. **Create Development Binary**
   ```bash
   cargo build --release
   # Binary at: target/release/finch-mcp
   ```

## Development Workflow

### Code Structure

```
src/
├── main.rs           # Entry point and command routing
├── cli.rs            # CLI argument parsing
├── run.rs            # Direct container execution
├── core/
│   ├── auto_containerize.rs    # Command containerization
│   └── git_containerize.rs     # Git/local containerization
├── finch/
│   └── client.rs     # Finch CLI wrapper
├── cache/            # Caching system
├── utils/            # Utilities
└── templates/        # Dockerfile templates
```

### Making Changes

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Follow Rust conventions
   - Add tests for new functionality
   - Update documentation

3. **Run Tests**
   ```bash
   # Format code
   cargo fmt
   
   # Run clippy
   cargo clippy -- -D warnings
   
   # Run tests
   cargo test
   ```

4. **Test Manually**
   ```bash
   # Build and test your changes
   cargo build
   ./target/debug/finch-mcp run uvx mcp-server-time
   ```

## Testing

### Unit Tests

Located alongside source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Your test
    }
}
```

### Integration Tests

Located in `tests/`:

```rust
#[tokio::test]
#[ignore = "Requires Finch"]
async fn test_integration() {
    // Your integration test
}
```

### Test Fixtures

Test projects in `tests/fixtures/`:
- `test-mcp-filesystem/` - Node.js MCP server
- `test-mcp-time/` - Another Node.js example
- `test-mcp-stdio.py` - Python MCP server

### Running Specific Tests

```bash
# Run specific test
cargo test test_name

# Run tests in specific module
cargo test cache::

# Run with output
cargo test -- --nocapture
```

## Code Style

### Rust Style

Follow standard Rust conventions:
- Use `cargo fmt` before committing
- Follow clippy suggestions
- Use meaningful variable names
- Add documentation comments

### Documentation

```rust
/// Brief description of function
/// 
/// # Arguments
/// 
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
/// 
/// # Returns
/// 
/// Description of return value
/// 
/// # Examples
/// 
/// ```
/// let result = function(arg1, arg2);
/// ```
pub fn function(arg1: Type, arg2: Type) -> Result<ReturnType> {
    // Implementation
}
```

### Error Handling

Use `anyhow` for error handling:

```rust
use anyhow::{Result, Context};

fn operation() -> Result<()> {
    something()
        .context("Failed to do something")?;
    Ok(())
}
```

### Logging

Use structured logging:

```rust
use log::{info, debug, error};

info!("Starting operation");
debug!("Debug details: {:?}", data);
error!("Operation failed: {}", err);
```

## Adding Features

### Adding a New Command

1. **Update CLI** in `cli.rs`:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // Existing commands...
       NewCommand {
           #[arg(help = "Description")]
           argument: String,
       },
   }
   ```

2. **Implement Handler** in `main.rs`:
   ```rust
   Commands::NewCommand { argument } => {
       handle_new_command(argument).await?;
   }
   ```

3. **Add Tests**
4. **Update Documentation**

### Adding Project Type Support

1. **Update Project Detector** in `utils/project_detector.rs`:
   ```rust
   pub enum ProjectType {
       // Existing types...
       NewType,
   }
   ```

2. **Add Detection Logic**:
   ```rust
   if path.join("new-type-file.ext").exists() {
       return Some(ProjectInfo {
           project_type: ProjectType::NewType,
           // ...
       });
   }
   ```

3. **Add Dockerfile Template**
4. **Test with Real Project**

## Pull Request Process

### Before Submitting

1. **Test Thoroughly**
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   cargo test -- --ignored  # If you have Finch
   ```

2. **Update Documentation**
   - Add/update relevant docs in `docs/`
   - Update README if needed
   - Add examples

3. **Write Good Commit Messages**
   ```
   feat: Add support for Ruby projects
   
   - Detect Gemfile and Gemfile.lock
   - Generate appropriate Dockerfile
   - Support both Bundler and plain Ruby
   
   Fixes #123
   ```

### PR Guidelines

1. **Title**: Clear and descriptive
2. **Description**: 
   - What changes were made
   - Why they were made
   - How to test them
3. **Size**: Keep PRs focused and reasonable in size
4. **Tests**: Include tests for new functionality
5. **Documentation**: Update relevant documentation

### Review Process

1. Automated checks must pass
2. At least one maintainer review
3. Address feedback constructively
4. Squash commits if requested

## Development Tips

### Debugging

```bash
# Run with verbose output
RUST_LOG=debug cargo run -- run -VV ./test-project

# Use VS Code debugger
# Install CodeLLDB extension and use launch.json
```

### Performance Testing

```bash
# Build with optimizations
cargo build --release

# Time execution
time ./target/release/finch-mcp run ./test-project

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph -- run ./test-project
```

### Common Issues

1. **Finch VM not running**
   ```bash
   finch vm start
   ```

2. **Permission errors**
   ```bash
   # Ensure Finch has proper permissions
   finch vm status
   ```

3. **Test failures**
   - Check if Finch is installed
   - Ensure no port conflicts
   - Clean up test artifacts

## Release Process

1. **Version Bump**
   ```bash
   cargo set-version 0.2.0
   ```

2. **Update CHANGELOG.md**

3. **Create Release PR**

4. **After Merge**:
   - Tag release
   - GitHub Actions builds binaries
   - Update documentation

## Getting Help

- **Discord**: Join our development channel
- **GitHub Issues**: For bugs and features
- **Discussions**: For questions and ideas

## Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project README

Thank you for contributing to Finch-MCP!