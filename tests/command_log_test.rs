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

use harper_workspace::core::{ApiConfig, ApiProvider};
use harper_workspace::memory::storage;
use harper_workspace::runtime::config::ExecPolicyConfig;
use harper_workspace::tools::shell::{self, CommandAuditContext};
use rusqlite::Connection;
use tempfile::NamedTempFile;

#[test]
fn test_execute_command_persists_audit_log() {
    let temp_file = NamedTempFile::new().expect("temp db");
    let conn = Connection::open(temp_file.path()).expect("open db");
    storage::init_db(&conn).expect("init db");

    let api_config = ApiConfig {
        provider: ApiProvider::OpenAI,
        api_key: String::new(),
        base_url: String::new(),
        model_name: String::new(),
    };

    let exec_policy = ExecPolicyConfig {
        allowed_commands: Some(vec!["echo".to_string()]),
        blocked_commands: None,
    };

    let audit_ctx = CommandAuditContext {
        conn: &conn,
        session_id: Some("test-session"),
        source: "test_harness",
    };

    let output = shell::execute_command(
        "[RUN_COMMAND echo audit-log]",
        &api_config,
        &exec_policy,
        Some(&audit_ctx),
    )
    .expect("command should succeed");
    assert!(output.contains("audit-log"));

    let entries =
        storage::load_command_logs_for_session(&conn, "test-session", 5).expect("load logs");
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert!(entry.command.contains("echo"));
    assert_eq!(entry.status, "succeeded");
    assert!(entry.approved);
}
