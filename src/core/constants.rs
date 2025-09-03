//! Application constants
//!
//! This module defines named constants to replace magic numbers throughout the codebase.

use std::time::Duration;

/// Timeout durations
pub mod timeouts {
    use super::Duration;

    /// Default API request timeout (90 seconds)
    pub const API_REQUEST: Duration = Duration::from_secs(90);

    /// Web search request timeout (15 seconds)
    pub const WEB_SEARCH: Duration = Duration::from_secs(15);

    // MCP service timeout (30 seconds) - temporarily disabled
    // pub const MCP_SERVICE: Duration = Duration::from_secs(30);
}

/// Cache configuration
pub mod cache {
    use super::Duration;

    /// Default API response cache TTL (5 minutes)
    pub const API_RESPONSE_TTL: Duration = Duration::from_secs(300);
}

/// Cryptographic constants
pub mod crypto {
    /// AES-256 key length in bytes (256 bits)
    pub const AES_256_KEY_LEN: usize = 32;

    /// AES-GCM authentication tag length in bytes
    pub const AES_GCM_TAG_LEN: usize = 16;

    /// AES-GCM nonce length in bytes (96 bits)
    pub const AES_GCM_NONCE_LEN: usize = 12;

    /// Minimum encrypted data length (nonce + tag)
    pub const MIN_ENCRYPTED_LEN: usize = AES_GCM_NONCE_LEN + AES_GCM_TAG_LEN;

    /// SHA-256 hash length in bytes
    #[allow(dead_code)]
    pub const SHA256_LEN: usize = 32;
}

/// Test data sizes
pub mod test_data {
    /// Large test message size (1MB)
    #[allow(dead_code)]
    pub const LARGE_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Invalid key length for testing (128 bits = 16 bytes)
    #[allow(dead_code)]
    pub const INVALID_KEY_LEN: usize = 16;

    /// Short data length for testing (less than minimum encrypted length)
    #[allow(dead_code)]
    pub const SHORT_DATA_LEN: usize = 20;
}

/// Menu choices
pub mod menu {
    /// Start new chat session
    #[allow(dead_code)]
    pub const START_CHAT: &str = "1";

    /// List previous sessions
    #[allow(dead_code)]
    pub const LIST_SESSIONS: &str = "2";

    /// View session history
    #[allow(dead_code)]
    pub const VIEW_SESSION: &str = "3";

    /// Export session history
    #[allow(dead_code)]
    pub const EXPORT_SESSION: &str = "4";

    /// Quit application
    #[allow(dead_code)]
    pub const QUIT: &str = "5";
}

/// Exit commands
pub mod exit_commands {
    /// Exit command
    #[allow(dead_code)]
    pub const EXIT: &str = "exit";

    /// Quit command
    #[allow(dead_code)]
    pub const QUIT: &str = "quit";
}

/// Tool commands
pub mod tools {
    /// Command execution prefix
    #[allow(dead_code)]
    pub const RUN_COMMAND: &str = "[RUN_COMMAND";

    /// Web search prefix
    #[allow(dead_code)]
    pub const SEARCH: &str = "[SEARCH:";

    /// Command suffix
    #[allow(dead_code)]
    pub const COMMAND_SUFFIX: &str = "]";
}
