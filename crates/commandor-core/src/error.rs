//! UIga chiqadigan xato modeli ([CONTRACTS §8](../../docs/CONTRACTS.uz.md#8-xatolar-va-kuzatuv)).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ErrorCode {
    PermissionDenied,
    RootUnavailable,
    NotFound,
    StaleReference,
    Conflict,
    DestinationExists,
    UnsupportedFormat,
    UnsupportedEncoding,
    PasswordRequired,
    EngineNotInstalled,
    EngineCrashed,
    ModelUnsupported,
    OutOfMemory,
    ChecksumMismatch,
    DownloadInterrupted,
    ProviderUnavailable,
    RateLimited,
    PolicyBlocked,
    PlanExpired,
    DiskFull,
    Cancelled,
    InvalidArgument,
    UnsupportedApiVersion,
    Internal,
}

impl ErrorCode {
    pub const ALL: &'static [ErrorCode] = &[
        Self::PermissionDenied,
        Self::RootUnavailable,
        Self::NotFound,
        Self::StaleReference,
        Self::Conflict,
        Self::DestinationExists,
        Self::UnsupportedFormat,
        Self::UnsupportedEncoding,
        Self::PasswordRequired,
        Self::EngineNotInstalled,
        Self::EngineCrashed,
        Self::ModelUnsupported,
        Self::OutOfMemory,
        Self::ChecksumMismatch,
        Self::DownloadInterrupted,
        Self::ProviderUnavailable,
        Self::RateLimited,
        Self::PolicyBlocked,
        Self::PlanExpired,
        Self::DiskFull,
        Self::Cancelled,
        Self::InvalidArgument,
        Self::UnsupportedApiVersion,
        Self::Internal,
    ];

    /// Fluent tarjima kaliti (`resources/i18n/*/main.ftl`).
    pub const fn message_key(self) -> &'static str {
        match self {
            Self::PermissionDenied => "error-permission-denied",
            Self::RootUnavailable => "error-root-unavailable",
            Self::NotFound => "error-not-found",
            Self::StaleReference => "error-stale-reference",
            Self::Conflict => "error-conflict",
            Self::DestinationExists => "error-destination-exists",
            Self::UnsupportedFormat => "error-unsupported-format",
            Self::UnsupportedEncoding => "error-unsupported-encoding",
            Self::PasswordRequired => "error-password-required",
            Self::EngineNotInstalled => "error-engine-not-installed",
            Self::EngineCrashed => "error-engine-crashed",
            Self::ModelUnsupported => "error-model-unsupported",
            Self::OutOfMemory => "error-out-of-memory",
            Self::ChecksumMismatch => "error-checksum-mismatch",
            Self::DownloadInterrupted => "error-download-interrupted",
            Self::ProviderUnavailable => "error-provider-unavailable",
            Self::RateLimited => "error-rate-limited",
            Self::PolicyBlocked => "error-policy-blocked",
            Self::PlanExpired => "error-plan-expired",
            Self::DiskFull => "error-disk-full",
            Self::Cancelled => "error-cancelled",
            Self::InvalidArgument => "error-invalid-argument",
            Self::UnsupportedApiVersion => "error-unsupported-api-version",
            Self::Internal => "error-internal",
        }
    }

    /// Sukut bo‘yicha qayta urinish mantiqiymi. Cheksiz retry UI qarori emas.
    pub const fn default_retryable(self) -> bool {
        matches!(
            self,
            Self::RootUnavailable
                | Self::StaleReference
                | Self::EngineCrashed
                | Self::DownloadInterrupted
                | Self::ProviderUnavailable
                | Self::RateLimited
        )
    }
}

/// Xato tafsiloti: faqat oddiy qiymatlar, yo‘l/stack/maxfiy ma’lumot emas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DetailValue {
    Bool(bool),
    Int(i64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}")]
pub struct AppError {
    pub code: ErrorCode,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, DetailValue>,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            retryable: code.default_retryable(),
            details: BTreeMap::new(),
        }
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: DetailValue) -> Self {
        self.details.insert(key.into(), value);
        self
    }

    pub const fn message_key(&self) -> &'static str {
        self.code.message_key()
    }
}

impl From<ErrorCode> for AppError {
    fn from(code: ErrorCode) -> Self {
        Self::new(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn message_keys_are_unique_and_prefixed() {
        let keys: HashSet<_> = ErrorCode::ALL.iter().map(|c| c.message_key()).collect();
        assert_eq!(keys.len(), ErrorCode::ALL.len());
        assert!(keys.iter().all(|k| k.starts_with("error-")));
    }

    #[test]
    fn wire_format_is_screaming_snake_case() {
        let json = serde_json::to_string(&ErrorCode::DestinationExists).expect("serialize");
        assert_eq!(json, "\"DESTINATION_EXISTS\"");
    }

    #[test]
    fn app_error_roundtrip_with_details() {
        let err =
            AppError::new(ErrorCode::DiskFull).with_detail("needed_bytes", DetailValue::Int(10));
        assert!(!err.retryable);
        let back: AppError =
            serde_json::from_str(&serde_json::to_string(&err).expect("serialize")).expect("de");
        assert_eq!(back, err);
    }
}
