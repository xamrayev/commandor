//! OS launcher: faylni tizimdagi standart dasturda yoki tanlangan dasturda ochish.
//!
//! Tashqi dasturlar faqat argument massivi bilan, shell’siz ishga tushiriladi; stdio ulanmaydi.
//! Fayl yo‘li broker grantidan keladi (UI yo‘l yubormaydi).
//!
//! | OS | Standart | Open with |
//! |---|---|---|
//! | macOS | `/usr/bin/open <fayl>` | `Info.plist` katalogi, `/usr/bin/open -a <app> <fayl>` |
//! | Linux/BSD | `xdg-open <fayl>` | `.desktop` yozuvlari + `xdg-mime`, `Exec` argv |
//! | Windows | `explorer.exe <fayl>` | Tizim dialogi: `rundll32 shell32.dll,OpenAs_RunDLL` |

// Har bir platforma moduli o‘z OSida ishlatiladi; qolganlarida faqat testlar (CI) uni ishlatadi.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
mod desktop;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod macos;
mod risk;
mod spawn;

pub use risk::looks_executable;

use commandor_core::{AppChoice, AppError, AppId, ErrorCode, OpenOutcome, OpenWith, OpenWithList};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Ro‘yxatdagi dastur qanday ishga tushiriladi (faqat launcher ichida).
#[derive(Debug, Clone)]
#[allow(dead_code)] // har OS faqat o‘z variantini yaratadi
enum Target {
    Bundle(PathBuf),
    Desktop(desktop::DesktopEntry),
}

#[allow(dead_code)] // Windows’da nomzodlar yo‘q
struct Candidate {
    name: String,
    detail: String,
    recommended: bool,
    is_default: bool,
    target: Target,
}

/// Broker’ning launcher threadida yashaydi. `AppId`lar oxirgi ro‘yxat uchun amal qiladi.
#[derive(Default)]
pub struct Launcher {
    targets: BTreeMap<AppId, Target>,
    next_id: u64,
    #[cfg(target_os = "macos")]
    catalog: Option<(std::time::Instant, macos::Catalog)>,
}

impl Launcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// "Open with" ro‘yxati: tavsiya etilganlar (nom bo‘yicha), keyin qolganlari.
    pub fn list_open_with(&mut self, file: &Path) -> OpenWithList {
        let mut candidates = self.candidates(file);
        candidates.sort_by(|a, b| {
            (!a.recommended, !a.is_default, a.name.to_lowercase()).cmp(&(
                !b.recommended,
                !b.is_default,
                b.name.to_lowercase(),
            ))
        });
        self.targets.clear();
        let apps = candidates
            .into_iter()
            .map(|candidate| {
                self.next_id += 1;
                let id = AppId::from_raw(self.next_id);
                self.targets.insert(id, candidate.target);
                AppChoice {
                    id,
                    name: candidate.name,
                    detail: candidate.detail,
                    recommended: candidate.recommended,
                    is_default: candidate.is_default,
                }
            })
            .collect();
        OpenWithList {
            apps,
            system_chooser: cfg!(windows),
        }
    }

    /// Faylni ochadi. Standart ochishda bajariladigan fayl tasdiqsiz ishga tushirilmaydi.
    pub fn open(
        &mut self,
        file: &Path,
        with: OpenWith,
        allow_executable: bool,
    ) -> Result<OpenOutcome, AppError> {
        match with {
            OpenWith::Default => {
                if !allow_executable && looks_executable(file, executable_bit(file)) {
                    return Ok(OpenOutcome::NeedsConfirmation);
                }
                platform::open_default(file)?;
            }
            OpenWith::App(id) => {
                let target = self.targets.get(&id).ok_or(ErrorCode::StaleReference)?;
                platform::open_with(target, file)?;
            }
            OpenWith::SystemChooser => platform::system_chooser(file)?,
        }
        Ok(OpenOutcome::Launched)
    }

    #[cfg(target_os = "macos")]
    fn candidates(&mut self, file: &Path) -> Vec<Candidate> {
        const TTL: std::time::Duration = std::time::Duration::from_secs(60);
        let fresh = self
            .catalog
            .as_ref()
            .is_some_and(|(at, _)| at.elapsed() < TTL);
        if !fresh {
            self.catalog = Some((std::time::Instant::now(), macos::Catalog::system()));
        }
        let Some((_, catalog)) = &self.catalog else {
            return Vec::new();
        };
        let ext = file.extension().and_then(|ext| ext.to_str());
        let types = ext
            .map(|ext| catalog.types_for_extension(ext))
            .unwrap_or_default();
        catalog
            .apps
            .iter()
            .map(|app| Candidate {
                name: app.name.clone(),
                detail: app
                    .path
                    .parent()
                    .map(|dir| dir.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                recommended: macos::Catalog::fit(app, ext, &types) == macos::Fit::Specific,
                is_default: false,
                target: Target::Bundle(app.path.clone()),
            })
            .collect()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn candidates(&mut self, file: &Path) -> Vec<Candidate> {
        let xdg_mime = Path::new("xdg-mime");
        let mime = spawn::capture(
            xdg_mime,
            &["query".as_ref(), "filetype".as_ref(), file.as_os_str()],
        )
        .map(|mime| mime.to_ascii_lowercase());
        let default = mime.as_deref().and_then(|mime| {
            spawn::capture(
                xdg_mime,
                &["query".as_ref(), "default".as_ref(), mime.as_ref()],
            )
        });
        let dirs = desktop::application_dirs(
            std::env::var_os("XDG_DATA_HOME"),
            std::env::var_os("XDG_DATA_DIRS"),
            std::env::home_dir(),
        );
        desktop::scan(&dirs)
            .into_iter()
            .filter_map(|entry| {
                let recommended = mime.as_deref().is_some_and(|mime| {
                    entry
                        .mime_types
                        .iter()
                        .any(|pattern| desktop::mime_matches(pattern, mime))
                });
                if entry.no_display && !recommended {
                    return None;
                }
                Some(Candidate {
                    name: entry.name.clone(),
                    detail: entry.id.clone(),
                    recommended,
                    is_default: default.as_deref() == Some(entry.id.as_str()),
                    target: Target::Desktop(entry),
                })
            })
            .collect()
    }

    /// Windows: o‘z ro‘yxatimiz yo‘q, tizim dialogi ishlatiladi.
    #[cfg(not(unix))]
    fn candidates(&mut self, _file: &Path) -> Vec<Candidate> {
        Vec::new()
    }
}

#[cfg(unix)]
fn executable_bit(file: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(file).is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable_bit(_file: &Path) -> bool {
    false
}

#[cfg(target_os = "macos")]
mod platform {
    use super::Target;
    use crate::spawn::{Wait, launch};
    use commandor_core::{AppError, ErrorCode};
    use std::path::Path;

    const OPEN: &str = "/usr/bin/open";

    pub(super) fn open_default(file: &Path) -> Result<(), AppError> {
        // Yo‘l mutlaq (`/` bilan boshlanadi), shuning uchun `open` uni opsiya deb o‘qimaydi.
        launch(Path::new(OPEN), &[file.into()], Wait::ExitCode)
    }

    pub(super) fn open_with(target: &Target, file: &Path) -> Result<(), AppError> {
        let Target::Bundle(app) = target else {
            return Err(ErrorCode::InvalidArgument.into());
        };
        launch(
            Path::new(OPEN),
            &["-a".into(), app.into(), file.into()],
            Wait::ExitCode,
        )
    }

    pub(super) fn system_chooser(_file: &Path) -> Result<(), AppError> {
        Err(ErrorCode::InvalidArgument.into())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::Target;
    use crate::desktop::exec_argv;
    use crate::spawn::{Wait, launch};
    use commandor_core::{AppError, ErrorCode};
    use std::path::Path;

    pub(super) fn open_default(file: &Path) -> Result<(), AppError> {
        launch(Path::new("xdg-open"), &[file.into()], Wait::ExitCode)
    }

    pub(super) fn open_with(target: &Target, file: &Path) -> Result<(), AppError> {
        let Target::Desktop(entry) = target else {
            return Err(ErrorCode::InvalidArgument.into());
        };
        let argv = exec_argv(entry, file).ok_or(ErrorCode::InvalidArgument)?;
        let (program, args) = argv.split_first().ok_or(ErrorCode::InvalidArgument)?;
        launch(Path::new(program), args, Wait::SpawnOnly)
    }

    pub(super) fn system_chooser(_file: &Path) -> Result<(), AppError> {
        Err(ErrorCode::InvalidArgument.into())
    }
}

#[cfg(windows)]
mod platform {
    use super::Target;
    use crate::spawn::{Wait, launch, launch_command};
    use commandor_core::{AppError, ErrorCode};
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    fn system_root() -> PathBuf {
        std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
    }

    pub(super) fn open_default(file: &Path) -> Result<(), AppError> {
        // explorer.exe exit kodi muvaffaqiyatda ham 1 bo‘lishi mumkin — faqat spawn tekshiriladi.
        launch(
            &system_root().join("explorer.exe"),
            &[file.into()],
            Wait::SpawnOnly,
        )
    }

    pub(super) fn open_with(_target: &Target, _file: &Path) -> Result<(), AppError> {
        Err(ErrorCode::InvalidArgument.into())
    }

    pub(super) fn system_chooser(file: &Path) -> Result<(), AppError> {
        // OpenAs_RunDLL qator qoldig‘ini yo‘l sifatida oladi; Windows yo‘lida `"` bo‘lmaydi,
        // shuning uchun yo‘l qo‘shtirnoqsiz, lekin baribir shell’siz uzatiladi.
        let path = file.as_os_str();
        if path.to_string_lossy().contains('"') {
            return Err(ErrorCode::InvalidArgument.into());
        }
        let mut command = Command::new(system_root().join(r"System32\rundll32.exe"));
        command
            .arg("shell32.dll,OpenAs_RunDLL")
            .raw_arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        launch_command(command, Wait::SpawnOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_app_id_is_stale_and_chooser_is_windows_only() {
        let dir = tempfile::tempdir().expect("temp");
        let file = dir.path().join("hujjat.txt");
        std::fs::write(&file, "x").expect("write");
        let mut launcher = Launcher::new();
        let error = launcher
            .open(&file, OpenWith::App(AppId::from_raw(42)), false)
            .expect_err("stale");
        assert_eq!(error.code, ErrorCode::StaleReference);
        if !cfg!(windows) {
            let error = launcher
                .open(&file, OpenWith::SystemChooser, false)
                .expect_err("chooser");
            assert_eq!(error.code, ErrorCode::InvalidArgument);
        }
    }

    #[test]
    fn executable_default_open_asks_for_confirmation_first() {
        let dir = tempfile::tempdir().expect("temp");
        let script = dir.path().join("o‘rnatish.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").expect("write");
        let mut launcher = Launcher::new();
        assert_eq!(
            launcher.open(&script, OpenWith::Default, false),
            Ok(OpenOutcome::NeedsConfirmation),
            "tasdiqsiz hech narsa ishga tushmaydi"
        );
    }

    #[cfg(unix)]
    #[test]
    fn extensionless_file_with_exec_bit_needs_confirmation() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("temp");
        let tool = dir.path().join("tool");
        std::fs::write(&tool, "#!/bin/sh\n").expect("write");
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        assert_eq!(
            Launcher::new().open(&tool, OpenWith::Default, false),
            Ok(OpenOutcome::NeedsConfirmation)
        );
    }

    #[test]
    fn list_assigns_fresh_ids_and_invalidates_old_ones() {
        let dir = tempfile::tempdir().expect("temp");
        let file = dir.path().join("rasm.png");
        std::fs::write(&file, "x").expect("write");
        let mut launcher = Launcher::new();
        let first = launcher.list_open_with(&file);
        assert_eq!(first.system_chooser, cfg!(windows));
        let second = launcher.list_open_with(&file);
        if let (Some(old), Some(new)) = (first.apps.first(), second.apps.first()) {
            assert_ne!(old.id, new.id);
            assert!(!launcher.targets.contains_key(&old.id));
        }
        // Tavsiya etilganlar doim ro‘yxat boshida.
        let first_other = second.apps.iter().position(|app| !app.recommended);
        if let Some(index) = first_other {
            assert!(second.apps[index..].iter().all(|app| !app.recommended));
        }
    }
}

/// Haqiqiy tizim katalogi bo‘yicha qo‘lda ishlatiladigan tekshiruv (hech narsa ishga tushirilmaydi):
/// `cargo test -p commandor-launch -- --ignored --nocapture`
#[cfg(all(test, target_os = "macos"))]
#[allow(clippy::print_stdout)] // natija `--nocapture` bilan o‘qiladi
mod system_catalog {
    use super::*;

    #[test]
    #[ignore = "mashinadagi o‘rnatilgan dasturlarga bog‘liq"]
    fn real_catalog_recommends_known_types() {
        let dir = tempfile::tempdir().expect("temp");
        let started = std::time::Instant::now();
        let mut launcher = Launcher::new();
        for name in [
            "hujjat.txt",
            "rasm.png",
            "hisobot.pdf",
            "eslatma.md",
            "noma’lum.qwerty",
        ] {
            let file = dir.path().join(name);
            std::fs::write(&file, "x").expect("write");
            let list = launcher.list_open_with(&file);
            let recommended: Vec<_> = list
                .apps
                .iter()
                .filter(|app| app.recommended)
                .map(|app| app.name.as_str())
                .collect();
            println!(
                "{name}: jami {}, tavsiya {}: {:?}",
                list.apps.len(),
                recommended.len(),
                recommended
                    .iter()
                    .filter(|n| ["Preview", "TextEdit", "Xcode", "Safari", "Books"].contains(n))
                    .collect::<Vec<_>>()
            );
        }
        println!("vaqt (birinchi skan + 4 kesh): {:?}", started.elapsed());
    }
}
