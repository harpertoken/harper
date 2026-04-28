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

use crate::core::plan::PlanState;
use std::sync::OnceLock;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct PlanUpdateEvent {
    pub event_id: i64,
    pub session_id: String,
    pub plan: PlanState,
}

static PLAN_UPDATES: OnceLock<broadcast::Sender<PlanUpdateEvent>> = OnceLock::new();

fn sender() -> &'static broadcast::Sender<PlanUpdateEvent> {
    PLAN_UPDATES.get_or_init(|| {
        let (tx, _rx) = broadcast::channel(256);
        tx
    })
}

pub fn subscribe() -> broadcast::Receiver<PlanUpdateEvent> {
    sender().subscribe()
}

pub fn notify(event_id: i64, session_id: &str, plan: PlanState) {
    let _ = sender().send(PlanUpdateEvent {
        event_id,
        session_id: session_id.to_string(),
        plan,
    });
}

#[cfg(test)]
mod tests {
    use super::{notify, subscribe};
    use crate::core::plan::PlanState;

    #[tokio::test]
    async fn broadcasts_plan_updates() {
        let mut rx = subscribe();
        let plan = PlanState::default();

        notify(42, "session-a", plan.clone());

        let event = rx.recv().await.expect("event");
        assert_eq!(event.event_id, 42);
        assert_eq!(event.session_id, "session-a");
        assert_eq!(event.plan, plan);
    }
}
