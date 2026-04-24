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
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Sandbox not available: {0}")]
    NotAvailable(String),
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SandboxError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub allowed_dirs: Vec<PathBuf>,
    pub allowed_commands: Option<Vec<String>>,
    pub blocked_commands: Option<Vec<String>>,
    pub readonly_home: bool,
    pub network_access: bool,
    pub max_execution_time_secs: Option<u64>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_dirs: vec![],
            allowed_commands: None,
            blocked_commands: None,
            readonly_home: true,
            network_access: false,
            max_execution_time_secs: Some(30),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxBackend {
    Bubblewrap,
    SandboxExec,
    None,
}

/// Sandbox execution environment for secure command running
///
/// Provides isolated execution of system commands with configurable
/// restrictions on directories, commands, and network access.
pub struct Sandbox {
    config: SandboxConfig,
    backend: SandboxBackend,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration
    ///
    /// Automatically detects the available backend (Bubblewrap, SandboxExec, or None).
    ///
    /// # Arguments
    /// * `config` - Sandbox configuration settings
    ///
    /// # Example
    /// ```
    /// let config = SandboxConfig::default();
    /// let sandbox = Sandbox::new(config);
    /// ```
    pub fn new(config: SandboxConfig) -> Self {
        let backend = Self::detect_backend();
        Self { config, backend }
    }

    fn detect_backend() -> SandboxBackend {
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

    /// Check if sandbox execution is available on this system
    ///
    /// Returns true if a supported sandbox backend is detected and available.
    ///
    /// # Returns
    /// true if sandboxing is supported, false otherwise
    pub fn is_available(&self) -> bool {
        self.backend != SandboxBackend::None
    }

    /// Get the name of the detected sandbox backend
    ///
    /// # Returns
    /// A string describing the backend: "bubblewrap (bwrap)", "sandbox-exec (macOS)", or "none"
    pub fn backend_name(&self) -> &str {
        match self.backend {
            SandboxBackend::Bubblewrap => "bubblewrap (bwrap)",
            SandboxBackend::SandboxExec => "sandbox-exec (macOS)",
            SandboxBackend::None => "none",
        }
    }

    /// Execute a command in the sandbox environment
    ///
    /// Runs the specified command with arguments, applying sandbox restrictions
    /// based on the configuration. If sandboxing is disabled, executes directly.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `args` - Command arguments as string slices
    ///
    /// # Returns
    /// The command output including stdout, stderr, and exit status
    ///
    /// # Errors
    /// Returns `SandboxError` for execution failures, timeouts, or unavailable sandbox
    ///
    /// # Example
    /// ```
    /// let output = sandbox.execute("echo", &["hello", "world"]).await?;
    /// println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    /// ```
    pub async fn execute(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        if !self.config.enabled {
            return self.execute_direct(command, args).await;
        }

        match self.backend {
            SandboxBackend::Bubblewrap => self.execute_bwrap(command, args).await,
            SandboxBackend::SandboxExec => self.execute_sandbox_exec(command, args).await,
            SandboxBackend::None => {
                log::warn!("Sandbox disabled - executing directly");
                self.execute_direct(command, args).await
            }
        }
    }

    async fn execute_direct(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        let mut cmd = Command::new(command);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(timeout) = self.config.max_execution_time_secs {
            match tokio::time::timeout(std::time::Duration::from_secs(timeout), cmd.output()).await
            {
                Ok(result) => result.map_err(|e| SandboxError::ExecutionFailed(e.to_string())),
                Err(_) => Err(SandboxError::ExecutionFailed(format!(
                    "Command timed out after {} seconds",
                    timeout
                ))),
            }
        } else {
            cmd.output()
                .await
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))
        }
    }

    #[cfg(target_os = "linux")]
    async fn execute_bwrap(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        let mut bwrap_args = vec!["--unshare-pid".to_string()];

        if !self.config.network_access {
            bwrap_args.push("--unshare-net".to_string());
        }

        bpush(&mut bwrap_args, "--ro-bind", "/usr".to_string());
        bpush(&mut bwrap_args, "--ro-bind", "/bin".to_string());
        bpush(&mut bwrap_args, "--ro-bind", "/lib".to_string());
        bpush(&mut bwrap_args, "--ro-bind", "/lib64".to_string());

        for dir in &self.config.allowed_dirs {
            if dir.exists() {
                bpush(&mut bwrap_args, "--ro-bind", dir.display().to_string());
            }
        }

        if self.config.readonly_home {
            if let Ok(home) = std::env::var("HOME") {
                bpush(&mut bwrap_args, "--ro-bind", home);
            }
        } else if let Ok(home) = std::env::var("HOME") {
            bpush(&mut bwrap_args, "--ro-bind", home);
        }

        bpush(
            &mut bwrap_args,
            "--chdir",
            std::env::current_dir()?.display().to_string(),
        );

        bwrap_args.push("--".to_string());
        bwrap_args.push(command.to_string());
        bwrap_args.extend(args.iter().map(|s| s.to_string()));

        log::debug!("Executing with bwrap: {:?}", bwrap_args);

        let mut cmd = Command::new("bwrap");
        cmd.args(&bwrap_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(timeout) = self.config.max_execution_time_secs {
            match tokio::time::timeout(std::time::Duration::from_secs(timeout), cmd.output()).await
            {
                Ok(result) => result.map_err(|e| SandboxError::ExecutionFailed(e.to_string())),
                Err(_) => Err(SandboxError::ExecutionFailed(format!(
                    "Command timed out after {} seconds",
                    timeout
                ))),
            }
        } else {
            cmd.output()
                .await
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))
        }
    }

    #[cfg(not(target_os = "linux"))]
    async fn execute_bwrap(&self, _command: &str, _args: &[&str]) -> Result<std::process::Output> {
        Err(SandboxError::NotAvailable(
            "Bubblewrap only available on Linux".to_string(),
        ))
    }

    #[cfg(target_os = "macos")]
    async fn execute_sandbox_exec(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<std::process::Output> {
        let mut sandbox_rules = vec!["(version 1)".to_string()];

        if !self.config.network_access {
            sandbox_rules.push("(deny network*)\n(allow default)".to_string());
        }

        sandbox_rules.push("(allow process-exec)".to_string());

        for dir in &self.config.allowed_dirs {
            if dir.exists() {
                sandbox_rules.push(format!(
                    "(allow file-read* (subpath \"{}\"))",
                    dir.display()
                ));
            }
        }

        let sandbox_profile = sandbox_rules.join("\n");

        let mut cmd = Command::new("sandbox-exec");
        cmd.arg("-p")
            .arg(&sandbox_profile)
            .arg(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(timeout) = self.config.max_execution_time_secs {
            match tokio::time::timeout(std::time::Duration::from_secs(timeout), cmd.output()).await
            {
                Ok(result) => result.map_err(|e| SandboxError::ExecutionFailed(e.to_string())),
                Err(_) => Err(SandboxError::ExecutionFailed(format!(
                    "Command timed out after {} seconds",
                    timeout
                ))),
            }
        } else {
            cmd.output()
                .await
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))
        }
    }

    #[cfg(not(target_os = "macos"))]
    async fn execute_sandbox_exec(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        Err(SandboxError::NotAvailable(
            "sandbox-exec only available on macOS".to_string(),
        ))
    }

    /// Check if a command is allowed to run based on configured restrictions
    ///
    /// Commands are allowed if they match the allowed list (if specified) and
    /// don't match the blocked list.
    ///
    /// # Arguments
    /// * `command` - The command string to check
    ///
    /// # Returns
    /// true if the command is permitted, false otherwise
    ///
    /// # Example
    /// ```
    /// if sandbox.is_command_allowed("ls") {
    ///     println!("ls command is allowed");
    /// }
    /// ```
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if let Some(allowed) = &self.config.allowed_commands {
            if !allowed.is_empty() && allowed.iter().any(|c| command.contains(c)) {
                return true;
            }
        }
        if let Some(blocked) = &self.config.blocked_commands {
            if blocked.iter().any(|b| command.contains(b)) {
                return false;
            }
        }
        self.config.allowed_commands.is_none()
    }
}

#[allow(dead_code)]
fn bpush(args: &mut Vec<String>, flag: &str, val: impl Into<String>) {
    args.push(flag.to_string());
    args.push(val.into());
}
// }
#[allow(dead_code)]
mod config {
    pub use super::SandboxConfig;

    use super::*;

    pub fn from_env() -> SandboxConfig {
        let enabled = std::env::var("HARPER_SANDBOX_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false);

        let allowed_dirs: Vec<PathBuf> = std::env::var("HARPER_SANDBOX_ALLOWED_DIRS")
            .map(|v| v.split(':').map(PathBuf::from).collect())
            .unwrap_or_default();

        let allowed_commands: Option<Vec<String>> =
            std::env::var("HARPER_SANDBOX_ALLOWED_COMMANDS")
                .map(|v| v.split(':').map(String::from).collect())
                .ok();

        let blocked_commands: Option<Vec<String>> =
            std::env::var("HARPER_SANDBOX_BLOCKED_COMMANDS")
                .map(|v| v.split(':').map(String::from).collect())
                .ok();

        let network_access = std::env::var("HARPER_SANDBOX_NETWORK")
            .map(|v| v == "true")
            .unwrap_or(false);

        let readonly_home = std::env::var("HARPER_SANDBOX_READONLY_HOME")
            .map(|v| v != "false")
            .unwrap_or(true);

        SandboxConfig {
            enabled,
            allowed_dirs,
            allowed_commands,
            blocked_commands,
            network_access,
            readonly_home,
            max_execution_time_secs: Some(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert!(!config.enabled);
        assert!(config.readonly_home);
        assert!(!config.network_access);
        assert_eq!(config.max_execution_time_secs, Some(30));
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("HARPER_SANDBOX_ENABLED", "true");
        let config = config::from_env();
        assert!(config.enabled);
        std::env::remove_var("HARPER_SANDBOX_ENABLED");
    }

    #[test]
    fn test_sandbox_new() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config);
        // Backend detection depends on platform, but new() should work
        assert!(matches!(
            sandbox.backend,
            SandboxBackend::Bubblewrap | SandboxBackend::SandboxExec | SandboxBackend::None
        ));
    }

    #[test]
    fn test_backend_name() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config);

        let name = sandbox.backend_name();
        match sandbox.backend {
            SandboxBackend::Bubblewrap => assert_eq!(name, "bubblewrap (bwrap)"),
            SandboxBackend::SandboxExec => assert_eq!(name, "sandbox-exec (macOS)"),
            SandboxBackend::None => assert_eq!(name, "none"),
        }
    }

    #[test]
    fn test_is_available() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config);
        // Availability depends on backend detection
        let available = sandbox.is_available();
        assert_eq!(available, sandbox.backend != SandboxBackend::None);
    }

    #[test]
    fn test_is_command_allowed_no_restrictions() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config);

        assert!(sandbox.is_command_allowed("ls"));
        assert!(sandbox.is_command_allowed("echo hello"));
        assert!(sandbox.is_command_allowed("rm -rf /"));
    }

    #[test]
    fn test_is_command_allowed_with_blocked() {
        let config = SandboxConfig {
            blocked_commands: Some(vec!["rm".to_string(), "sudo".to_string()]),
            ..Default::default()
        };
        let sandbox = Sandbox::new(config);

        assert!(sandbox.is_command_allowed("ls"));
        assert!(!sandbox.is_command_allowed("rm -rf /"));
        assert!(!sandbox.is_command_allowed("sudo apt update"));
    }

    #[test]
    fn test_is_command_allowed_with_allowed() {
        let config = SandboxConfig {
            allowed_commands: Some(vec!["ls".to_string(), "cat".to_string()]),
            ..Default::default()
        };
        let sandbox = Sandbox::new(config);

        assert!(sandbox.is_command_allowed("ls"));
        assert!(sandbox.is_command_allowed("cat file.txt"));
        assert!(!sandbox.is_command_allowed("rm -rf /"));
    }
}
