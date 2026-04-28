// Copyright 2026 harpertoken
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

use harper_core::{ApprovalProfile, SandboxProfile};
use std::fs;
use std::io;
use std::path::Path;

pub fn save_execution_policy_settings(
    approval: ApprovalProfile,
    sandbox: SandboxProfile,
    retry_max_attempts: u32,
    allowed_commands: &[String],
    blocked_commands: &[String],
) -> io::Result<()> {
    let path = Path::new("config/local.toml");
    let existing = fs::read_to_string(path).unwrap_or_default();
    let updated = upsert_exec_policy_settings(
        &existing,
        approval,
        sandbox,
        retry_max_attempts,
        allowed_commands,
        blocked_commands,
    );
    fs::write(path, updated)
}

fn upsert_exec_policy_settings(
    input: &str,
    approval: ApprovalProfile,
    sandbox: SandboxProfile,
    retry_max_attempts: u32,
    allowed_commands: &[String],
    blocked_commands: &[String],
) -> String {
    let approval_line = format!("approval_profile = \"{}\"", approval_profile_name(approval));
    let sandbox_line = format!("sandbox_profile = \"{}\"", sandbox_profile_name(sandbox));
    let retry_line = format!("retry_max_attempts = {}", retry_max_attempts);
    let allowed_line = format!(
        "allowed_commands = {}",
        serde_json::to_string(allowed_commands).unwrap_or_else(|_| "[]".to_string())
    );
    let blocked_line = format!(
        "blocked_commands = {}",
        serde_json::to_string(blocked_commands).unwrap_or_else(|_| "[]".to_string())
    );

    let mut lines: Vec<String> = input.lines().map(str::to_string).collect();
    let section_start = lines.iter().position(|line| line.trim() == "[exec_policy]");

    match section_start {
        Some(start) => {
            let section_end = lines
                .iter()
                .enumerate()
                .skip(start + 1)
                .find(|(_, line)| line.trim_start().starts_with('['))
                .map(|(index, _)| index)
                .unwrap_or(lines.len());

            let mut section_body: Vec<String> = lines[start + 1..section_end]
                .iter()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    !(trimmed.starts_with("approval_profile")
                        || trimmed.starts_with("sandbox_profile")
                        || trimmed.starts_with("retry_max_attempts")
                        || trimmed.starts_with("allowed_commands")
                        || trimmed.starts_with("blocked_commands"))
                })
                .cloned()
                .collect();
            section_body.insert(0, blocked_line);
            section_body.insert(0, allowed_line);
            section_body.insert(0, retry_line);
            section_body.insert(0, sandbox_line);
            section_body.insert(0, approval_line);

            lines.splice(start + 1..section_end, section_body);
        }
        None => {
            if !lines.is_empty() && !lines.last().is_some_and(|line| line.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push("[exec_policy]".to_string());
            lines.push(approval_line);
            lines.push(sandbox_line);
            lines.push(retry_line);
            lines.push(allowed_line);
            lines.push(blocked_line);
        }
    }

    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

pub fn approval_profile_name(profile: ApprovalProfile) -> &'static str {
    match profile {
        ApprovalProfile::Strict => "strict",
        ApprovalProfile::AllowListed => "allow_listed",
        ApprovalProfile::AllowAll => "allow_all",
    }
}

pub fn sandbox_profile_name(profile: SandboxProfile) -> &'static str {
    match profile {
        SandboxProfile::Disabled => "disabled",
        SandboxProfile::Workspace => "workspace",
        SandboxProfile::NetworkedWorkspace => "networked_workspace",
    }
}

pub fn next_approval_profile(profile: ApprovalProfile) -> ApprovalProfile {
    match profile {
        ApprovalProfile::Strict => ApprovalProfile::AllowListed,
        ApprovalProfile::AllowListed => ApprovalProfile::AllowAll,
        ApprovalProfile::AllowAll => ApprovalProfile::Strict,
    }
}

pub fn next_sandbox_profile(profile: SandboxProfile) -> SandboxProfile {
    match profile {
        SandboxProfile::Disabled => SandboxProfile::Workspace,
        SandboxProfile::Workspace => SandboxProfile::NetworkedWorkspace,
        SandboxProfile::NetworkedWorkspace => SandboxProfile::Disabled,
    }
}

pub fn next_retry_max_attempts(value: u32) -> u32 {
    (value + 1) % 4
}

#[cfg(test)]
mod tests {
    use super::{
        approval_profile_name, next_approval_profile, next_retry_max_attempts,
        next_sandbox_profile, sandbox_profile_name, upsert_exec_policy_settings,
    };
    use harper_core::{ApprovalProfile, SandboxProfile};

    #[test]
    fn upsert_exec_policy_profiles_creates_section() {
        let output = upsert_exec_policy_settings(
            "",
            ApprovalProfile::AllowListed,
            SandboxProfile::Workspace,
            2,
            &["git".to_string()],
            &["rm".to_string()],
        );
        assert!(output.contains("[exec_policy]"));
        assert!(output.contains("approval_profile = \"allow_listed\""));
        assert!(output.contains("sandbox_profile = \"workspace\""));
        assert!(output.contains("retry_max_attempts = 2"));
        assert!(output.contains("allowed_commands = [\"git\"]"));
        assert!(output.contains("blocked_commands = [\"rm\"]"));
    }

    #[test]
    fn upsert_exec_policy_profiles_replaces_existing_values() {
        let input = "[exec_policy]\napproval_profile = \"strict\"\nsandbox_profile = \"disabled\"\nallowed_commands = [\"git\"]\nblocked_commands = [\"rm\"]\n";
        let output = upsert_exec_policy_settings(
            input,
            ApprovalProfile::AllowAll,
            SandboxProfile::NetworkedWorkspace,
            3,
            &["ls".to_string()],
            &["sudo".to_string()],
        );
        assert!(output.contains("approval_profile = \"allow_all\""));
        assert!(output.contains("sandbox_profile = \"networked_workspace\""));
        assert!(output.contains("retry_max_attempts = 3"));
        assert!(output.contains("allowed_commands = [\"ls\"]"));
        assert!(output.contains("blocked_commands = [\"sudo\"]"));
        assert!(!output.contains("approval_profile = \"strict\""));
    }

    #[test]
    fn profile_name_helpers_match_config_values() {
        assert_eq!(
            approval_profile_name(ApprovalProfile::AllowListed),
            "allow_listed"
        );
        assert_eq!(
            sandbox_profile_name(SandboxProfile::NetworkedWorkspace),
            "networked_workspace"
        );
    }

    #[test]
    fn profile_cycle_helpers_advance() {
        assert_eq!(
            next_approval_profile(ApprovalProfile::Strict),
            ApprovalProfile::AllowListed
        );
        assert_eq!(
            next_sandbox_profile(SandboxProfile::Workspace),
            SandboxProfile::NetworkedWorkspace
        );
        assert_eq!(next_retry_max_attempts(0), 1);
        assert_eq!(next_retry_max_attempts(3), 0);
    }
}
