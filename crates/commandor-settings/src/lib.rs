//! Sozlamalar fayli (UX-09): `settings.toml` OS konfiguratsiya papkasida.
//!
//! - Fayl yo‘q — sukut qiymatlar, xato emas.
//! - Fayl buzilgan yoki juda katta — sukut qiymatlar va `problem`; asl fayl
//!   `settings.toml.broken` nomi bilan saqlanadi (keyingi saqlash uni yo‘qotmaydi).
//! - Saqlash atomik: `.tmp` → `sync_all` → `rename`.

use commandor_core::{AppError, DetailValue, ErrorCode, Settings, SettingsLoad};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "settings.toml";
const MAX_BYTES: u64 = 1 << 20;

/// Standart joy: macOS `~/Library/Application Support/Commandor`, Windows `%APPDATA%\Commandor`,
/// Linux/BSD `$XDG_CONFIG_HOME/commandor` yoki `~/.config/commandor`.
pub fn default_path() -> Option<PathBuf> {
    let dir = if cfg!(target_os = "macos") {
        std::env::home_dir()?.join("Library/Application Support/Commandor")
    } else if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?).join("Commandor")
    } else {
        match std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
            Some(dir) => PathBuf::from(dir).join("commandor"),
            None => std::env::home_dir()?.join(".config/commandor"),
        }
    };
    Some(dir.join(FILE_NAME))
}

fn problem(reason: &str) -> AppError {
    AppError::new(ErrorCode::InvalidArgument)
        .with_detail("reason", DetailValue::Text(reason.into()))
}

fn io_error(error: &std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem => {
            ErrorCode::PermissionDenied
        }
        std::io::ErrorKind::StorageFull => ErrorCode::DiskFull,
        _ => ErrorCode::RootUnavailable,
    }
    .into()
}

/// `path: None` — saqlash o‘chiq (testlar, smoke): o‘qish sukut beradi, yozish hech narsa qilmaydi.
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: Option<PathBuf>,
}

impl SettingsStore {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn load(&self) -> SettingsLoad {
        let fallback = |problem| SettingsLoad {
            settings: Settings::default(),
            problem,
        };
        let Some(path) = &self.path else {
            return fallback(None);
        };
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return fallback(None),
            Err(error) => return fallback(Some(io_error(&error))),
        };
        let mut text = String::new();
        let read = Read::by_ref(&mut file)
            .take(MAX_BYTES + 1)
            .read_to_string(&mut text);
        let parsed = match read {
            Ok(len) if len as u64 > MAX_BYTES => Err(problem("settings_too_large")),
            Ok(_) => toml::from_str::<Settings>(&text).map_err(|_| problem("settings_invalid")),
            // UTF-8 bo‘lmagan fayl ham buzilgan hisoblanadi.
            Err(_) => Err(problem("settings_invalid")),
        };
        match parsed {
            Ok(settings) => SettingsLoad {
                settings,
                problem: None,
            },
            Err(error) => {
                let _ = fs::copy(path, path.with_extension("toml.broken"));
                fallback(Some(error))
            }
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), AppError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let text = toml::to_string_pretty(settings).map_err(|_| ErrorCode::Internal)?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|error| io_error(&error))?;
        }
        let tmp = path.with_extension("toml.tmp");
        let result = (|| {
            let mut file = File::create(&tmp)?;
            file.write_all(b"# Commandor sozlamalari. Ilova ishlayotganda o\xe2\x80\x98zgartirilsa, ilova yopilganda qayta yoziladi.\n")?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            fs::rename(&tmp, path)
        })();
        result.map_err(|error| {
            let _ = fs::remove_file(&tmp);
            io_error(&error)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commandor_core::{PanelSession, SortDirection, SortField, TabSession};

    fn sample() -> Settings {
        let mut settings = Settings {
            bookmarks: vec!["/Users/me/docs".into(), "/Volumes/USB".into()],
            ..Settings::default()
        };
        settings.session.active = 1;
        settings.session.left = PanelSession {
            current: 1,
            tabs: vec![
                TabSession {
                    path: "/Users/me".into(),
                    ..TabSession::default()
                },
                TabSession {
                    path: "/Users/me/O‘zbek papka".into(),
                    hidden: true,
                    sort: commandor_core::SortCriteria {
                        field: SortField::Size,
                        direction: SortDirection::Descending,
                    },
                },
            ],
        };
        settings
    }

    fn store(dir: &Path) -> SettingsStore {
        SettingsStore::new(Some(dir.join("conf/settings.toml")))
    }

    #[test]
    fn roundtrip_creates_directory_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().expect("temp");
        let store = store(dir.path());
        assert_eq!(
            store.load(),
            SettingsLoad {
                settings: Settings::default(),
                problem: None
            },
            "fayl yo‘q — sukut, xato emas"
        );
        store.save(&sample()).expect("save");
        let loaded = store.load();
        assert_eq!(loaded.problem, None);
        assert_eq!(loaded.settings, sample());
        let names: Vec<_> = fs::read_dir(dir.path().join("conf"))
            .expect("dir")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["settings.toml"]);
        let text = fs::read_to_string(dir.path().join("conf/settings.toml")).expect("read");
        assert!(text.starts_with("# Commandor"), "{text}");
    }

    #[test]
    fn partial_and_unknown_fields_are_tolerated() {
        let dir = tempfile::tempdir().expect("temp");
        let path = dir.path().join("settings.toml");
        fs::write(
            &path,
            "bookmarks = [\"/a\"]\nkelajak_maydoni = 5\n[ui]\nrestore_session = false\n",
        )
        .expect("write");
        let loaded = SettingsStore::new(Some(path)).load();
        assert_eq!(loaded.problem, None);
        assert_eq!(loaded.settings.bookmarks, ["/a"]);
        assert!(!loaded.settings.ui.restore_session);
        assert_eq!(loaded.settings.version, commandor_core::SETTINGS_VERSION);
    }

    #[test]
    fn broken_file_gives_defaults_and_is_kept_aside() {
        let dir = tempfile::tempdir().expect("temp");
        let path = dir.path().join("settings.toml");
        fs::write(&path, "bookmarks = [ \"yopilmagan").expect("write");
        let store = SettingsStore::new(Some(path.clone()));
        let loaded = store.load();
        assert_eq!(loaded.settings, Settings::default());
        let problem = loaded.problem.expect("problem");
        assert_eq!(problem.code, ErrorCode::InvalidArgument);
        assert_eq!(
            problem.details.get("reason"),
            Some(&DetailValue::Text("settings_invalid".into()))
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("settings.toml.broken")).expect("backup"),
            "bookmarks = [ \"yopilmagan"
        );
        store.save(&sample()).expect("save");
        assert!(
            dir.path().join("settings.toml.broken").exists(),
            "zaxira qoladi"
        );

        fs::write(&path, vec![0xff_u8, 0xfe, 0x00]).expect("write");
        assert!(store.load().problem.is_some(), "UTF-8 emas");
        fs::write(&path, "#".repeat((MAX_BYTES + 10) as usize)).expect("write");
        let large = store.load().problem.expect("large");
        assert_eq!(
            large.details.get("reason"),
            Some(&DetailValue::Text("settings_too_large".into()))
        );
    }

    #[test]
    fn disabled_store_never_touches_disk() {
        let store = SettingsStore::new(None);
        assert_eq!(store.load().settings, Settings::default());
        store.save(&sample()).expect("no-op");
        assert!(store.path().is_none());
    }

    #[test]
    fn default_path_ends_with_settings_file() {
        if let Some(path) = default_path() {
            assert!(path.ends_with("settings.toml"));
            assert!(path.is_absolute());
        }
    }
}
