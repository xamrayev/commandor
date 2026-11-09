//! Worker xabarlari.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

/// Diagnostika so‘rovlari (`Sleep`, `Abort`) faqat shu env o‘zgaruvchisi `1` bo‘lganda bajariladi.
pub const DIAG_ENV: &str = "COMMANDOR_WORKER_DIAG";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerKind {
    /// Protokol va supervisor sinovi uchun minimal worker.
    Diag,
}

impl WorkerKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "diag" => Some(Self::Diag),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Diag => "diag",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WorkerRequest {
    Ping,
    Echo(String),
    /// Diagnostika: timeoutni sinash.
    Sleep {
        millis: u64,
    },
    /// Diagnostika: crashni sinash.
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WorkerResponse {
    Pong,
    Echo(String),
    Slept,
    /// So‘rov tushunildi, lekin rad etildi (masalan, diagnostika o‘chiq).
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WorkerMessage {
    Hello {
        protocol_version: u32,
        kind: WorkerKind,
    },
    Request {
        id: u64,
        request: WorkerRequest,
    },
    Response {
        id: u64,
        response: WorkerResponse,
    },
}
