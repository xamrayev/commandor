//! Faylni tizim dasturida ochish va "Open with" kontraktlari.
//!
//! UI yo‘l yubormaydi: fayl listingdagi `LocationId` + `EntryId` bilan ko‘rsatiladi, dastur esa
//! broker bergan opaque [`AppId`] bilan tanlanadi.

use serde::{Deserialize, Serialize};

/// Broker bergan dastur identifikatori; faqat oxirgi `ListOpenWith` javobi doirasida amal qiladi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(u64);

impl AppId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Fayl qaysi dastur bilan ochiladi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenWith {
    /// OS’dagi standart dastur.
    Default,
    /// `ListOpenWith` ro‘yxatidagi dastur.
    App(AppId),
    /// OS’ning o‘z "Open with" dialogi (hozircha Windows).
    SystemChooser,
}

/// "Open with" ro‘yxatidagi bitta dastur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppChoice {
    pub id: AppId,
    pub name: String,
    /// Foydalanuvchiga ko‘rsatiladigan qo‘shimcha ma’lumot (masalan, dastur yo‘li).
    pub detail: String,
    /// Dastur aynan shu fayl turini e’lon qilgan.
    pub recommended: bool,
    /// OS bu tur uchun standart deb biladi (aniqlab bo‘lsa).
    pub is_default: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenWithList {
    /// Avval tavsiya etilganlar, keyin qolganlari; har guruh ichida nom bo‘yicha.
    pub apps: Vec<AppChoice>,
    /// OS’ning o‘z tanlash dialogi mavjud (`OpenWith::SystemChooser`).
    pub system_chooser: bool,
}

/// `OpenEntry` natijasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenOutcome {
    Launched,
    /// Fayl bajariladigan ko‘rinadi; foydalanuvchi tasdiqlasa `allow_executable: true` bilan qayta yuboriladi.
    NeedsConfirmation,
}
