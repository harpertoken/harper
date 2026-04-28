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

use crate::errors::{Result, SandboxError};
use crate::policy::SandboxConfig;
use crate::request::SandboxRequest;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxBackend {
    Bubblewrap,
    SandboxExec,
    None,
}

pub fn detect_backend() -> SandboxBackend {
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/usr/bin/bwrap").exists()
            || std::path::Path::new("/bin/bwrap").exists()
        {
            return SandboxBackend::Bubblewrap;
        }
    }
    #[cfg(target_os = "macos")]
    {
        return SandboxBackend::SandboxExec;
    }
    #[allow(unreachable_code)]
    SandboxBackend::None
}

pub fn backend_name(backend: SandboxBackend) -> &'static str {
    match backend {
        SandboxBackend::Bubblewrap => "bubblewrap (bwrap)",
        SandboxBackend::SandboxExec => "sandbox-exec (macOS)",
        SandboxBackend::None => "none",
    }
}

pub async fn execute_with_backend_streaming<C>(
    backend: SandboxBackend,
    config: &SandboxConfig,
    request: &SandboxRequest,
    on_output: C,
) -> Result<std::process::Output>
where
    C: FnMut(String, bool) + Send + 'static,
{
    match backend {
        SandboxBackend::None => execute_direct(config, request, on_output).await,
        SandboxBackend::Bubblewrap => execute_bwrap(config, request, on_output).await,
        SandboxBackend::SandboxExec => execute_sandbox_exec(config, request, on_output).await,
    }
}

async fn execute_direct(
    config: &SandboxConfig,
    request: &SandboxRequest,
    on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    let mut cmd = Command::new(&request.command);
    cmd.args(&request.args)
        .current_dir(&request.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in &request.env {
        cmd.env(key, value);
    }

    run_command_with_timeout_streaming(config, cmd, on_output).await
}

#[cfg(target_os = "linux")]
async fn execute_bwrap(
    config: &SandboxConfig,
    request: &SandboxRequest,
    on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    let mut bwrap_args = vec!["--unshare-pid".to_string()];

    if !config.network_access {
        bwrap_args.push("--unshare-net".to_string());
    }

    bpush(&mut bwrap_args, "--ro-bind", "/usr".to_string());
    bpush(&mut bwrap_args, "--ro-bind", "/bin".to_string());
    bpush(&mut bwrap_args, "--ro-bind", "/lib".to_string());
    bpush(&mut bwrap_args, "--ro-bind", "/lib64".to_string());

    for dir in &config.allowed_dirs {
        if dir.exists() {
            bpush(&mut bwrap_args, "--ro-bind", dir.display().to_string());
        }
    }
    for dir in &config.writable_dirs {
        if dir.exists() {
            bpush(&mut bwrap_args, "--bind", dir.display().to_string());
        }
    }

    if config.readonly_home {
        if let Ok(home) = std::env::var("HOME") {
            bpush(&mut bwrap_args, "--ro-bind", home);
        }
    } else if let Ok(home) = std::env::var("HOME") {
        bpush(&mut bwrap_args, "--ro-bind", home);
    }

    bpush(
        &mut bwrap_args,
        "--chdir",
        request.working_dir.display().to_string(),
    );

    bwrap_args.push("--".to_string());
    bwrap_args.push(request.command.clone());
    bwrap_args.extend(request.args.iter().cloned());

    let mut cmd = Command::new("bwrap");
    cmd.args(&bwrap_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in &request.env {
        cmd.env(key, value);
    }
    run_command_with_timeout_streaming(config, cmd, on_output).await
}

#[cfg(not(target_os = "linux"))]
async fn execute_bwrap(
    _config: &SandboxConfig,
    _request: &SandboxRequest,
    _on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    Err(SandboxError::BackendUnavailable(
        "Bubblewrap only available on Linux".to_string(),
    ))
}

#[cfg(target_os = "macos")]
async fn execute_sandbox_exec(
    config: &SandboxConfig,
    request: &SandboxRequest,
    on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    let mut sandbox_rules = vec!["(version 1)".to_string()];

    if !config.network_access {
        sandbox_rules.push("(deny network*)\n(allow default)".to_string());
    }

    sandbox_rules.push("(allow process-exec)".to_string());

    for dir in &config.allowed_dirs {
        if dir.exists() {
            sandbox_rules.push(format!(
                "(allow file-read* (subpath \"{}\"))",
                dir.display()
            ));
        }
    }
    for dir in &config.writable_dirs {
        if dir.exists() {
            sandbox_rules.push(format!(
                "(allow file-write* (subpath \"{}\"))",
                dir.display()
            ));
        }
    }

    let sandbox_profile = sandbox_rules.join("\n");

    let mut cmd = Command::new("sandbox-exec");
    cmd.arg("-p")
        .arg(&sandbox_profile)
        .arg(&request.command)
        .args(&request.args)
        .current_dir(&request.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in &request.env {
        cmd.env(key, value);
    }
    run_command_with_timeout_streaming(config, cmd, on_output).await
}

#[cfg(not(target_os = "macos"))]
async fn execute_sandbox_exec(
    _config: &SandboxConfig,
    _request: &SandboxRequest,
    _on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    Err(SandboxError::BackendUnavailable(
        "sandbox-exec only available on macOS".to_string(),
    ))
}

async fn run_command_with_timeout_streaming(
    config: &SandboxConfig,
    mut cmd: Command,
    on_output: impl FnMut(String, bool) + Send + 'static,
) -> Result<std::process::Output> {
    let operation = async move {
        let mut child = cmd
            .spawn()
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let (tx, mut rx) = mpsc::unbounded_channel::<(String, bool)>();

        let stdout_task = tokio::spawn(read_stream(stdout, false, tx.clone()));
        let stderr_task = tokio::spawn(read_stream(stderr, true, tx));
        let consumer_task = tokio::spawn(async move {
            let mut on_output = on_output;
            while let Some((chunk, is_error)) = rx.recv().await {
                on_output(chunk, is_error);
            }
        });

        let status = child
            .wait()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;
        let stdout = stdout_task
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))??;
        let stderr = stderr_task
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))??;
        consumer_task
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    };

    if let Some(timeout) = config.max_execution_time_secs {
        match tokio::time::timeout(std::time::Duration::from_secs(timeout), operation).await {
            Ok(result) => result,
            Err(_) => Err(SandboxError::Timeout {
                timeout_secs: timeout,
            }),
        }
    } else {
        operation.await
    }
}

async fn read_stream(
    stream: Option<impl tokio::io::AsyncRead + Unpin>,
    is_error: bool,
    tx: mpsc::UnboundedSender<(String, bool)>,
) -> Result<Vec<u8>> {
    let Some(stream) = stream else {
        return Ok(Vec::new());
    };

    let mut reader = BufReader::new(stream);
    let mut all = Vec::new();
    loop {
        let mut buf = Vec::new();
        let read = reader
            .read_until(b'\n', &mut buf)
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;
        if read == 0 {
            break;
        }
        all.extend_from_slice(&buf);
        let chunk = String::from_utf8_lossy(&buf).into_owned();
        let _ = tx.send((chunk, is_error));
    }
    Ok(all)
}

#[cfg(target_os = "linux")]
fn bpush(args: &mut Vec<String>, flag: &str, val: impl Into<String>) {
    args.push(flag.to_string());
    args.push(val.into());
}
