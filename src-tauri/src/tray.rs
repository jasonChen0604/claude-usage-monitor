use crate::settings::{Settings, TrayItem};
use crate::snapshot::{UsageSnapshot, UsageWindow};
use chrono::{DateTime, Utc};

fn format_percentage(window: &UsageWindow) -> String {
    format!("{}%", window.used_percentage.round() as i64)
}

fn format_countdown(window: &UsageWindow) -> String {
    match window.resets_at.parse::<DateTime<Utc>>() {
        Ok(resets_at) => {
            let remaining = resets_at - Utc::now();
            if remaining.num_seconds() <= 0 {
                "0m".to_string()
            } else {
                let hours = remaining.num_hours();
                let minutes = remaining.num_minutes() % 60;
                if hours > 0 {
                    format!("{hours}h{minutes}m")
                } else {
                    format!("{minutes}m")
                }
            }
        }
        Err(_) => "?".to_string(),
    }
}

/// Builds the tray title text from the Claude snapshot (if present) and the
/// user's selected tray items, in the order the user configured them.
/// Other providers' snapshots are read by the popover UI but don't
/// currently have tray real estate of their own — ponytail: single-provider
/// tray text for now, revisit layout when a second provider ships.
pub fn tray_title(snapshots: &[UsageSnapshot], settings: &Settings) -> String {
    let Some(claude) = snapshots.iter().find(|s| s.provider == "claude") else {
        return "–".to_string();
    };

    let five_hour = claude.windows.iter().find(|w| w.label == "5h");
    let weekly = claude.windows.iter().find(|w| w.label == "7d");

    let parts: Vec<String> = settings
        .tray_items
        .iter()
        .filter_map(|item| match item {
            TrayItem::FiveHourPercentage => five_hour.map(format_percentage),
            TrayItem::FiveHourCountdown => five_hour.map(format_countdown),
            TrayItem::WeeklyPercentage => weekly.map(format_percentage),
            TrayItem::WeeklyCountdown => weekly.map(format_countdown),
        })
        .collect();

    if parts.is_empty() {
        "–".to_string()
    } else {
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(label: &str, pct: f64, resets_in_minutes: i64) -> UsageWindow {
        UsageWindow {
            label: label.to_string(),
            used_percentage: pct,
            resets_at: (Utc::now() + chrono::Duration::minutes(resets_in_minutes)).to_rfc3339(),
        }
    }

    fn claude_snapshot(windows: Vec<UsageWindow>) -> UsageSnapshot {
        UsageSnapshot {
            provider: "claude".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            windows,
        }
    }

    #[test]
    fn formats_percentage() {
        let w = window("5h", 42.0, 60);
        assert_eq!(format_percentage(&w), "42%");
    }

    #[test]
    fn formats_countdown() {
        let w = window("5h", 42.0, 130);
        let text = format_countdown(&w);
        assert!(
            text == "2h10m" || text == "2h9m",
            "expected ~2h10m, got {text}"
        );
    }

    #[test]
    fn tray_title_no_claude_snapshot() {
        assert_eq!(tray_title(&[], &Settings::default()), "–");
    }

    #[test]
    fn tray_title_joins_default_items() {
        let snap = claude_snapshot(vec![window("5h", 42.0, 60), window("7d", 18.0, 60)]);
        assert_eq!(tray_title(&[snap], &Settings::default()), "42% · 18%");
    }

    #[test]
    fn tray_title_respects_custom_selection_and_order() {
        let snap = claude_snapshot(vec![window("5h", 42.0, 130), window("7d", 18.0, 60)]);
        let settings = Settings {
            poll_interval_minutes: 10,
            tray_items: vec![TrayItem::WeeklyPercentage, TrayItem::FiveHourCountdown],
        };
        let text = tray_title(&[snap], &settings);
        assert!(text.starts_with("18% · 2h"), "got {text}");
    }

    #[test]
    fn tray_title_skips_missing_window() {
        let snap = claude_snapshot(vec![window("5h", 42.0, 60)]);
        let settings = Settings {
            poll_interval_minutes: 10,
            tray_items: vec![TrayItem::WeeklyPercentage, TrayItem::FiveHourPercentage],
        };
        assert_eq!(tray_title(&[snap], &settings), "42%");
    }
}
