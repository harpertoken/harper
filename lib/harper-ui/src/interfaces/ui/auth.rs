use harper_core::{AuthSession, SessionStateView};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TuiAuthCommand {
    Login { provider: String },
    Logout,
    Status,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TuiAuthStartResponse {
    pub flow_id: String,
    pub login_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TuiAuthPollResponse {
    pub status: String,
    pub session: Option<AuthSession>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteSessionListItem {
    pub id: String,
    pub user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAuthSession {
    session: AuthSession,
}

pub fn parse_tui_auth_command(input: &str) -> Option<TuiAuthCommand> {
    let trimmed = input.trim();
    let parts: Vec<_> = trimmed.split_whitespace().collect();
    match parts.as_slice() {
        ["/auth", "login", provider] => Some(TuiAuthCommand::Login {
            provider: provider.to_string(),
        }),
        ["/auth", "logout"] => Some(TuiAuthCommand::Logout),
        ["/auth", "status"] => Some(TuiAuthCommand::Status),
        _ => None,
    }
}

pub async fn start_tui_auth_flow(
    client: &reqwest::Client,
    server_base_url: &str,
    provider: &str,
) -> Result<TuiAuthStartResponse, String> {
    let url = format!(
        "{}/auth/tui/start/{}",
        server_base_url.trim_end_matches('/'),
        provider
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(response.text().await.unwrap_or_default());
    }
    response.json().await.map_err(|err| err.to_string())
}

pub async fn poll_tui_auth_flow(
    client: &reqwest::Client,
    server_base_url: &str,
    flow_id: &str,
) -> Result<TuiAuthPollResponse, String> {
    let url = format!(
        "{}/auth/tui/flow/{}",
        server_base_url.trim_end_matches('/'),
        flow_id
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(response.text().await.unwrap_or_default());
    }
    response.json().await.map_err(|err| err.to_string())
}

pub async fn fetch_remote_sessions(
    client: &reqwest::Client,
    server_base_url: &str,
    session: &AuthSession,
) -> Result<Vec<RemoteSessionListItem>, String> {
    let url = format!("{}/api/sessions", server_base_url.trim_end_matches('/'));
    let response = client
        .get(&url)
        .bearer_auth(&session.access_token)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(response.text().await.unwrap_or_default());
    }
    response.json().await.map_err(|err| err.to_string())
}

pub async fn fetch_remote_session_state(
    client: &reqwest::Client,
    server_base_url: &str,
    session: &AuthSession,
    session_id: &str,
) -> Result<SessionStateView, String> {
    let url = format!(
        "{}/api/sessions/{}",
        server_base_url.trim_end_matches('/'),
        session_id
    );
    let response = client
        .get(&url)
        .bearer_auth(&session.access_token)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(response.text().await.unwrap_or_default());
    }
    response.json().await.map_err(|err| err.to_string())
}

pub fn load_auth_session() -> Option<AuthSession> {
    let path = auth_session_path()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<StoredAuthSession>(&content)
        .ok()
        .map(|stored| stored.session)
}

pub fn save_auth_session(session: &AuthSession) -> Result<(), String> {
    let path = auth_session_path().ok_or_else(|| "Home directory not found".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_string_pretty(&StoredAuthSession {
        session: session.clone(),
    })
    .map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| err.to_string())
}

pub fn clear_auth_session() -> Result<(), String> {
    let Some(path) = auth_session_path() else {
        return Ok(());
    };
    if path.exists() {
        fs::remove_file(path).map_err(|err| err.to_string())?;
    }
    Ok(())
}

pub fn launch_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(url);
        cmd
    };

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(url);
        cmd
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", "", url]);
        cmd
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("Browser launch is not supported on this platform".to_string());

    cmd.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

fn auth_session_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)?;
    Some(home.join(".harper").join("auth").join("tui-session.json"))
}

#[cfg(test)]
mod tests {
    use super::{parse_tui_auth_command, TuiAuthCommand};

    #[test]
    fn parses_tui_auth_login_command() {
        let command = parse_tui_auth_command("/auth login github").expect("parse command");
        match command {
            TuiAuthCommand::Login { provider } => assert_eq!(provider, "github"),
            _ => panic!("expected login command"),
        }
    }
}
