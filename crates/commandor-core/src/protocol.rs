//! UI ↔ broker protokoli. Yangi servislar o‘z bosqichida variant qo‘shadi.

use crate::{AppError, DirectoryListing, DirectoryTarget, ListOptions, RequestId, Volume};
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
}

pub type Reply = Result<Response, AppError>;

/// Broker → UI hodisalari.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    Reply { request: RequestId, reply: Reply },
}
