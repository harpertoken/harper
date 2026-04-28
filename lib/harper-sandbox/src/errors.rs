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

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Sandbox backend not available: {0}")]
    BackendUnavailable(String),
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Command blocked by sandbox policy: {command}")]
    CommandBlocked { command: String },
    #[error("Path blocked by sandbox policy: {path}")]
    PathBlocked { path: PathBuf },
    #[error("Network blocked by sandbox policy")]
    NetworkBlocked,
    #[error("Command timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SandboxError>;
