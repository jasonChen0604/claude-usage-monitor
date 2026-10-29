use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir available")
        .join(".claude")
        .join("settings.json")
}

/// The statusLine command: this app's own binary in `--collect` mode (see
/// collect.rs), so it works without `node` on Claude Code's PATH.
fn collector_command() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(format!("\"{}\" --collect", exe.to_string_lossy()))
}

#[derive(Debug, serde::Serialize)]
pub struct StatusLineState {
    pub configured: bool,
    pub existing_command: Option<String>,
}

/// Checks whether Claude Code's statusLine is already pointed at our
/// collector command. Doesn't distinguish "not configured" from "configured
/// for something else" beyond returning the existing command string, since
/// the caller (onboarding UI) needs to show that to the user either way.
#[tauri::command]
pub fn check_statusline() -> StatusLineState {
    let Some(command) = collector_command() else {
        return StatusLineState {
            configured: false,
            existing_command: None,
        };
    };

    let Ok(contents) = fs::read_to_string(claude_settings_path()) else {
        return StatusLineState {
            configured: false,
            existing_command: None,
        };
    };

    let Ok(settings) = serde_json::from_str::<Value>(&contents) else {
        return StatusLineState {
            configured: false,
            existing_command: None,
        };
    };

    let existing_command = settings
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .map(String::from);

    let configured = existing_command
        .as_deref()
        .is_some_and(|cmd| cmd.contains(&command));

    StatusLineState {
        configured,
        existing_command,
    }
}

/// Writes `statusLine.command` in `~/.claude/settings.json` to point at our
/// collector command, preserving every other existing setting. Overwrites
/// any prior `statusLine.command` — the onboarding UI must have already
/// shown the user what was there (via check_statusline) before calling
/// this, since we can't safely merge two arbitrary shell commands.
#[tauri::command]
pub fn install_statusline() -> Result<(), String> {
    let command = collector_command().ok_or("could not resolve app executable path")?;

    let settings_path = claude_settings_path();
    let mut settings: Value = fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    settings["statusLine"] = json!({
        "type": "command",
        "command": command,
        // Without this the snapshot only refreshes on Claude Code activity.
        "refreshInterval": 30,
    });

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &settings_path,
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
