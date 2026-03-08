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

use harper_core::core::{ApiConfig, ApiProvider};
use harper_core::memory::storage::{self};
use harper_core::runtime::config::ExecPolicyConfig;
use harper_core::tools::shell::{self, CommandAuditContext};
use rusqlite::Connection;

#[tokio::test]
async fn test_command_logging() {
    let conn = Connection::open_in_memory().unwrap();
    storage::init_db(&conn).unwrap();

    let session_id = "test-session";
    storage::save_session(&conn, session_id).unwrap();

    let audit_ctx = CommandAuditContext {
        conn: &conn,
        session_id: Some(session_id),
        source: "test_source",
    };

    let api_config = ApiConfig {
        provider: ApiProvider::OpenAI,
        api_key: "test-key".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4".to_string(),
    };

    let exec_policy = ExecPolicyConfig {
        allowed_commands: Some(vec!["echo".to_string()]),
        blocked_commands: None,
    };

    let output = shell::execute_command(
        "[RUN_COMMAND echo audit-log]",
        &api_config,
        &exec_policy,
        Some(&audit_ctx),
        None,
    )
    .await
    .expect("command should succeed");

    assert!(output.contains("audit-log"));

    // Verify log entry exists
    let logs = storage::load_command_logs_for_session(&conn, session_id, 10).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].command, "echo audit-log");
    assert_eq!(logs[0].status, "succeeded");
    assert!(!logs[0].requires_approval); // Should be auto-approved
}
