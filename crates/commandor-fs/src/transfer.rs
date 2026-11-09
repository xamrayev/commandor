//! Nusxalash va ko‘chirish (FS-02, FS-04, FS-07): fon vazifasi, progress, bekor qilish.
//!
//! - Fayl avval maqsad papkadagi vaqtinchalik `.part` faylga yoziladi, `sync_all`dan keyin
//!   rename qilinadi: bekor qilish yoki xato yarim faylni to‘liq deb qoldirmaydi.
//! - Symlinklar kuzatilmaydi: Unix’da link sifatida nusxalanadi, Windows’da rad etiladi.
//! - Ko‘chirish: bir diskda `rename`; disklararo — nusxa → hajm tekshiruvi → manbani o‘chirish.

use crate::ops::{Source, still_same};
use crate::{identity, io_error};
use commandor_core::{
    AppError, ConflictPolicy, ErrorCode, ItemFailure, OperationReport, PlanAction, Totals,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const CHUNK: usize = 1 << 20;
const SCAN_ENTRY_LIMIT: u64 = 200_000;
const SCAN_TIME_LIMIT: Duration = Duration::from_secs(3);
const PART_SUFFIX: &str = ".commandor-part";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    Copy,
    Move,
}

/// Vazifa davomidagi holat; har fayl bo‘lagidan keyin callback chaqiriladi.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Progress {
    pub done_items: u64,
    pub done_bytes: u64,
    pub current: String,
}

/// File threadda tuziladi, jobs threadda bajariladi (yo‘llar grantdan olingan).
#[derive(Debug)]
pub struct TransferJob {
    pub(crate) mode: TransferMode,
    pub(crate) sources: Vec<(Source, String)>,
    pub(crate) target_dir: PathBuf,
    pub(crate) policy: ConflictPolicy,
    pub(crate) directory: String,
    pub(crate) target_display: String,
    pub(crate) totals: Option<Totals>,
}

/// Yozuvni (papka bo‘lsa ichidagilar bilan) sanaydi; symlink ichiga kirilmaydi.
/// Limitdan oshsa `None` (progress foizsiz ko‘rsatiladi).
pub(crate) fn scan(path: &Path) -> Option<Totals> {
    let started = Instant::now();
    let mut totals = Totals::default();
    let mut stack = vec![path.to_path_buf()];
    while let Some(path) = stack.pop() {
        if totals.items >= SCAN_ENTRY_LIMIT || started.elapsed() > SCAN_TIME_LIMIT {
            return None;
        }
        let metadata = fs::symlink_metadata(&path).ok()?;
        totals.items += 1;
        if metadata.is_file() {
            totals.bytes += metadata.len();
        } else if metadata.is_dir() {
            for entry in fs::read_dir(&path).ok()? {
                stack.push(entry.ok()?.path());
            }
        }
    }
    Some(totals)
}

/// `nom (2).ext`, `nom (3).ext`, ... — bo‘sh nom topilguncha.
pub(crate) fn unique_name(dir: &Path, name: &str) -> Result<PathBuf, AppError> {
    let (stem, ext) = match name.rfind('.') {
        Some(index) if index > 0 => (&name[..index], &name[index..]),
        _ => (name, ""),
    };
    for n in 2..10_000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if fs::symlink_metadata(&candidate).is_err() {
            return Ok(candidate);
        }
    }
    Err(ErrorCode::DestinationExists.into())
}

struct Run<'a> {
    cancel: &'a AtomicBool,
    progress: Progress,
    callback: &'a mut dyn FnMut(&Progress),
}

impl Run<'_> {
    fn check(&self) -> Result<(), AppError> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(ErrorCode::Cancelled.into())
        } else {
            Ok(())
        }
    }

    fn tick(&mut self, items: u64, bytes: u64) {
        self.progress.done_items += items;
        self.progress.done_bytes += bytes;
        (self.callback)(&self.progress);
    }

    /// Faylni `.part` orqali nusxalaydi. `replace` bo‘lmasa mavjud maqsad tegilmaydi.
    fn copy_file(&mut self, src: &Path, dst: &Path, replace: bool) -> Result<u64, AppError> {
        let name = dst.file_name().ok_or(ErrorCode::InvalidArgument)?;
        let part = dst.with_file_name(format!(".{}{PART_SUFFIX}", name.to_string_lossy()));
        let result = self.copy_to_part(src, &part).and_then(|copied| {
            if !replace && fs::symlink_metadata(dst).is_ok() {
                return Err(ErrorCode::DestinationExists.into());
            }
            fs::rename(&part, dst).map_err(io_error)?;
            Ok(copied)
        });
        if result.is_err() {
            let _ = fs::remove_file(&part);
        }
        result
    }

    fn copy_to_part(&mut self, src: &Path, part: &Path) -> Result<u64, AppError> {
        let mut input = File::open(src).map_err(io_error)?;
        let metadata = input.metadata().map_err(io_error)?;
        // `create_new`: oldingi muvaffaqiyatsiz urinishdan qolgan begona faylni bosib yozmaydi.
        let _ = fs::remove_file(part);
        let mut output = File::create_new(part).map_err(io_error)?;
        let mut buffer = vec![0_u8; CHUNK];
        let mut copied = 0_u64;
        loop {
            self.check()?;
            let read = input.read(&mut buffer).map_err(io_error)?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read]).map_err(io_error)?;
            copied += read as u64;
            self.tick(0, read as u64);
        }
        output.sync_all().map_err(io_error)?;
        let _ = output.set_permissions(metadata.permissions());
        if let Ok(modified) = metadata.modified() {
            let _ = output.set_modified(modified);
        }
        if copied != metadata.len() {
            // Manba nusxalash davomida o‘zgargan.
            return Err(ErrorCode::Conflict.into());
        }
        Ok(copied)
    }

    /// Yozuvni (rekursiv) nusxalaydi. Papka mavjud bo‘lsa ichiga qo‘shadi (`merge`).
    fn copy_tree(&mut self, src: &Path, dst: &Path, replace: bool) -> Result<u64, AppError> {
        self.check()?;
        let metadata = fs::symlink_metadata(src).map_err(io_error)?;
        self.progress.current = src
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if metadata.is_symlink() {
            copy_symlink(src, dst, replace)?;
            self.tick(1, 0);
            return Ok(0);
        }
        if metadata.is_file() {
            let bytes = self.copy_file(src, dst, replace)?;
            self.tick(1, 0);
            return Ok(bytes);
        }
        if !metadata.is_dir() {
            return Err(ErrorCode::UnsupportedFormat.into());
        }
        match fs::symlink_metadata(dst) {
            Ok(existing) if existing.is_dir() && !existing.is_symlink() && replace => {}
            Ok(_) => return Err(ErrorCode::DestinationExists.into()),
            Err(_) => fs::create_dir(dst).map_err(io_error)?,
        }
        self.tick(1, 0);
        let mut total = 0;
        let mut entries: Vec<_> = fs::read_dir(src)
            .map_err(io_error)?
            .collect::<Result<_, _>>()
            .map_err(io_error)?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let child_dst = dst.join(entry.file_name());
            total += self.copy_tree(&entry.path(), &child_dst, replace)?;
        }
        let _ = fs::set_permissions(dst, metadata.permissions());
        Ok(total)
    }
}

#[cfg(unix)]
fn copy_symlink(src: &Path, dst: &Path, replace: bool) -> Result<(), AppError> {
    let target = fs::read_link(src).map_err(io_error)?;
    if fs::symlink_metadata(dst).is_ok() {
        if !replace {
            return Err(ErrorCode::DestinationExists.into());
        }
        if fs::symlink_metadata(dst).is_ok_and(|m| m.is_dir() && !m.is_symlink()) {
            return Err(ErrorCode::DestinationExists.into());
        }
        fs::remove_file(dst).map_err(io_error)?;
    }
    std::os::unix::fs::symlink(target, dst).map_err(io_error)
}

#[cfg(not(unix))]
fn copy_symlink(_src: &Path, _dst: &Path, _replace: bool) -> Result<(), AppError> {
    // Windows’da symlink yaratish alohida huquq talab qiladi; jim kuzatib nusxalamaymiz.
    Err(ErrorCode::PolicyBlocked.into())
}

/// Ko‘chirishdan keyin manbani o‘chiradi (symlink ichiga kirmaydi).
fn remove_source(path: &Path) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.is_dir() && !metadata.is_symlink() {
        fs::remove_dir_all(path).map_err(io_error)
    } else {
        fs::remove_file(path).map_err(io_error)
    }
}

impl TransferJob {
    pub fn action(&self) -> PlanAction {
        match self.mode {
            TransferMode::Copy => PlanAction::Copy,
            TransferMode::Move => PlanAction::Move,
        }
    }

    pub fn totals(&self) -> Option<Totals> {
        self.totals
    }

    /// Vazifani bajaradi. Har yozuv natijasi hisobotda; bekor qilinsa qolganlari tegilmaydi.
    pub fn run(self, cancel: &AtomicBool, callback: &mut dyn FnMut(&Progress)) -> OperationReport {
        let mut report = OperationReport {
            action: self.action(),
            directory: self.directory.clone(),
            done: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            target: Some(self.target_display.clone()),
            cancelled: false,
        };
        let mut run = Run {
            cancel,
            progress: Progress::default(),
            callback,
        };
        for (source, name) in &self.sources {
            if run.check().is_err() {
                report.cancelled = true;
                break;
            }
            match self.transfer_one(&mut run, source, name) {
                Ok(Some(done)) => report.done.push(done),
                Ok(None) => report.skipped.push(name.clone()),
                Err(error) if error.code == ErrorCode::Cancelled => {
                    report.cancelled = true;
                    break;
                }
                Err(error) => report.failed.push(ItemFailure {
                    name: name.clone(),
                    error,
                }),
            }
        }
        report
    }

    /// `Ok(None)` — konflikt sababli o‘tkazib yuborildi.
    fn transfer_one(
        &self,
        run: &mut Run<'_>,
        source: &Source,
        name: &str,
    ) -> Result<Option<String>, AppError> {
        still_same(source)?;
        let mut dest = self.target_dir.join(name);
        let mut replace = false;
        if fs::symlink_metadata(&dest).is_ok() {
            match self.policy {
                ConflictPolicy::Ask => return Err(ErrorCode::Conflict.into()),
                ConflictPolicy::Skip => return Ok(None),
                ConflictPolicy::KeepBoth => dest = unique_name(&self.target_dir, name)?,
                ConflictPolicy::Replace => replace = true,
            }
        }
        let shown = dest
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if self.mode == TransferMode::Move && !replace {
            match fs::rename(&source.path, &dest) {
                Ok(()) => {
                    let totals = scan(&dest).unwrap_or_default();
                    run.tick(totals.items, totals.bytes);
                    return Ok(Some(shown));
                }
                Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {}
                Err(error) => return Err(io_error(error)),
            }
        }
        let expected = scan(&source.path).map(|totals| totals.bytes);
        let copied = run.copy_tree(&source.path, &dest, replace)?;
        if self.mode == TransferMode::Move {
            // Manba faqat to‘liq va hajmi mos nusxadan keyin o‘chiriladi.
            if expected.is_some_and(|bytes| bytes != copied) {
                return Err(ErrorCode::ChecksumMismatch.into());
            }
            still_same(source)?;
            remove_source(&source.path)?;
        }
        Ok(Some(shown))
    }
}

/// Maqsad papka manbaning o‘zi yoki uning ichida emasmi (o‘ziga nusxalash cheksiz bo‘ladi).
pub(crate) fn target_inside(source: &Path, target: &Path) -> bool {
    target == source || target.starts_with(source)
}

/// Reja tuzishda: maqsadda shu nom bormi.
pub(crate) fn conflicts(target_dir: &Path, name: &str) -> bool {
    fs::symlink_metadata(target_dir.join(name)).is_ok()
}

/// Reja xeshi uchun maqsad identifikatori.
pub(crate) fn dir_identity(path: &Path) -> Option<commandor_core::FileIdentity> {
    fs::symlink_metadata(path).ok().and_then(|m| identity(&m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names_keep_extension_and_count_up() {
        let dir = tempfile::tempdir().expect("temp");
        fs::write(dir.path().join("hisobot.txt"), "").expect("w");
        fs::write(dir.path().join("hisobot (2).txt"), "").expect("w");
        assert_eq!(
            unique_name(dir.path(), "hisobot.txt").expect("name"),
            dir.path().join("hisobot (3).txt")
        );
        assert_eq!(
            unique_name(dir.path(), ".config").expect("name"),
            dir.path().join(".config (2)")
        );
        assert_eq!(
            unique_name(dir.path(), "papka").expect("name"),
            dir.path().join("papka (2)")
        );
    }

    #[test]
    fn scan_counts_files_dirs_and_does_not_follow_links() {
        let dir = tempfile::tempdir().expect("temp");
        let root = dir.path().join("a");
        fs::create_dir_all(root.join("b")).expect("mkdir");
        fs::write(root.join("x"), "12345").expect("w");
        fs::write(root.join("b/y"), "123").expect("w");
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path(), root.join("loop")).expect("link");
        let totals = scan(&root).expect("totals");
        assert_eq!(totals.bytes, 8);
        assert_eq!(totals.items, if cfg!(unix) { 5 } else { 4 });
    }

    #[test]
    fn target_inside_source_is_detected() {
        assert!(target_inside(Path::new("/a/b"), Path::new("/a/b")));
        assert!(target_inside(Path::new("/a/b"), Path::new("/a/b/c")));
        assert!(!target_inside(Path::new("/a/b"), Path::new("/a/bc")));
        assert!(!target_inside(Path::new("/a/b"), Path::new("/a")));
    }
}
