// Copyright 2025 harpertoken
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Application constants
//!
//! This module defines named constants to replace magic numbers throughout the codebase.

use std::time::Duration;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Timeout durations
pub mod timeouts {
    use super::Duration;

    /// Default API request timeout (90 seconds)
    #[allow(dead_code)]
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

/// UI messages
pub mod messages {
    /// Exit message when quitting the application
    #[allow(dead_code)]
    pub const GOODBYE: &str = "Goodbye!";

    /// Menu title
    #[allow(dead_code)]
    pub const MAIN_MENU_TITLE: &str = "Main Menu";

    /// Prompt for user input
    #[allow(dead_code)]
    pub const ENTER_CHOICE: &str = "Enter your choice: ";
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

    /// Read file prefix
    #[allow(dead_code)]
    pub const READ_FILE: &str = "[READ_FILE";

    /// Write file prefix
    #[allow(dead_code)]
    pub const WRITE_FILE: &str = "[WRITE_FILE";

    /// Search and replace prefix
    #[allow(dead_code)]
    pub const SEARCH_REPLACE: &str = "[SEARCH_REPLACE";

    /// Todo management prefix
    #[allow(dead_code)]
    pub const TODO: &str = "[TODO";

    /// GitHub issue creation prefix
    #[allow(dead_code)]
    pub const GITHUB_ISSUE: &str = "[GITHUB_ISSUE";

    /// GitHub PR creation prefix
    #[allow(dead_code)]
    pub const GITHUB_PR: &str = "[GITHUB_PR";

    /// API testing prefix
    #[allow(dead_code)]
    pub const API_TEST: &str = "[API_TEST";

    /// Code analysis prefix
    #[allow(dead_code)]
    pub const CODE_ANALYZE: &str = "[CODE_ANALYZE";

    /// Database query prefix
    #[allow(dead_code)]
    pub const DB_QUERY: &str = "[DB_QUERY";

    /// Image info prefix
    #[allow(dead_code)]
    pub const IMAGE_INFO: &str = "[IMAGE_INFO";

    /// Image resize prefix
    #[allow(dead_code)]
    pub const IMAGE_RESIZE: &str = "[IMAGE_RESIZE";

    /// Command suffix
    #[allow(dead_code)]
    pub const COMMAND_SUFFIX: &str = "]";
}
