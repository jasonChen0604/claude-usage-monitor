use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// One rate-limit window (e.g. "5h" or "7d") for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percentage: f64,
    pub resets_at: String,
}

/// A provider's usage snapshot, as written by its collector script.
/// See docs/usage-snapshot-schema.md — this is the shared contract that
/// keeps the app provider-agnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub provider: String,
    pub updated_at: String,
    pub windows: Vec<UsageWindow>,
}

pub fn snapshot_dir() -> PathBuf {
    dirs::data_dir()
        .expect("no data dir available")
        .join("claude-usage-monitor")
        .join("snapshots")
}

const APP_GROUP_ID: &str = "group.dev.jasonchen.claude-usage-monitor";

/// The macOS App Group shared container, where the WidgetKit extension
/// reads from (see widget/README.md). Only meaningful once the app is
/// codesigned with that App Group entitlement — in dev builds this path
/// exists but the widget extension isn't there to read it.
fn app_group_dir() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir available")
        .join("Library")
        .join("Group Containers")
        .join(APP_GROUP_ID)
        .join("snapshots")
}

/// Mirrors every current snapshot into the App Group container so the
/// widget extension can read them. Best-effort: failures here shouldn't
/// break the tray, which is why this only logs to stderr.
pub fn mirror_to_app_group(snapshots: &[UsageSnapshot]) {
    let dir = app_group_dir();
    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("claude-usage-monitor: failed to create app group dir: {err}");
        return;
    }
    for snapshot in snapshots {
        let path = dir.join(format!("{}.json", snapshot.provider));
        if let Ok(contents) = serde_json::to_string_pretty(snapshot) {
            if let Err(err) = fs::write(&path, contents) {
                eprintln!("claude-usage-monitor: failed to mirror snapshot: {err}");
            }
        }
    }
}

/// Reads every `*.json` file in the snapshot directory. Unreadable or
/// malformed files are skipped rather than failing the whole read, since a
/// collector script for one provider shouldn't be able to break others.
pub fn read_all_snapshots() -> Vec<UsageSnapshot> {
    let dir = snapshot_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| fs::read_to_string(e.path()).ok())
        .filter_map(|contents| serde_json::from_str(&contents).ok())
        .collect()
}
