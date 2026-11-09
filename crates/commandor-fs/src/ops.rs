//! Fayl amallari rejasi va bajarilishi: yangi papka, rename, savatga tashlash.
//!
//! Reja tuzishda disk o‘zgarmaydi. Bajarishda grant, manba identifikatori va maqsad yo‘qligi
//! qayta tekshiriladi; reja bir marta ishlatiladi va muddati o‘tsa `PlanExpired`.

use crate::transfer::{self, TransferJob, TransferMode};
use crate::{FileService, identity, io_error};
use commandor_core::{
    AppError, ConflictPolicy, DetailValue, EntryKind, ErrorCode, FileIdentity, ItemFailure,
    LocationId, OperationPlan, OperationReport, PlanAction, PlanId, PlanItem, PlanRequest, Totals,
};

/// `create_plan` ichida so‘rov turini (request ko‘chirilgandan keyin) eslab qolish.
enum RequestKind {
    Move,
    Other,
}
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PLAN_TTL: Duration = Duration::from_secs(10 * 60);
const PLAN_LIMIT: usize = 32;
const MAX_TRASH_ITEMS: usize = 10_000;
const MAX_NAME_BYTES: usize = 255;

/// Savatga tashlash usuli (testlarda soxta implementatsiya).
pub trait Trasher: Send {
    /// Yo‘lni savatga ko‘chiradi. Savat bo‘lmasa `TrashUnavailable`; doimiy o‘chirmaydi.
    fn trash(&self, path: &Path) -> Result<(), AppError>;
}

/// OS savati (`trash` crate): macOS NSFileManager, Windows Recycle Bin, Linux freedesktop.
pub struct SystemTrash;

impl Trasher for SystemTrash {
    fn trash(&self, path: &Path) -> Result<(), AppError> {
        #[allow(unused_mut)]
        let mut context = trash::TrashContext::default();
        #[cfg(target_os = "macos")]
        {
            // Finder usuli AppleScript orqali ishlaydi va Automation ruxsatini so‘raydi.
            use trash::macos::{DeleteMethod, TrashContextExtMacos};
            context.set_delete_method(DeleteMethod::NsFileManager);
        }
        context.delete(path).map_err(|error| match error {
            trash::Error::CouldNotAccess { .. } => ErrorCode::PermissionDenied.into(),
            trash::Error::TargetedRoot => ErrorCode::PolicyBlocked.into(),
            _ => AppError::from(ErrorCode::TrashUnavailable),
        })
    }
}

/// Nom qoidalari qaysi OS bo‘yicha tekshiriladi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRules {
    Unix,
    MacOs,
    Windows,
}

impl NameRules {
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Unix
        }
    }
}

/// Noto‘g‘ri so‘rov (nom emas), sababi bilan: UI `error-reason-*` matnini tanlaydi.
fn refused(reason: &str) -> AppError {
    AppError::new(ErrorCode::InvalidArgument)
        .with_detail("reason", DetailValue::Text(reason.into()))
}

fn invalid(reason: &str) -> AppError {
    AppError::new(ErrorCode::InvalidName).with_detail("reason", DetailValue::Text(reason.into()))
}

/// Bitta yo‘l komponenti sifatidagi nomni tekshiradi (papka ajratgichi, `.`/`..`, OS taqiqlari).
pub fn validate_name(name: &str, rules: NameRules) -> Result<(), AppError> {
    if name.is_empty() || name.trim().is_empty() {
        return Err(invalid("empty"));
    }
    if name == "." || name == ".." {
        return Err(invalid("dot"));
    }
    if name.len() > MAX_NAME_BYTES {
        return Err(invalid("too_long"));
    }
    if name
        .chars()
        .any(|c| c == '/' || c == '\0' || c.is_control())
    {
        return Err(invalid("character"));
    }
    match rules {
        NameRules::Unix => {}
        NameRules::MacOs => {
            // Finder `:`ni `/` sifatida ko‘rsatadi; chalkashlikka yo‘l qo‘ymaymiz.
            if name.contains(':') {
                return Err(invalid("character"));
            }
        }
        NameRules::Windows => {
            if name
                .chars()
                .any(|c| matches!(c, '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*'))
            {
                return Err(invalid("character"));
            }
            if name.ends_with(' ') || name.ends_with('.') {
                return Err(invalid("trailing"));
            }
            let stem = name
                .split('.')
                .next()
                .unwrap_or(name)
                .trim_end()
                .to_ascii_uppercase();
            let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.len() == 4
                    && stem.as_bytes()[3].is_ascii_digit()
                    && stem.as_bytes()[3] != b'0');
            if reserved {
                return Err(invalid("reserved"));
            }
        }
    }
    Ok(())
}

/// Barqaror 64-bit FNV-1a (std `Hasher` versiyalar orasida barqaror emas).
fn fnv1a(parts: &[&[u8]]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.iter().chain(&[0xff]) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub(crate) path: PathBuf,
    identity: Option<FileIdentity>,
    is_symlink: bool,
}

#[derive(Debug, Clone)]
enum Stored {
    CreateDirectory {
        target: PathBuf,
    },
    Rename {
        source: Source,
        target: PathBuf,
    },
    Trash {
        sources: Vec<Source>,
    },
    Transfer {
        mode: TransferMode,
        sources: Vec<Source>,
        target_location: LocationId,
        target_dir: PathBuf,
        target_identity: Option<FileIdentity>,
        /// Barcha manbalar daraxti bo‘yicha jami (sanash limitdan oshsa `None`).
        totals: Option<Totals>,
    },
}

pub(crate) struct StoredPlan {
    plan: OperationPlan,
    location: LocationId,
    created: Instant,
    stored: Stored,
}

fn kind_of(metadata: &fs::Metadata) -> EntryKind {
    if metadata.is_symlink() {
        EntryKind::Symlink
    } else if metadata.is_dir() {
        EntryKind::Directory
    } else if metadata.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

fn source_of(path: PathBuf, name: &str) -> Result<(Source, PlanItem), AppError> {
    let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
    let item = PlanItem {
        name: name.to_owned(),
        new_name: None,
        kind: kind_of(&metadata),
        size_bytes: metadata.is_file().then_some(metadata.len()),
        conflict: false,
    };
    let source = Source {
        path,
        identity: identity(&metadata),
        is_symlink: metadata.is_symlink(),
    };
    Ok((source, item))
}

/// Manba reja tuzilgandan beri almashtirilmaganmi (o‘sha inode, o‘sha tur).
pub(crate) fn still_same(source: &Source) -> Result<fs::Metadata, AppError> {
    let metadata = fs::symlink_metadata(&source.path).map_err(io_error)?;
    if metadata.is_symlink() != source.is_symlink || identity(&metadata) != source.identity {
        return Err(ErrorCode::StaleReference.into());
    }
    Ok(metadata)
}

impl FileService {
    /// Rejani tuzadi va saqlaydi. Disk o‘zgarmaydi.
    pub fn create_plan(&mut self, request: PlanRequest) -> Result<OperationPlan, AppError> {
        self.expire_plans();
        let request_kind = match &request {
            PlanRequest::Move { .. } => RequestKind::Move,
            _ => RequestKind::Other,
        };
        let rules = NameRules::current();
        let mut policy = ConflictPolicy::default();
        let (location, action, items, stored) = match request {
            PlanRequest::CreateDirectory { location, name } => {
                validate_name(&name, rules)?;
                let grant = self.checked(location)?;
                let target = grant.path.join(&name);
                if fs::symlink_metadata(&target).is_ok() {
                    return Err(ErrorCode::DestinationExists.into());
                }
                let item = PlanItem {
                    name,
                    new_name: None,
                    kind: EntryKind::Directory,
                    size_bytes: None,
                    conflict: false,
                };
                (
                    location,
                    PlanAction::CreateDirectory,
                    vec![item],
                    Stored::CreateDirectory { target },
                )
            }
            PlanRequest::Rename {
                location,
                entry,
                new_name,
            } => {
                validate_name(&new_name, rules)?;
                let grant = self.checked(location)?;
                let name = grant.entries.get(&entry).ok_or(ErrorCode::StaleReference)?;
                let display = name.to_string_lossy().into_owned();
                if OsString::from(&new_name) == *name {
                    return Err(invalid("unchanged"));
                }
                let (source, mut item) = source_of(grant.path.join(name), &display)?;
                let target = grant.path.join(&new_name);
                check_rename_target(&source, &target)?;
                item.new_name = Some(new_name);
                (
                    location,
                    PlanAction::Rename,
                    vec![item],
                    Stored::Rename { source, target },
                )
            }
            PlanRequest::Trash { location, entries } => {
                if entries.is_empty() || entries.len() > MAX_TRASH_ITEMS {
                    return Err(ErrorCode::InvalidArgument.into());
                }
                let grant = self.checked(location)?;
                let mut sources = Vec::with_capacity(entries.len());
                let mut items = Vec::with_capacity(entries.len());
                let mut seen = std::collections::BTreeSet::new();
                for entry in entries {
                    if !seen.insert(entry) {
                        continue;
                    }
                    let name = grant.entries.get(&entry).ok_or(ErrorCode::StaleReference)?;
                    let display = name.to_string_lossy().into_owned();
                    let (source, item) = source_of(grant.path.join(name), &display)?;
                    sources.push(source);
                    items.push(item);
                }
                (
                    location,
                    PlanAction::Trash,
                    items,
                    Stored::Trash { sources },
                )
            }
            PlanRequest::Copy {
                location,
                entries,
                target,
                policy: chosen,
            }
            | PlanRequest::Move {
                location,
                entries,
                target,
                policy: chosen,
            } => {
                let mode = if matches!(request_kind, RequestKind::Move) {
                    TransferMode::Move
                } else {
                    TransferMode::Copy
                };
                policy = chosen;
                let (items, stored) = self.plan_transfer(mode, location, &entries, target)?;
                let action = match mode {
                    TransferMode::Copy => PlanAction::Copy,
                    TransferMode::Move => PlanAction::Move,
                };
                (location, action, items, stored)
            }
        };
        let directory = self.checked(location)?.path.to_string_lossy().into_owned();
        let (target, totals) = match &stored {
            Stored::Transfer {
                target_dir, totals, ..
            } => (Some(target_dir.to_string_lossy().into_owned()), *totals),
            _ => (None, None),
        };
        let id = PlanId::from_raw(self.id());
        let hash = plan_hash(id, action, &directory, &items, &stored, policy);
        let plan = OperationPlan {
            id,
            hash,
            action,
            directory,
            items,
            requires_approval: matches!(
                action,
                PlanAction::Trash | PlanAction::Copy | PlanAction::Move
            ),
            target,
            policy,
            totals,
        };
        self.plans.insert(
            id,
            StoredPlan {
                plan: plan.clone(),
                location,
                created: Instant::now(),
                stored,
            },
        );
        while self.plans.len() > PLAN_LIMIT {
            self.plans.pop_first();
        }
        Ok(plan)
    }

    pub fn discard_plan(&mut self, id: PlanId) {
        self.plans.remove(&id);
    }

    /// Rejani bir marta bajaradi. Xesh, muddat, grant va manbalar qayta tekshiriladi.
    pub fn execute_plan(&mut self, id: PlanId, hash: u64) -> Result<OperationReport, AppError> {
        self.expire_plans();
        let stored = self.plans.remove(&id).ok_or(ErrorCode::PlanExpired)?;
        if stored.plan.hash != hash {
            return Err(ErrorCode::PlanExpired.into());
        }
        // Papka o‘zi almashtirilgan bo‘lsa grant rad etiladi.
        self.checked(stored.location)?;
        let mut report = OperationReport {
            action: stored.plan.action,
            directory: stored.plan.directory.clone(),
            done: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            target: None,
            cancelled: false,
        };
        match stored.stored {
            // Fon vazifasi: `start_transfer` orqali boshlanadi.
            Stored::Transfer { .. } => return Err(ErrorCode::InvalidArgument.into()),
            Stored::CreateDirectory { target } => {
                fs::create_dir(&target).map_err(io_error)?;
                report.done.push(stored.plan.items[0].name.clone());
            }
            Stored::Rename { source, target } => {
                still_same(&source)?;
                check_rename_target(&source, &target)?;
                // Tekshiruv va rename orasida maqsad paydo bo‘lishi mumkin (TOCTOU oynasi);
                // `renameat2(RENAME_NOREPLACE)` FFI talab qiladi — cheklov hisobotda.
                fs::rename(&source.path, &target).map_err(io_error)?;
                let item = &stored.plan.items[0];
                report.done.push(item.new_name.clone().unwrap_or_default());
            }
            Stored::Trash { sources } => {
                for (source, item) in sources.iter().zip(&stored.plan.items) {
                    let result = still_same(source).and_then(|_| self.trash.trash(&source.path));
                    match result {
                        Ok(()) => report.done.push(item.name.clone()),
                        Err(error) => report.failed.push(ItemFailure {
                            name: item.name.clone(),
                            error,
                        }),
                    }
                }
            }
        }
        Ok(report)
    }

    /// Reja turi (broker vazifa yoki oddiy bajarishni tanlaydi).
    pub fn plan_action(&self, id: PlanId) -> Option<PlanAction> {
        self.plans.get(&id).map(|stored| stored.plan.action)
    }

    /// Nusxalash/ko‘chirish rejasini fon vazifasiga aylantiradi (reja iste’mol qilinadi).
    pub fn start_transfer(&mut self, id: PlanId, hash: u64) -> Result<TransferJob, AppError> {
        self.expire_plans();
        let stored = self.plans.remove(&id).ok_or(ErrorCode::PlanExpired)?;
        if stored.plan.hash != hash {
            return Err(ErrorCode::PlanExpired.into());
        }
        let Stored::Transfer {
            mode,
            sources,
            target_location,
            target_dir,
            target_identity,
            ..
        } = stored.stored
        else {
            return Err(ErrorCode::InvalidArgument.into());
        };
        self.checked(stored.location)?;
        self.checked(target_location)?;
        if transfer::dir_identity(&target_dir) != target_identity {
            return Err(ErrorCode::StaleReference.into());
        }
        let conflicted = stored.plan.items.iter().any(|item| item.conflict);
        if conflicted && stored.plan.policy == ConflictPolicy::Ask {
            return Err(ErrorCode::Conflict.into());
        }
        let names = stored.plan.items.iter().map(|item| item.name.clone());
        Ok(TransferJob {
            mode,
            sources: sources.into_iter().zip(names).collect(),
            target_display: target_dir.to_string_lossy().into_owned(),
            target_dir,
            policy: stored.plan.policy,
            directory: stored.plan.directory,
            totals: stored.plan.totals,
        })
    }

    fn plan_transfer(
        &self,
        mode: TransferMode,
        location: LocationId,
        entries: &[commandor_core::EntryId],
        target: LocationId,
    ) -> Result<(Vec<PlanItem>, Stored), AppError> {
        if entries.is_empty() || entries.len() > MAX_TRASH_ITEMS {
            return Err(ErrorCode::InvalidArgument.into());
        }
        let grant = self.checked(location)?;
        let target_dir = self.checked(target)?.path.clone();
        if mode == TransferMode::Move && target_dir == grant.path {
            return Err(refused("same_directory"));
        }
        let mut sources = Vec::with_capacity(entries.len());
        let mut items = Vec::with_capacity(entries.len());
        let mut seen = std::collections::BTreeSet::new();
        let mut totals = Some(Totals::default());
        for entry in entries {
            if !seen.insert(*entry) {
                continue;
            }
            let name = grant.entries.get(entry).ok_or(ErrorCode::StaleReference)?;
            let display = name.to_string_lossy().into_owned();
            let (source, mut item) = source_of(grant.path.join(name), &display)?;
            if item.kind == EntryKind::Directory
                && transfer::target_inside(&source.path, &target_dir)
            {
                return Err(refused("into_itself"));
            }
            let tree = transfer::scan(&source.path);
            if item.kind == EntryKind::Directory {
                item.size_bytes = tree.map(|tree| tree.bytes);
            }
            totals = totals.zip(tree).map(|(sum, tree)| Totals {
                items: sum.items + tree.items,
                bytes: sum.bytes + tree.bytes,
            });
            item.conflict = transfer::conflicts(&target_dir, &display);
            sources.push(source);
            items.push(item);
        }
        let stored = Stored::Transfer {
            mode,
            sources,
            target_location: target,
            target_identity: transfer::dir_identity(&target_dir),
            target_dir,
            totals,
        };
        Ok((items, stored))
    }

    fn expire_plans(&mut self) {
        self.plans
            .retain(|_, plan| plan.created.elapsed() < PLAN_TTL);
    }
}

/// Maqsad bo‘sh bo‘lishi kerak. Istisno: case-only rename (o‘sha fayl, registrga befarq FS).
fn check_rename_target(source: &Source, target: &Path) -> Result<(), AppError> {
    match fs::symlink_metadata(target) {
        Ok(existing) => {
            let same = source.identity.is_some() && identity(&existing) == source.identity;
            // Windowsda identity yo‘q: faqat registr farqi bo‘lsa o‘sha fayl deb qabul qilinadi.
            let case_only = source.identity.is_none()
                && source
                    .path
                    .file_name()
                    .zip(target.file_name())
                    .is_some_and(|(a, b)| {
                        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
                    });
            if same || case_only {
                Ok(())
            } else {
                Err(ErrorCode::DestinationExists.into())
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

fn plan_hash(
    id: PlanId,
    action: PlanAction,
    directory: &str,
    items: &[PlanItem],
    stored: &Stored,
    policy: ConflictPolicy,
) -> u64 {
    let mut parts: Vec<Vec<u8>> = vec![
        id.raw().to_le_bytes().to_vec(),
        format!("{action:?}{policy:?}").into_bytes(),
        directory.as_bytes().to_vec(),
    ];
    for item in items {
        parts.push(item.name.as_bytes().to_vec());
        parts.push(item.new_name.clone().unwrap_or_default().into_bytes());
        parts.push(format!("{:?}{:?}{}", item.kind, item.size_bytes, item.conflict).into_bytes());
    }
    let identity_bytes = |source: &Source| format!("{:?}", source.identity).into_bytes();
    match stored {
        Stored::CreateDirectory { target } => {
            parts.push(target.to_string_lossy().as_bytes().to_vec());
        }
        Stored::Rename { source, target } => {
            parts.push(identity_bytes(source));
            parts.push(target.to_string_lossy().as_bytes().to_vec());
        }
        Stored::Trash { sources } => parts.extend(sources.iter().map(identity_bytes)),
        Stored::Transfer {
            mode,
            sources,
            target_dir,
            target_identity,
            ..
        } => {
            parts.push(format!("{mode:?}{target_identity:?}").into_bytes());
            parts.push(target_dir.to_string_lossy().as_bytes().to_vec());
            parts.extend(sources.iter().map(identity_bytes));
        }
    }
    let slices: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
    fnv1a(&slices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_each_os_rules() {
        for rules in [NameRules::Unix, NameRules::MacOs, NameRules::Windows] {
            assert!(validate_name("Yangi papka", rules).is_ok());
            assert!(validate_name("O‘zbek hujjat (1).docx", rules).is_ok());
            assert!(validate_name(".config", rules).is_ok());
            for bad in ["", "   ", ".", "..", "a/b", "a\0b", "tab\there"] {
                assert_eq!(
                    validate_name(bad, rules).expect_err(bad).code,
                    ErrorCode::InvalidName,
                    "{rules:?} {bad:?}"
                );
            }
            assert!(validate_name(&"x".repeat(256), rules).is_err());
        }
        assert!(validate_name("a:b", NameRules::Unix).is_ok());
        assert!(validate_name("a:b", NameRules::MacOs).is_err());
        assert!(validate_name("a\\b", NameRules::Unix).is_ok());
        for bad in [
            "a\\b",
            "a?b",
            "a*b",
            "a<b",
            "nuqta.",
            "bo‘shliq ",
            "CON",
            "nul.txt",
            "Com1",
            "LPT9.log",
        ] {
            assert!(validate_name(bad, NameRules::Windows).is_err(), "{bad}");
        }
        for ok in ["COM0", "COM10", "CONSOLE", "lpt"] {
            assert!(validate_name(ok, NameRules::Windows).is_ok(), "{ok}");
        }
    }

    #[test]
    fn fnv_is_stable_and_separates_parts() {
        assert_eq!(fnv1a(&[b"a", b"bc"]), fnv1a(&[b"a", b"bc"]));
        assert_ne!(fnv1a(&[b"ab", b"c"]), fnv1a(&[b"a", b"bc"]));
    }
}
