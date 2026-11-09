//! Foydalanuvchi sozlamalari va sessiya (UX-09, P3-07). Fayl formati — TOML (`commandor-settings`).
//!
//! Hamma maydon `#[serde(default)]`: eski yoki qisman fayl ham o‘qiladi, noma’lum maydon xato emas.

use crate::{AppError, SortCriteria};
use serde::{Deserialize, Serialize};

pub const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub ui: UiSettings,
    /// Mutlaq papka yo‘llari.
    pub bookmarks: Vec<String>,
    pub session: Session,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            ui: UiSettings::default(),
            bookmarks: Vec::new(),
            session: Session::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// Ilova qayta ochilganda tablar tiklanadimi.
    pub restore_session: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            restore_session: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    /// 0 — A (chap), 1 — B (o‘ng).
    pub active: u8,
    pub left: PanelSession,
    pub right: PanelSession,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelSession {
    pub current: usize,
    pub tabs: Vec<TabSession>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TabSession {
    pub path: String,
    pub hidden: bool,
    pub sort: SortCriteria,
}

/// `LoadSettings` javobi: sozlama doim bor (sukut bo‘lsa ham); `problem` — fayl o‘qilmadi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsLoad {
    pub settings: Settings,
    pub problem: Option<AppError>,
}
