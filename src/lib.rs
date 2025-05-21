// Export modules
pub mod cli;
pub mod run;
pub mod finch {
    pub mod client;
}
pub mod templates {
    pub mod dockerfile;
}

// Re-export main types for easier access
pub use run::{RunOptions, run_stdio_container};
pub use finch::client::{FinchClient, StdioRunOptions};
pub use templates::dockerfile::{DockerfileOptions, generate_stdio_dockerfile};