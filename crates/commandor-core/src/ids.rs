//! Opaque identifikatorlar. UI ularning ichki qiymatiga ma’no bermaydi.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! opaque_u64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            /// Faqat broker/servis yangi ID yaratadi.
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

opaque_u64!(
    /// Har bir UI so‘rovi uchun monoton o‘suvchi raqam; eski javobni ajratishga ham xizmat qiladi.
    RequestId
);
opaque_u64!(
    /// Broker bergan papka grant/location identifikatori.
    LocationId
);
opaque_u64!(
    /// Listingdagi yozuv identifikatori; panel/oynaga bog‘langan.
    EntryId
);
opaque_u64!(
    /// Ochilgan hujjat sessiyasi identifikatori.
    DocumentId
);
opaque_u64!(
    /// Broker tuzgan fayl amali rejasi; faqat shu broker sessiyasida amal qiladi.
    PlanId
);

/// Uzoq ishlar identifikatori.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// O‘zgartiruvchi amalni ikki marta bajarmaslik kaliti.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdempotencyKey(Uuid);

impl IdempotencyKey {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for IdempotencyKey {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_ids_serialize_as_plain_numbers() {
        let json = serde_json::to_string(&EntryId::from_raw(42)).expect("serialize");
        assert_eq!(json, "42");
        let back: EntryId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.raw(), 42);
    }

    #[test]
    fn job_ids_are_unique() {
        assert_ne!(JobId::new(), JobId::new());
        assert_ne!(IdempotencyKey::new(), IdempotencyKey::new());
    }
}
