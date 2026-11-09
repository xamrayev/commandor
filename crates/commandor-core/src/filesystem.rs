//! Listing shartnomasi. Disk yo‘li va OsStringlar servisda qoladi; UI opaque ID ishlatadi.

use crate::{EntryId, LocationId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectoryTarget {
    Home,
    Path(String),
    Refresh(LocationId),
    Parent(LocationId),
    Child {
        location: LocationId,
        entry: EntryId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SortCriteria {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListOptions {
    pub hidden: bool,
    pub filter: String,
    pub sort: SortCriteria,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub display_path: String,
    pub can_go_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: EntryId,
    pub name: String,
    pub kind: EntryKind,
    pub size_bytes: Option<u64>,
    pub modified_unix_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub location: Location,
    pub entries: Vec<FileEntry>,
    pub truncated: bool,
    pub limit: usize,
    pub skipped_entries: usize,
}

/// Tom (disk, bo‘lim, mount nuqtasi) turi — UI ikonka va rangni shunga ko‘ra tanlaydi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeKind {
    /// Tizim ishga tushgan tom (`/` yoki Windows tizim drayveri).
    System,
    /// Ichki qo‘shimcha disk yoki bo‘lim.
    Internal,
    /// Olinadigan qurilma (USB, SD karta).
    Removable,
    /// Tarmoq ulanishi (SMB, NFS, AFP, WebDAV).
    Network,
    Other,
}

/// Foydalanuvchiga ko‘rinadigan tom. `mount_path` faqat `DirectoryTarget::Path` sifatida
/// qayta yuboriladi; broker uni oddiy yo‘l kabi tekshiradi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub mount_path: String,
    pub kind: VolumeKind,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub read_only: bool,
}

/// `path` qaysi tomga tegishli: eng uzun mos keluvchi mount nuqtasi indeksi.
/// Komponent chegarasi hisobga olinadi (`/Volumes/A` → `/Volumes/AB` ga mos emas);
/// Windows drayv harflari katta-kichik harfga sezgir emas.
pub fn volume_index_for_path(volumes: &[Volume], path: &str) -> Option<usize> {
    volumes
        .iter()
        .enumerate()
        .filter(|(_, volume)| path_is_under(path, &volume.mount_path))
        .max_by_key(|(_, volume)| volume.mount_path.len())
        .map(|(index, _)| index)
}

fn path_is_under(path: &str, mount: &str) -> bool {
    if mount.is_empty() {
        return false;
    }
    let drive_like = mount.as_bytes().get(1) == Some(&b':');
    let (path, mount) = if drive_like {
        (path.to_ascii_lowercase(), mount.to_ascii_lowercase())
    } else {
        (path.to_owned(), mount.to_owned())
    };
    let trimmed = mount.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        // Ildiz (`/`): har qanday mutlaq unix yo‘li unga tegishli.
        return path.starts_with('/');
    }
    match path.strip_prefix(trimmed) {
        Some("") => true,
        Some(rest) => rest.starts_with(['/', '\\']),
        None => false,
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;

    fn volume(mount: &str) -> Volume {
        Volume {
            name: mount.into(),
            mount_path: mount.into(),
            kind: VolumeKind::Internal,
            file_system: String::new(),
            total_bytes: 0,
            available_bytes: 0,
            read_only: false,
        }
    }

    #[test]
    fn longest_mount_wins_and_respects_component_boundary() {
        let list = [volume("/"), volume("/Volumes/A"), volume("/Volumes/AB")];
        assert_eq!(volume_index_for_path(&list, "/Users/x"), Some(0));
        assert_eq!(volume_index_for_path(&list, "/Volumes/A"), Some(1));
        assert_eq!(volume_index_for_path(&list, "/Volumes/A/docs"), Some(1));
        assert_eq!(volume_index_for_path(&list, "/Volumes/AB/docs"), Some(2));
    }

    #[test]
    fn windows_drives_are_case_insensitive() {
        let list = [volume("C:\\"), volume("D:\\")];
        assert_eq!(volume_index_for_path(&list, "d:\\Projects"), Some(1));
        assert_eq!(volume_index_for_path(&list, "C:\\"), Some(0));
        assert_eq!(volume_index_for_path(&list, "E:\\x"), None);
    }
}
