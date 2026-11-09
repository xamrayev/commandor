//! Asosiy jarayon ↔ `commandor --worker` aloqasi.
//!
//! Freym: 4 bayt little-endian uzunlik + JSON tana ([`frame`]). Ulanish boshida worker
//! [`WorkerMessage::Hello`] yuboradi; supervisor protokol versiyasini tekshiradi.
//! Ishonchsiz kontent parserlari keyingi bosqichlarda shu protokolga so‘rov turi sifatida qo‘shiladi.

pub mod frame;
pub mod protocol;
pub mod supervisor;
pub mod worker;

pub use protocol::{PROTOCOL_VERSION, WorkerKind, WorkerMessage, WorkerRequest, WorkerResponse};
pub use supervisor::{Supervisor, SupervisorError, WorkerConfig};
