use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir available")
        .join(".claude")
        .join("settings.json")
}

fn collector_script_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .resolve(
            "scripts/claude-statusline-collector.cjs",
            tauri::path::BaseDirectory::Resource,
        )
        .ok()
}

use tauri::Manager;

#[derive(Debug, serde::Serialize)]
pub struct StatusLineState {
    pub configured: bool,
    pub existing_command: Option<String>,
}

/// Checks whether Claude Code's statusLine is already pointed at our
/// collector script. Doesn't distinguish "not configured" from "configured
/// for something else" beyond returning the existing command string, since
/// the caller (onboarding UI) needs to show that to the user either way.
#[tauri::command]
pub fn check_statusline(app: tauri::AppHandle) -> StatusLineState {
    let Some(script_path) = collector_script_path(&app) else {
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
        .is_some_and(|cmd| cmd.contains(&script_path.to_string_lossy().to_string()));

    StatusLineState {
        configured,
        existing_command,
    }
}

/// Writes `statusLine.command` in `~/.claude/settings.json` to point at our
/// collector script, preserving every other existing setting. Overwrites
/// any prior `statusLine.command` — the onboarding UI must have already
/// shown the user what was there (via check_statusline) before calling
/// this, since we can't safely merge two arbitrary shell commands.
#[tauri::command]
pub fn install_statusline(app: tauri::AppHandle) -> Result<(), String> {
    let script_path = collector_script_path(&app)
        .ok_or("could not resolve collector script path")?
        .to_string_lossy()
        .to_string();

    let settings_path = claude_settings_path();
    let mut settings: Value = fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    settings["statusLine"] = json!({
        "type": "command",
        "command": format!("node \"{script_path}\""),
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
