// Export modules
pub mod cli;
pub mod run;
pub mod finch {
    pub mod client;
}
pub mod templates {
    pub mod dockerfile;
}
pub mod utils {
    pub mod command_detector;
    pub mod command_parser;
}
pub mod core {
    pub mod auto_containerize;
}

// Re-export main types for easier access
pub use run::{RunOptions, run_stdio_container};
pub use finch::client::{FinchClient, StdioRunOptions};
pub use templates::dockerfile::{DockerfileOptions, generate_stdio_dockerfile};
pub use core::auto_containerize::{AutoContainerizeOptions, auto_containerize_and_run};