//! Read-only papka servisi. UI yo‘llari disk amali uchun qayta ishlatilmaydi.

mod volumes;

pub use volumes::{MountRules, display_name, is_network_fs, list_volumes};

use commandor_core::{
    AppError, DirectoryListing, DirectoryTarget, EntryId, EntryKind, ErrorCode, FileEntry,
    FileIdentity, ListOptions, Location, LocationId, SortDirection, SortField,
};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub const LIST_LIMIT: usize = 5_000;
const SCAN_LIMIT: usize = 100_000;
const GRANT_LIMIT: usize = 128;

struct Grant {
    path: PathBuf,
    identity: Option<FileIdentity>,
    entries: BTreeMap<EntryId, OsString>,
}

pub struct FileService {
    home: PathBuf,
    next_id: u64,
    grants: BTreeMap<LocationId, Grant>,
}

impl FileService {
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            next_id: 1,
            grants: BTreeMap::new(),
        }
    }

    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn checked(&self, location: LocationId) -> Result<&Grant, AppError> {
        let grant = self
            .grants
            .get(&location)
            .ok_or(ErrorCode::StaleReference)?;
        let metadata = fs::symlink_metadata(&grant.path).map_err(io_error)?;
        if !metadata.is_dir()
            || metadata.is_symlink()
            || fs::canonicalize(&grant.path).map_err(io_error)? != grant.path
            || identity(&metadata) != grant.identity
        {
            return Err(ErrorCode::StaleReference.into());
        }
        Ok(grant)
    }

    fn resolve(&self, target: DirectoryTarget) -> Result<PathBuf, AppError> {
        match target {
            DirectoryTarget::Home => Ok(self.home.clone()),
            DirectoryTarget::Path(text) => {
                if text.is_empty() || text.len() > 32_768 || text.contains('\0') {
                    return Err(ErrorCode::InvalidArgument.into());
                }
                let path = PathBuf::from(text);
                if !path.is_absolute() {
                    return Err(ErrorCode::InvalidArgument.into());
                }
                Ok(path)
            }
            DirectoryTarget::Refresh(id) => Ok(self.checked(id)?.path.clone()),
            DirectoryTarget::Parent(id) => {
                let path = &self.checked(id)?.path;
                Ok(path.parent().unwrap_or(path).to_path_buf())
            }
            DirectoryTarget::Child { location, entry } => {
                let grant = self.checked(location)?;
                let name = grant.entries.get(&entry).ok_or(ErrorCode::StaleReference)?;
                let path = grant.path.join(name);
                if fs::symlink_metadata(&path).map_err(io_error)?.is_symlink() {
                    return Err(ErrorCode::PolicyBlocked.into());
                }
                Ok(path)
            }
        }
    }

    pub fn read(
        &mut self,
        target: DirectoryTarget,
        options: ListOptions,
    ) -> Result<DirectoryListing, AppError> {
        if options.filter.len() > 1024 || options.filter.contains('\0') {
            return Err(ErrorCode::InvalidArgument.into());
        }
        let path = fs::canonicalize(self.resolve(target)?).map_err(io_error)?;
        let metadata = fs::metadata(&path).map_err(io_error)?;
        if !metadata.is_dir() {
            return Err(ErrorCode::InvalidArgument.into());
        }
        let original_identity = identity(&metadata);
        let mut entries = Vec::new();
        let mut names = BTreeMap::new();
        let mut skipped_entries = 0;
        let mut truncated = false;
        let filter = options.filter.to_lowercase();
        for (index, result) in fs::read_dir(&path).map_err(io_error)?.enumerate() {
            if index == SCAN_LIMIT {
                truncated = true;
                break;
            }
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => {
                    skipped_entries += 1;
                    continue;
                }
            };
            let name = entry.file_name();
            let display = name.to_string_lossy().into_owned();
            if !display.to_lowercase().contains(&filter) {
                continue;
            }
            let metadata = fs::symlink_metadata(entry.path());
            if !options.hidden && is_hidden(&name, metadata.as_ref().ok()) {
                continue;
            }
            if entries.len() == LIST_LIMIT {
                truncated = true;
                break;
            }
            let metadata = match metadata {
                Ok(value) => value,
                Err(_) => {
                    skipped_entries += 1;
                    continue;
                }
            };
            let kind = if metadata.is_symlink() {
                EntryKind::Symlink
            } else if metadata.is_dir() {
                EntryKind::Directory
            } else if metadata.is_file() {
                EntryKind::File
            } else {
                EntryKind::Other
            };
            let id = EntryId::from_raw(self.id());
            names.insert(id, name);
            entries.push(FileEntry {
                id,
                name: display,
                kind,
                size_bytes: metadata.is_file().then_some(metadata.len()),
                modified_unix_seconds: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs()),
            });
        }
        // A replaced directory must not silently inherit a previous grant.
        let after = fs::symlink_metadata(&path).map_err(io_error)?;
        if after.is_symlink()
            || identity(&after) != original_identity
            || fs::canonicalize(&path).map_err(io_error)? != path
        {
            return Err(ErrorCode::StaleReference.into());
        }
        entries.sort_by(|a, b| {
            let dir_cmp = (b.kind == EntryKind::Directory).cmp(&(a.kind == EntryKind::Directory));
            if dir_cmp != std::cmp::Ordering::Equal {
                return dir_cmp;
            }
            let ordering = match options.sort.field {
                SortField::Name => a
                    .name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then_with(|| a.name.cmp(&b.name)),
                SortField::Size => a
                    .size_bytes
                    .cmp(&b.size_bytes)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortField::Modified => a
                    .modified_unix_seconds
                    .cmp(&b.modified_unix_seconds)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            };
            match options.sort.direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
        let location = Location {
            id: LocationId::from_raw(self.id()),
            display_path: path.to_string_lossy().into_owned(),
            can_go_up: path.parent().is_some(),
        };
        self.grants.insert(
            location.id,
            Grant {
                path,
                identity: original_identity,
                entries: names,
            },
        );
        if self.grants.len() > GRANT_LIMIT {
            self.grants.pop_first();
        }
        Ok(DirectoryListing {
            location,
            entries,
            truncated,
            limit: LIST_LIMIT,
            skipped_entries,
        })
    }
}

fn io_error(error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::NotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        std::io::ErrorKind::NotADirectory | std::io::ErrorKind::InvalidInput => {
            ErrorCode::InvalidArgument
        }
        _ => ErrorCode::RootUnavailable,
    }
    .into()
}

#[cfg(unix)]
fn identity(metadata: &Metadata) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    Some(FileIdentity {
        volume: metadata.dev(),
        file: u128::from(metadata.ino()),
    })
}
#[cfg(not(unix))]
fn identity(_metadata: &Metadata) -> Option<FileIdentity> {
    None
}

fn is_hidden(name: &std::ffi::OsStr, _metadata: Option<&Metadata>) -> bool {
    let dot = name.to_string_lossy().starts_with('.');
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        dot || _metadata.is_some_and(|m| m.file_attributes() & 0x2 != 0)
    }
    #[cfg(not(windows))]
    {
        dot
    }
}

/// Bootstrap ishlatadi; testlar doim o‘z temp katalogini beradi.
pub fn home_directory() -> PathBuf {
    std::env::home_dir().unwrap_or_else(|| Path::new(std::path::MAIN_SEPARATOR_STR).to_path_buf())
}
