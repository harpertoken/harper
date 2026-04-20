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

pub struct Sandbox {
    config: SandboxConfig,
    backend: SandboxBackend,
}

impl Sandbox {
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

    pub fn is_available(&self) -> bool {
        self.backend != SandboxBackend::None
    }

    pub fn backend_name(&self) -> &str {
        match self.backend {
            SandboxBackend::Bubblewrap => "bubblewrap (bwrap)",
            SandboxBackend::SandboxExec => "sandbox-exec (macOS)",
            SandboxBackend::None => "none",
        }
    }

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

    pub fn is_command_allowed(&self, command: &str) -> bool {
        if let Some(allowed) = &self.config.allowed_commands {
            if !allowed.is_empty() {
                return allowed.iter().any(|c| command.contains(c));
            }
        }
        if let Some(blocked) = &self.config.blocked_commands {
            for b in blocked {
                if command.contains(b) {
                    return false;
                }
            }
        }
        true
    }
}

#[allow(dead_code)]
fn bpush(args: &mut Vec<String>, flag: &str, val: impl Into<String>) {
    args.push(flag.to_string());
    args.push(val.into());
}

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
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("HARPER_SANDBOX_ENABLED", "true");
        let config = config::from_env();
        assert!(config.enabled);
    }
}
