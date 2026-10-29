//! `--collect` mode: the app binary doubles as Claude Code's statusLine
//! command, so collecting usage doesn't depend on `node` being on the PATH
//! of whatever process Claude Code runs it from. Reads the statusLine JSON
//! from stdin, writes a `UsageSnapshot` for the "claude" provider, and
//! prints the statusline text. See docs/data-source.md.

use crate::snapshot::{snapshot_dir, UsageSnapshot, UsageWindow};
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};

fn to_iso(unix_seconds: i64) -> String {
    DateTime::<Utc>::from_timestamp(unix_seconds, 0)
        .unwrap_or_default()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn build_snapshot(rate_limits: &Value) -> UsageSnapshot {
    let windows = [("five_hour", "5h"), ("seven_day", "7d")]
        .into_iter()
        .filter_map(|(key, label)| {
            let w = rate_limits.get(key)?;
            Some(UsageWindow {
                label: label.to_string(),
                used_percentage: w.get("used_percentage")?.as_f64()?,
                resets_at: to_iso(w.get("resets_at")?.as_i64()?),
            })
        })
        .collect();
    UsageSnapshot {
        provider: "claude".to_string(),
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        windows,
    }
}

pub fn run() {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let mut stdout = std::io::stdout();

    let Ok(payload) = serde_json::from_str::<Value>(&input) else {
        // Not JSON we understand — forward input untouched and exit quietly.
        let _ = stdout.write_all(input.as_bytes());
        return;
    };

    if let Some(rate_limits) = payload.get("rate_limits") {
        let dir = snapshot_dir();
        let result = fs::create_dir_all(&dir).and_then(|_| {
            let json = serde_json::to_string_pretty(&build_snapshot(rate_limits))?;
            fs::write(dir.join("claude.json"), json)
        });
        if let Err(err) = result {
            eprintln!("claude-usage-monitor: failed to write snapshot: {err}");
        }
    }

    // Whatever this prints IS the statusline text (Claude Code has no
    // separate output field), so print the model name rather than nothing.
    let model = payload["model"]["display_name"].as_str().unwrap_or("");
    let _ = stdout.write_all(model.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_both_windows_with_iso_resets() {
        let snap = build_snapshot(&json!({
            "five_hour": { "used_percentage": 42, "resets_at": 1742651200 },
            "seven_day": { "used_percentage": 18.5, "resets_at": 1743120000 }
        }));
        assert_eq!(snap.provider, "claude");
        assert_eq!(snap.windows.len(), 2);
        assert_eq!(snap.windows[0].label, "5h");
        assert_eq!(snap.windows[0].used_percentage, 42.0);
        assert_eq!(snap.windows[0].resets_at, "2025-03-22T13:46:40.000Z");
        assert_eq!(snap.windows[1].used_percentage, 18.5);
    }

    #[test]
    fn skips_missing_window() {
        let snap = build_snapshot(&json!({
            "five_hour": { "used_percentage": 10, "resets_at": 1742651200 }
        }));
        assert_eq!(snap.windows.len(), 1);
    }
}
