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

use crate::backend::SandboxBackend;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRequest {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env: Vec<(String, String)>,
    pub declared_read_paths: Vec<PathBuf>,
    pub declared_write_paths: Vec<PathBuf>,
    pub requires_network: bool,
}

impl SandboxRequest {
    pub fn new(command: impl Into<String>, args: &[&str]) -> Result<Self> {
        Ok(Self {
            command: command.into(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            working_dir: std::env::current_dir()?,
            env: vec![],
            declared_read_paths: vec![],
            declared_write_paths: vec![],
            requires_network: false,
        })
    }
}

#[derive(Debug)]
pub struct SandboxExecutionResult {
    pub request: SandboxRequest,
    pub backend: SandboxBackend,
    pub output: std::process::Output,
}
