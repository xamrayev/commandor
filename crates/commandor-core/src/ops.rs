//! Fayl amallari: reja → tasdiq → bajarish ([CONTRACTS 2.1](../../docs/CONTRACTS.uz.md)).
//!
//! UI faqat so‘rov yuboradi. Broker manbalarni grant orqali aniqlaydi, nomlarni tekshiradi va
//! o‘zgarmas rejani `PlanId` + `hash` bilan qaytaradi. Bajarishda manba qayta tekshiriladi.

use crate::{AppError, EntryId, EntryKind, JobId, LocationId, PlanId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanRequest {
    /// Joriy papkada yangi papka.
    CreateDirectory { location: LocationId, name: String },
    /// Bitta yozuv nomini o‘zgartirish (shu papka ichida).
    Rename {
        location: LocationId,
        entry: EntryId,
        new_name: String,
    },
    /// Yozuvlarni tizim savatiga tashlash. Savat bo‘lmasa doimiy o‘chirish qilinmaydi.
    Trash {
        location: LocationId,
        entries: Vec<EntryId>,
    },
    /// Yozuvlarni boshqa papkaga nusxalash (fon vazifasi).
    Copy {
        location: LocationId,
        entries: Vec<EntryId>,
        target: LocationId,
        policy: ConflictPolicy,
    },
    /// Ko‘chirish: bir diskda rename, disklararo — nusxa, hajm tekshiruvi, keyin manbani o‘chirish.
    Move {
        location: LocationId,
        entries: Vec<EntryId>,
        target: LocationId,
        policy: ConflictPolicy,
    },
}

/// Maqsadda shu nomli yozuv bo‘lsa (FS-04). Qaror butun reja uchun bitta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConflictPolicy {
    /// Konflikt bo‘lsa reja bajarilmaydi — foydalanuvchi tanlashi kerak.
    #[default]
    Ask,
    Skip,
    /// Yangi nom: `nom (2).ext`.
    KeepBoth,
    /// Faylni almashtirish; papka bo‘lsa ichiga qo‘shish (eski fayllar o‘chirilmaydi).
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanAction {
    CreateDirectory,
    Rename,
    Trash,
    Copy,
    Move,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanItem {
    /// Mavjud yozuv nomi (yangi papka uchun — yaratiladigan nom).
    pub name: String,
    /// Rename uchun yangi nom.
    pub new_name: Option<String>,
    pub kind: EntryKind,
    /// Fayl hajmi; nusxalash/ko‘chirishda papka ichidagi jami hajm.
    pub size_bytes: Option<u64>,
    /// Maqsadda shu nomli yozuv bor.
    #[serde(default)]
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationPlan {
    pub id: PlanId,
    /// Rejaning barcha ta’sir qiluvchi maydonlari xeshi; `ExecutePlan` uni qaytarib yuboradi.
    pub hash: u64,
    pub action: PlanAction,
    /// Amal bajariladigan papkaning ko‘rsatish yo‘li.
    pub directory: String,
    pub items: Vec<PlanItem>,
    /// Foydalanuvchi alohida tasdiqlashi kerak (masalan, savat).
    pub requires_approval: bool,
    /// Nusxalash/ko‘chirish maqsadi.
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub policy: ConflictPolicy,
    /// Jami fayl+papka soni va baytlar (oldindan sanash limitdan oshsa `None`).
    #[serde(default)]
    pub totals: Option<Totals>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Totals {
    pub items: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemFailure {
    pub name: String,
    pub error: AppError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationReport {
    pub action: PlanAction,
    pub directory: String,
    /// Muvaffaqiyatli bajarilgan yozuvlar (rename uchun yangi nom).
    pub done: Vec<String>,
    pub failed: Vec<ItemFailure>,
    /// Konflikt sababli o‘tkazib yuborilgan yozuvlar.
    #[serde(default)]
    pub skipped: Vec<String>,
    /// Nusxalash/ko‘chirish maqsadi (UI shu papkani ham yangilaydi).
    #[serde(default)]
    pub target: Option<String>,
    /// Bekor qilish bilan to‘xtadi; qolgan yozuvlar tegilmadi.
    #[serde(default)]
    pub cancelled: bool,
}

/// Fon vazifasi holati (cheklangan chastotada yuboriladi).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobProgress {
    pub job: JobId,
    pub action: PlanAction,
    pub done_items: u64,
    pub done_bytes: u64,
    pub totals: Option<Totals>,
    /// Hozir ishlanayotgan yozuv nomi (ko‘rsatish uchun).
    pub current: String,
}
