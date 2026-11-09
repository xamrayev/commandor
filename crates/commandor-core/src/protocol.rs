//! UI ↔ broker protokoli. Yangi servislar o‘z bosqichida variant qo‘shadi.

use crate::{
    AppError, DirectoryListing, DirectoryTarget, EntryId, JobId, JobProgress, ListOptions,
    LocationId, OpenOutcome, OpenWith, OpenWithList, OperationPlan, OperationReport, PlanId,
    PlanRequest, RequestId, Volume,
};
use serde::{Deserialize, Serialize};

/// Kontrakt versiyasi; broker boshqa versiyani `UNSUPPORTED_API_VERSION` bilan rad etadi.
pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub request: RequestId,
    pub api_version: u32,
    pub body: T,
}

/// UI yubora oladigan yagona buyruqlar to‘plami (allowlist).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Command {
    /// Broker tirikligini tekshirish.
    Ping,
    /// Ilova va platforma haqida ma’lumot.
    GetAppInfo,
    /// Alohida file threadda o‘qiladi; UI natijani RequestId bilan bog‘laydi.
    ReadDirectory {
        target: DirectoryTarget,
        options: ListOptions,
    },
    /// Foydalanuvchiga ko‘rinadigan tomlar (disklar) ro‘yxati; file threadda o‘qiladi.
    ListVolumes,
    /// Listingdagi faylni tizim dasturida ochish. Papka va symlink rad etiladi.
    /// Bajariladigan fayl `allow_executable` bo‘lmasa ishga tushmaydi (`OpenOutcome::NeedsConfirmation`).
    OpenEntry {
        location: LocationId,
        entry: EntryId,
        with: OpenWith,
        allow_executable: bool,
    },
    /// Fayl uchun "Open with" dasturlari ro‘yxati.
    ListOpenWith {
        location: LocationId,
        entry: EntryId,
    },
    /// Fayl amali rejasini tuzish. Disk o‘zgarmaydi; broker nomlarni tekshiradi va reja qaytaradi.
    CreatePlan(PlanRequest),
    /// Tasdiqlangan rejani bajarish. `hash` reja bilan mos kelmasa yoki manba o‘zgargan bo‘lsa rad etiladi.
    ExecutePlan { plan: PlanId, hash: u64 },
    /// Foydalanuvchi bekor qilgan rejani unutish.
    DiscardPlan { plan: PlanId },
    /// Fon vazifasini bekor qilish; yarim fayl to‘liq deb qoldirilmaydi.
    CancelJob { job: JobId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub api_version: u32,
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Response {
    Pong,
    AppInfo(AppInfo),
    Directory(DirectoryListing),
    Volumes(Vec<Volume>),
    Opened(OpenOutcome),
    OpenWithApps(OpenWithList),
    Plan(OperationPlan),
    Executed(OperationReport),
    Discarded,
    /// Reja fon vazifasi sifatida boshlandi; natija `Event::JobFinished` bilan keladi.
    JobStarted(JobId),
    CancelRequested,
}

pub type Reply = Result<Response, AppError>;

/// Broker → UI hodisalari.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    Reply {
        request: RequestId,
        reply: Reply,
    },
    JobProgress(JobProgress),
    JobFinished {
        job: JobId,
        result: Result<OperationReport, AppError>,
    },
}
