//! 应用的状态定义模块

use super::i18n::{self, Translations};

/// 菜单事件的枚举
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Event {
    ShowMenu,
    ToggleActive,
    SetDuration(DurationOption),
    ThemeChanged, // 系统主题变化
    Exit,
    NoOp, // 空操作事件
}

/// 可选的持续时间
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DurationOption {
    Permanent,
    Minutes(u32),
}

impl DurationOption {
    pub fn to_seconds(self) -> Option<u64> {
        match self {
            DurationOption::Permanent => None,
            DurationOption::Minutes(m) => Some(m as u64 * 60),
        }
    }

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

pub const DURATION_OPTIONS: &[DurationOption] = &[
    DurationOption::Permanent,
    DurationOption::Minutes(15),
    DurationOption::Minutes(30),
    DurationOption::Minutes(60),
    DurationOption::Minutes(120),
];

/// 保存应用当前状态的结构体
pub struct AppState {
    pub is_active: bool,
    pub duration: DurationOption,
    pub translations: Translations,
    pub timer_shutdown_tx: Option<crossbeam_channel::Sender<()>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_active: true, // 默认开启
            duration: DurationOption::Permanent,
            translations: i18n::load(),
            timer_shutdown_tx: None,
        }
    }
}
