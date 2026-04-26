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

use crate::core::error::{HarperError, HarperResult};
use crate::core::plan::{PlanItem, PlanRuntime, PlanState, PlanStepStatus};
use rusqlite::Connection;

pub fn update_plan(
    conn: &Connection,
    session_id: &str,
    args: &serde_json::Value,
) -> HarperResult<String> {
    let items_value = args
        .get("items")
        .or_else(|| args.get("plan"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            HarperError::Validation("update_plan requires an items array".to_string())
        })?;

    if items_value.is_empty() {
        return Err(HarperError::Validation(
            "update_plan requires at least one plan item".to_string(),
        ));
    }

    let mut items = Vec::with_capacity(items_value.len());
    for raw_item in items_value {
        let step = raw_item
            .get("step")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                HarperError::Validation("each plan item requires a non-empty step".to_string())
            })?;
        let status = parse_status(raw_item.get("status"))?;
        items.push(PlanItem {
            step: step.to_string(),
            status,
        });
    }

    let explanation = args
        .get("explanation")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let plan = PlanState {
        explanation: explanation.clone(),
        items,
        runtime: None,
        updated_at: None,
    };

    crate::memory::storage::save_plan_state(conn, session_id, &plan)?;

    let in_progress = plan
        .items
        .iter()
        .filter(|item| item.status == PlanStepStatus::InProgress)
        .count();
    let completed = plan
        .items
        .iter()
        .filter(|item| item.status == PlanStepStatus::Completed)
        .count();

    let mut summary = format!(
        "Plan updated: {} steps ({} completed, {} in progress).",
        plan.items.len(),
        completed,
        in_progress
    );
    if let Some(explanation) = explanation {
        summary.push_str(&format!(" {}", explanation));
    }

    Ok(summary)
}

pub fn set_plan_runtime(
    conn: &Connection,
    session_id: &str,
    runtime: Option<PlanRuntime>,
) -> HarperResult<()> {
    let Some(mut plan) = crate::memory::storage::load_plan_state(conn, session_id)? else {
        return Ok(());
    };
    plan.runtime = runtime;
    crate::memory::storage::save_plan_state(conn, session_id, &plan)
}

fn parse_status(value: Option<&serde_json::Value>) -> HarperResult<PlanStepStatus> {
    let raw = value
        .and_then(|status| status.as_str())
        .unwrap_or("pending")
        .trim()
        .to_ascii_lowercase();

    match raw.as_str() {
        "pending" => Ok(PlanStepStatus::Pending),
        "in_progress" | "in-progress" | "in progress" => Ok(PlanStepStatus::InProgress),
        "completed" | "done" => Ok(PlanStepStatus::Completed),
        _ => Err(HarperError::Validation(format!(
            "invalid plan status '{}'",
            raw
        ))),
    }
}
