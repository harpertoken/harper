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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanItem {
    pub step: String,
    pub status: PlanStepStatus,
    #[serde(default)]
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanJobStatus {
    WaitingApproval,
    Running,
    Blocked,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanJobRecord {
    pub job_id: String,
    pub tool: String,
    pub command: Option<String>,
    pub status: PlanJobStatus,
    #[serde(default)]
    pub output_transcript: String,
    #[serde(default)]
    pub output_preview: Option<String>,
    #[serde(default)]
    pub has_error_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlanRuntime {
    #[serde(default)]
    pub active_tool: Option<String>,
    #[serde(default)]
    pub active_command: Option<String>,
    #[serde(default)]
    pub active_status: Option<String>,
    #[serde(default)]
    pub active_job_id: Option<String>,
    #[serde(default)]
    pub jobs: Vec<PlanJobRecord>,
}

impl PlanRuntime {
    pub fn has_active_state(&self) -> bool {
        self.active_tool.is_some() || self.active_command.is_some() || self.active_status.is_some()
    }

    pub fn is_empty(&self) -> bool {
        !self.has_active_state() && self.active_job_id.is_none() && self.jobs.is_empty()
    }

    pub fn set_active_tool_state(
        &mut self,
        tool: impl Into<String>,
        command: Option<String>,
        status: impl Into<String>,
    ) {
        self.active_tool = Some(tool.into());
        self.active_command = command;
        self.active_status = Some(status.into());
    }

    pub fn start_job(
        &mut self,
        tool: impl Into<String>,
        command: Option<String>,
        status: PlanJobStatus,
    ) -> String {
        let tool = tool.into();
        let job_id = Uuid::new_v4().to_string();
        self.active_job_id = Some(job_id.clone());
        self.active_tool = Some(tool.clone());
        self.active_command = command.clone();
        self.active_status = Some(job_status_label(&status).to_string());
        self.jobs.push(PlanJobRecord {
            job_id: job_id.clone(),
            tool,
            command,
            status,
            output_transcript: String::new(),
            output_preview: None,
            has_error_output: false,
        });
        job_id
    }

    pub fn update_active_job_status(&mut self, status: PlanJobStatus) {
        let Some(active_job_id) = self.active_job_id.as_deref() else {
            return;
        };
        if let Some(job) = self
            .jobs
            .iter_mut()
            .rev()
            .find(|job| job.job_id == active_job_id)
        {
            job.status = status.clone();
        }
        self.active_status = Some(job_status_label(&status).to_string());
    }

    pub fn set_active_job_output(
        &mut self,
        output_preview: Option<String>,
        has_error_output: bool,
    ) {
        let Some(active_job_id) = self.active_job_id.as_deref() else {
            return;
        };
        if let Some(job) = self
            .jobs
            .iter_mut()
            .rev()
            .find(|job| job.job_id == active_job_id)
        {
            job.output_preview = output_preview;
            job.has_error_output = has_error_output;
        }
    }

    pub fn append_active_job_output(&mut self, chunk: &str, is_error: bool) {
        const MAX_JOB_TRANSCRIPT_CHARS: usize = 16 * 1024;

        let Some(active_job_id) = self.active_job_id.as_deref() else {
            return;
        };
        if let Some(job) = self
            .jobs
            .iter_mut()
            .rev()
            .find(|job| job.job_id == active_job_id)
        {
            job.output_transcript.push_str(chunk);
            if job.output_transcript.chars().count() > MAX_JOB_TRANSCRIPT_CHARS {
                let trimmed: String = job
                    .output_transcript
                    .chars()
                    .rev()
                    .take(MAX_JOB_TRANSCRIPT_CHARS)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                job.output_transcript = trimmed;
            }
            job.has_error_output |= is_error;
            job.output_preview = preview_text(&job.output_transcript, 512);
        }
    }

    pub fn finish_active_job(&mut self, status: PlanJobStatus) {
        self.update_active_job_status(status);
        self.clear_active_state();
    }

    pub fn clear_active_state(&mut self) {
        self.active_tool = None;
        self.active_command = None;
        self.active_status = None;
        self.active_job_id = None;
    }
}

fn job_status_label(status: &PlanJobStatus) -> &'static str {
    match status {
        PlanJobStatus::WaitingApproval => "waiting_approval",
        PlanJobStatus::Running => "running",
        PlanJobStatus::Blocked => "blocked",
        PlanJobStatus::Succeeded => "succeeded",
        PlanJobStatus::Failed => "failed",
    }
}

fn preview_text(text: &str, max_chars: usize) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let preview: String = trimmed.chars().take(max_chars).collect();
    if trimmed.chars().count() > max_chars {
        Some(format!("{}…", preview))
    } else {
        Some(preview)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlanState {
    pub explanation: Option<String>,
    pub items: Vec<PlanItem>,
    pub runtime: Option<PlanRuntime>,
    pub updated_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{PlanJobStatus, PlanRuntime};

    #[test]
    fn runtime_deserializes_legacy_shape_with_empty_jobs() {
        let runtime: PlanRuntime = serde_json::from_str(
            r#"{
                "active_tool":"run_command",
                "active_command":"echo hi",
                "active_status":"running"
            }"#,
        )
        .expect("legacy runtime json should deserialize");

        assert_eq!(runtime.active_tool.as_deref(), Some("run_command"));
        assert_eq!(runtime.active_command.as_deref(), Some("echo hi"));
        assert_eq!(runtime.active_status.as_deref(), Some("running"));
        assert!(runtime.active_job_id.is_none());
        assert!(runtime.jobs.is_empty());
    }

    #[test]
    fn runtime_tracks_job_lifecycle() {
        let mut runtime = PlanRuntime::default();
        let job_id = runtime.start_job(
            "run_command",
            Some("echo hi".to_string()),
            PlanJobStatus::Running,
        );

        assert_eq!(runtime.active_job_id.as_deref(), Some(job_id.as_str()));
        assert_eq!(runtime.jobs.len(), 1);
        assert_eq!(runtime.active_status.as_deref(), Some("running"));

        runtime.finish_active_job(PlanJobStatus::Succeeded);

        assert!(runtime.active_job_id.is_none());
        assert!(!runtime.has_active_state());
        assert_eq!(runtime.jobs[0].status, PlanJobStatus::Succeeded);
    }
}
