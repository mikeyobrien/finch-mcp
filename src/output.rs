/// Smart output macro that respects quiet mode
/// 
/// This macro automatically checks for MCP_STDIO environment variable
/// and suppresses output when in STDIO mode for clean MCP communication.

use std::sync::OnceLock;

/// Cache the MCP_STDIO environment variable check
static IS_QUIET_MODE: OnceLock<bool> = OnceLock::new();

/// Check if we're in quiet mode (MCP_STDIO is set)
pub fn is_quiet_mode() -> bool {
    *IS_QUIET_MODE.get_or_init(|| {
        std::env::var("MCP_STDIO").is_ok()
    })
}

/// Print status message only if not in quiet mode
/// Usage: status!("Starting server...")
#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => {
        if !$crate::output::is_quiet_mode() {
            println!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiet_mode_detection() {
        // Test with environment variable unset
        std::env::remove_var("MCP_STDIO");
        assert!(!is_quiet_mode());
        
        // Note: We can't easily test with env var set in the same process
        // due to OnceLock caching, but the logic is straightforward
    }
}