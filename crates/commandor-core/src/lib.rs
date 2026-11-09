//! Commandor domen turlari.
//!
//! Bu crate hech qanday IO qilmaydi: UI, broker va servislar o‘rtasidagi umumiy
//! shartnoma ([CONTRACTS](../../docs/CONTRACTS.uz.md)) shu yerda turlar sifatida yashaydi.

pub mod error;
pub mod filesystem;
pub mod ids;
pub mod job;
pub mod launch;
pub mod ops;
pub mod protocol;
pub mod revision;
pub mod settings;

pub use error::{AppError, DetailValue, ErrorCode};
pub use filesystem::{
    DirectoryListing, DirectoryTarget, EntryKind, FileEntry, ListOptions, Location, SortCriteria,
    SortDirection, SortField, Volume, VolumeKind, volume_index_for_path,
};
pub use ids::{DocumentId, EntryId, IdempotencyKey, JobId, LocationId, PlanId, RequestId};
pub use job::{JobState, TransitionError};
pub use launch::{AppChoice, AppId, OpenOutcome, OpenWith, OpenWithList};
pub use ops::{
    ConflictPolicy, ItemFailure, JobProgress, OperationPlan, OperationReport, PlanAction, PlanItem,
    PlanRequest, Totals,
};
pub use protocol::{API_VERSION, AppInfo, Command, Envelope, Event, Reply, Response};
pub use revision::{FileIdentity, Revision};
pub use settings::{
    PanelSession, SETTINGS_VERSION, Session, Settings, SettingsLoad, TabSession, UiSettings,
};
