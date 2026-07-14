use serde::{Deserialize, Serialize};

/// One selectable piece of info in the tray title. Each is independently
/// toggleable so a user can show e.g. both 5h% and its countdown at once.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrayItem {
    FiveHourPercentage,
    FiveHourCountdown,
    WeeklyPercentage,
    WeeklyCountdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Minutes between re-reads of the snapshot directory.
    pub poll_interval_minutes: u32,
    /// Which items to show in the tray title, in display order.
    pub tray_items: Vec<TrayItem>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_interval_minutes: 1,
            tray_items: vec![TrayItem::FiveHourPercentage, TrayItem::WeeklyPercentage],
        }
    }
}
