//! Defines the application's state.

use super::i18n::{self, Translations};

/// Enum representing events triggered by the tray menu.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Event {
    ShowMenu,
    ToggleActive,
    SetDuration(DurationOption),
    Exit,
    NoOp, // A no-operation event, used for parent menu items.
}

/// Enum for the selectable duration options.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DurationOption {
    Permanent,
    Minutes(u32),
}

impl DurationOption {
    /// Converts the duration into seconds. Returns `None` for `Permanent`.
    pub fn to_seconds(&self) -> Option<u64> {
        match self {
            DurationOption::Permanent => None,
            DurationOption::Minutes(m) => Some(*m as u64 * 60),
        }
    }

    /// Gets the localized display text for the duration option.
    pub fn display_text(&self, t: &Translations) -> String {
        match self {
            DurationOption::Permanent => t.get("permanent"),
            DurationOption::Minutes(15) => t.get("minutes_15"),
            DurationOption::Minutes(30) => t.get("minutes_30"),
            DurationOption::Minutes(60) => t.get("hour_1"),
            DurationOption::Minutes(120) => t.get("hours_2"),
            _ => "Unknown".to_string(),
        }
    }
}

/// A constant list of all available duration options.
pub const DURATION_OPTIONS: &[DurationOption] = &[
    DurationOption::Permanent,
    DurationOption::Minutes(15),
    DurationOption::Minutes(30),
    DurationOption::Minutes(60),
    DurationOption::Minutes(120),
];

/// Holds the application's current runtime state.
pub struct AppState {
    pub is_active: bool,
    pub duration: DurationOption,
    pub translations: Translations,
    pub timer_shutdown_tx: Option<crossbeam_channel::Sender<()>>,
}

impl AppState {
    /// Creates a new `AppState` with default values.
    pub fn new() -> Self {
        AppState {
            is_active: true, // App is active by default.
            duration: DurationOption::Permanent,
            translations: i18n::load(),
            timer_shutdown_tx: None,
        }
    }
}
