//! Fayl revision modeli ([CONTRACTS §1](../../docs/CONTRACTS.uz.md#1-umumiy-kontrakt)).

use serde::{Deserialize, Serialize};

/// Platformaga xos barqaror fayl identity (masalan, Unix `dev`+`ino`, Windows volume serial + file index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileIdentity {
    pub volume: u64,
    pub file: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    /// Mutatsiya uchun snapshotdan olinadi; listingda `None` bo‘lishi mumkin.
    pub content_sha256: Option<[u8; 32]>,
    pub size_bytes: u64,
    /// Unix epoch’dan nanosekund; aniqlik yo‘qolmasligi uchun `i128`.
    pub modified_ns: i128,
    pub file_identity: Option<FileIdentity>,
}

impl Revision {
    /// Ikki revision bir xil mazmunni bildirishi *isbotlanganmi*.
    ///
    /// Hash ikkala tomonda bo‘lsagina `Some(true/false)`; aks holda `None` —
    /// faqat mtime/size moslik yakuniy konflikt kafolati emas.
    pub fn same_content(&self, other: &Self) -> Option<bool> {
        match (&self.content_sha256, &other.content_sha256) {
            (Some(a), Some(b)) => Some(a == b && self.size_bytes == other.size_bytes),
            _ => None,
        }
    }

    /// Metadata bo‘yicha o‘zgarish belgisi (hint, isbot emas).
    pub fn metadata_differs(&self, other: &Self) -> bool {
        self.size_bytes != other.size_bytes
            || self.modified_ns != other.modified_ns
            || self.file_identity != other.file_identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rev(hash: Option<u8>, size: u64, mtime: i128) -> Revision {
        Revision {
            content_sha256: hash.map(|b| [b; 32]),
            size_bytes: size,
            modified_ns: mtime,
            file_identity: Some(FileIdentity { volume: 1, file: 7 }),
        }
    }

    #[test]
    fn content_equality_requires_both_hashes() {
        assert_eq!(
            rev(Some(1), 10, 5).same_content(&rev(Some(1), 10, 9)),
            Some(true)
        );
        assert_eq!(
            rev(Some(1), 10, 5).same_content(&rev(Some(2), 10, 5)),
            Some(false)
        );
        assert_eq!(rev(None, 10, 5).same_content(&rev(Some(1), 10, 5)), None);
    }

    #[test]
    fn metadata_hint_detects_changes() {
        assert!(!rev(None, 10, 5).metadata_differs(&rev(None, 10, 5)));
        assert!(rev(None, 10, 5).metadata_differs(&rev(None, 11, 5)));
        assert!(rev(None, 10, 5).metadata_differs(&rev(None, 10, 6)));
    }

    #[test]
    fn nanosecond_precision_survives_json() {
        let original = rev(Some(3), 1, 1_762_500_000_123_456_789_i128);
        let json = serde_json::to_string(&original).expect("serialize");
        let back: Revision = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, original);
    }
}
