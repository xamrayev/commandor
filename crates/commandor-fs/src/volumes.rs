//! Foydalanuvchiga ko‘rinadigan tomlar (disklar) ro‘yxati.
//!
//! Tizimning ichki mountlari (macOS `/System/Volumes/*`, Linux `/proc`, `/sys`, `/run`, ...)
//! yashiriladi: foydalanuvchi uchun macOS’da `/` va `/Volumes/*`, Windows’da drayvlar,
//! Linux’da `/`, `/home`, `/mnt/*`, `/media/*`, `/run/media/*` kabi joylar qoladi.

use commandor_core::{Volume, VolumeKind};
use std::path::Path;
use sysinfo::Disks;

/// Mount nuqtalarini filtrlash qoidalari (testda istalgan OS qoidasi sinaladi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountRules {
    MacOs,
    Linux,
    Windows,
}

impl MountRules {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    /// Mount nuqtasi foydalanuvchiga ko‘rsatiladimi.
    pub fn is_visible(self, mount: &str) -> bool {
        match self {
            Self::Windows => !mount.is_empty(),
            Self::MacOs => {
                mount == "/"
                    || mount
                        .strip_prefix("/Volumes/")
                        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('.'))
            }
            Self::Linux => {
                const HIDDEN: [&str; 9] = [
                    "/proc", "/sys", "/dev", "/run", "/snap", "/boot", "/var/lib", "/tmp", "/efi",
                ];
                if mount.starts_with("/run/media/") {
                    return true;
                }
                !HIDDEN.iter().any(|prefix| {
                    mount == *prefix
                        || mount
                            .strip_prefix(prefix)
                            .is_some_and(|rest| rest.starts_with('/'))
                })
            }
        }
    }
}

/// Fayl tizimi nomi bo‘yicha tarmoq ulanishini aniqlash.
pub fn is_network_fs(file_system: &str) -> bool {
    let fs = file_system.to_ascii_lowercase();
    ["nfs", "smb", "cifs", "afp", "webdav", "sshfs", "9p"]
        .iter()
        .any(|kind| fs.contains(kind))
}

/// Ko‘rsatish nomi: qurilma yo‘li (`/dev/sda1`) yoki bo‘sh nom o‘rniga mount nomi.
pub fn display_name(name: &str, mount: &str) -> String {
    let name = name.trim();
    if !name.is_empty() && !name.starts_with("/dev/") {
        return name.to_owned();
    }
    let base = Path::new(mount)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| mount.to_owned());
    // Windows drayvi (`D:\\`) boshqa OSda ham to‘g‘ri qisqarishi uchun ajratgichlar qo‘lda olinadi.
    let trimmed = base.trim_end_matches(['\\', '/']);
    if trimmed.is_empty() {
        mount.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn is_system_mount(rules: MountRules, mount: &str) -> bool {
    match rules {
        MountRules::Windows => {
            let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_owned());
            mount
                .trim_end_matches('\\')
                .eq_ignore_ascii_case(system.trim_end_matches('\\'))
        }
        MountRules::MacOs | MountRules::Linux => mount == "/",
    }
}

/// Tizimdagi tomlarni o‘qiydi. Xato bo‘lgan disk ro‘yxatdan tushib qoladi, butun so‘rov yiqilmaydi.
pub fn list_volumes() -> Vec<Volume> {
    let rules = MountRules::current();
    let disks = Disks::new_with_refreshed_list();
    let mut volumes: Vec<Volume> = Vec::new();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().into_owned();
        if !rules.is_visible(&mount) || volumes.iter().any(|v| v.mount_path == mount) {
            continue;
        }
        let file_system = disk.file_system().to_string_lossy().into_owned();
        let kind = if is_system_mount(rules, &mount) {
            VolumeKind::System
        } else if is_network_fs(&file_system) {
            VolumeKind::Network
        } else if disk.is_removable() {
            VolumeKind::Removable
        } else {
            VolumeKind::Internal
        };
        volumes.push(Volume {
            name: display_name(&disk.name().to_string_lossy(), &mount),
            mount_path: mount,
            kind,
            file_system,
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
            read_only: disk.is_read_only(),
        });
    }
    volumes.sort_by(|a, b| {
        (a.kind != VolumeKind::System, a.name.to_lowercase())
            .cmp(&(b.kind != VolumeKind::System, b.name.to_lowercase()))
    });
    volumes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_hides_system_internal_mounts() {
        let r = MountRules::MacOs;
        assert!(r.is_visible("/"));
        assert!(r.is_visible("/Volumes/yojik HD"));
        assert!(!r.is_visible("/System/Volumes/Data"));
        assert!(!r.is_visible("/System/Volumes/VM"));
        assert!(!r.is_visible("/Volumes/.timemachine"));
        assert!(!r.is_visible("/private/var/vm"));
    }

    #[test]
    fn linux_hides_pseudo_mounts_but_keeps_user_media() {
        let r = MountRules::Linux;
        for visible in ["/", "/home", "/mnt/data", "/media/usb", "/run/media/u/USB"] {
            assert!(r.is_visible(visible), "{visible}");
        }
        for hidden in [
            "/proc",
            "/sys/fs/cgroup",
            "/run/user/1000",
            "/boot/efi",
            "/snap/core",
        ] {
            assert!(!r.is_visible(hidden), "{hidden}");
        }
        assert!(
            r.is_visible("/runtime"),
            "prefiks komponent chegarasida tekshiriladi"
        );
    }

    #[test]
    fn windows_shows_all_drives() {
        assert!(MountRules::Windows.is_visible("C:\\"));
        assert!(MountRules::Windows.is_visible("E:\\"));
    }

    #[test]
    fn display_name_falls_back_to_mount() {
        assert_eq!(display_name("Macintosh HD", "/"), "Macintosh HD");
        assert_eq!(display_name("/dev/sdb1", "/media/usb"), "usb");
        assert_eq!(display_name("", "D:\\"), "D:");
        assert_eq!(display_name("/dev/nvme0n1p2", "/"), "/");
    }

    #[test]
    fn network_file_systems() {
        assert!(is_network_fs("smbfs"));
        assert!(is_network_fs("nfs4"));
        assert!(is_network_fs("fuse.sshfs"));
        assert!(!is_network_fs("apfs"));
        assert!(!is_network_fs("ext4"));
    }

    #[test]
    fn real_volume_list_contains_a_system_volume() {
        // Faqat o‘qish: haqiqiy disklar ro‘yxati so‘raladi, hech narsa yozilmaydi.
        let volumes = list_volumes();
        assert!(volumes.iter().any(|v| v.kind == VolumeKind::System));
        assert!(
            volumes
                .iter()
                .all(|v| MountRules::current().is_visible(&v.mount_path))
        );
    }
}
