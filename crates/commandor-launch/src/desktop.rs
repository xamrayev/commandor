//! Linux/BSD: freedesktop `.desktop` yozuvlari (Desktop Entry Specification 1.5).
//!
//! `Exec` qatori shell’ga berilmaydi: spetsifikatsiyadagi qo‘shtirnoq va escape qoidalari bilan
//! argumentlarga ajratiladi, `%f/%F/%u/%U` fayl yo‘li bilan almashtiriladi.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopEntry {
    /// Desktop ID (`org.gnome.Evince.desktop`; ichki papka `-` bilan qo‘shiladi).
    pub id: String,
    pub name: String,
    pub exec: String,
    pub mime_types: Vec<String>,
    pub no_display: bool,
    pub path: PathBuf,
}

/// Qiymat darajasidagi escape’lar: `\s`, `\n`, `\t`, `\r`, `\\`.
fn unescape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// `[Desktop Entry]` guruhini o‘qiydi. Ilova bo‘lmagan, yashirin yoki terminal talab qiladigan
/// yozuvlar `None`.
pub(crate) fn parse(id: &str, path: &Path, text: &str) -> Option<DesktopEntry> {
    let mut in_main = false;
    let mut keys = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_main = line == "[Desktop Entry]";
            continue;
        }
        if !in_main {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            // Lokallashtirilgan `Name[uz]` kalitlari hozircha e’tiborsiz.
            keys.entry(key.trim().to_owned())
                .or_insert_with(|| value.trim().to_owned());
        }
    }
    let flag = |key: &str| keys.get(key).is_some_and(|value| value == "true");
    if keys.get("Type").map(String::as_str) != Some("Application")
        || flag("Hidden")
        || flag("Terminal")
    {
        return None;
    }
    let exec = unescape_value(keys.get("Exec")?);
    let name = unescape_value(keys.get("Name")?);
    let mime_types = keys
        .get("MimeType")
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|mime| !mime.is_empty())
                .map(str::to_ascii_lowercase)
                .collect()
        })
        .unwrap_or_default();
    Some(DesktopEntry {
        id: id.to_owned(),
        name,
        exec,
        mime_types,
        no_display: flag("NoDisplay"),
        path: path.to_path_buf(),
    })
}

/// `Exec` qatorini so‘zlarga ajratadi (qo‘shtirnoq ichida `\"`, `` \` ``, `\$`, `\\`).
fn split_exec(exec: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut has_word = false;
    let mut chars = exec.chars();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                has_word = true;
            }
            '\\' if quoted => match chars.next() {
                Some(next @ ('"' | '`' | '$' | '\\')) => current.push(next),
                Some(other) => {
                    current.push('\\');
                    current.push(other);
                }
                None => return None,
            },
            ' ' | '\t' if !quoted => {
                if has_word {
                    words.push(std::mem::take(&mut current));
                    has_word = false;
                }
            }
            _ => {
                current.push(c);
                has_word = true;
            }
        }
    }
    if quoted {
        return None; // yopilmagan qo‘shtirnoq — yozuv noto‘g‘ri
    }
    if has_word {
        words.push(current);
    }
    Some(words)
}

/// Fayl bilan ishga tushirish uchun argv. Field kod bo‘lmasa yo‘l oxiriga qo‘shiladi.
pub(crate) fn exec_argv(entry: &DesktopEntry, file: &Path) -> Option<Vec<OsString>> {
    let mut argv = Vec::new();
    let mut used_file = false;
    for word in split_exec(&entry.exec)? {
        match word.as_str() {
            "%f" | "%F" | "%u" | "%U" => {
                if !used_file {
                    argv.push(file.as_os_str().to_owned());
                    used_file = true;
                }
                continue;
            }
            "%i" | "%d" | "%D" | "%n" | "%N" | "%v" | "%m" => continue,
            _ => {}
        }
        let mut out = String::new();
        let mut chars = word.chars();
        while let Some(c) = chars.next() {
            if c != '%' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('%') => out.push('%'),
                Some('f' | 'F' | 'u' | 'U') if !used_file => {
                    out.push_str(&file.to_string_lossy());
                    used_file = true;
                }
                Some('c') => out.push_str(&entry.name),
                Some('k') => out.push_str(&entry.path.to_string_lossy()),
                _ => {} // noma’lum yoki eskirgan kod tashlanadi
            }
        }
        argv.push(out.into());
    }
    if argv.is_empty() {
        return None;
    }
    if !used_file {
        argv.push(file.as_os_str().to_owned());
    }
    Some(argv)
}

/// `image/*` kabi naqshlar ham qo‘llanadi.
pub(crate) fn mime_matches(pattern: &str, mime: &str) -> bool {
    match pattern.strip_suffix("/*") {
        Some(major) => mime
            .split_once('/')
            .is_some_and(|(file_major, _)| file_major == major),
        None => pattern == mime,
    }
}

/// XDG ma’lumot papkalari ustuvorlik tartibida (`$XDG_DATA_HOME`, so‘ng `$XDG_DATA_DIRS`).
pub(crate) fn application_dirs(
    data_home: Option<OsString>,
    data_dirs: Option<OsString>,
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    match data_home.filter(|value| !value.is_empty()) {
        Some(value) => dirs.push(PathBuf::from(value)),
        None => dirs.extend(home.map(|home| home.join(".local/share"))),
    }
    let system = data_dirs
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs.extend(std::env::split_paths(&system));
    dirs.into_iter()
        .map(|dir| dir.join("applications"))
        .collect()
}

/// Papkalardagi yozuvlar; bir xil ID uchun birinchi (ustuvor) papka g‘olib.
pub(crate) fn scan(dirs: &[PathBuf]) -> Vec<DesktopEntry> {
    let mut found: BTreeMap<String, Option<DesktopEntry>> = BTreeMap::new();
    for dir in dirs {
        scan_dir(dir, "", &mut found, 0);
    }
    found.into_values().flatten().collect()
}

fn scan_dir(
    dir: &Path,
    prefix: &str,
    found: &mut BTreeMap<String, Option<DesktopEntry>>,
    depth: u8,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if found.len() >= MAX_ENTRIES {
            return;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && depth < 3 {
            scan_dir(&path, &format!("{prefix}{name}-"), found, depth + 1);
            continue;
        }
        let Some(stem) = name.strip_suffix(".desktop") else {
            continue;
        };
        let id = format!("{prefix}{stem}.desktop");
        if found.contains_key(&id) {
            continue; // ustuvor papkadagi yozuv (yoki uni yashirgan Hidden=true) saqlanadi
        }
        let parsed = fs::read_to_string(&path)
            .ok()
            .and_then(|text| parse(&id, &path, &text));
        found.insert(id, parsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(exec: &str) -> DesktopEntry {
        DesktopEntry {
            id: "app.desktop".into(),
            name: "Ko‘ruvchi".into(),
            exec: exec.into(),
            mime_types: vec![],
            no_display: false,
            path: "/usr/share/applications/app.desktop".into(),
        }
    }

    fn argv(exec: &str, file: &str) -> Option<Vec<String>> {
        exec_argv(&entry(exec), Path::new(file)).map(|argv| {
            argv.into_iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect()
        })
    }

    #[test]
    fn field_codes_become_single_arguments_without_a_shell() {
        let file = "/home/u/Hujjat; rm -rf ~ $(id).pdf";
        assert_eq!(
            argv("evince %U", file),
            Some(vec!["evince".into(), file.into()])
        );
        assert_eq!(
            argv(r#""/opt/My App/bin" --name=%c -x %i %f %F"#, file),
            Some(vec![
                "/opt/My App/bin".into(),
                "--name=Ko‘ruvchi".into(),
                "-x".into(),
                file.into()
            ])
        );
        assert_eq!(
            argv("gimp-2.10", file),
            Some(vec!["gimp-2.10".into(), file.into()]),
            "field kod bo‘lmasa yo‘l oxiriga"
        );
        assert_eq!(
            argv(r#"sh -c "echo \"hi\" 100%%""#, "/f"),
            Some(vec![
                "sh".into(),
                "-c".into(),
                r#"echo "hi" 100%"#.into(),
                "/f".into()
            ])
        );
        assert_eq!(argv(r#"broken "unterminated"#, "/f"), None);
        assert_eq!(argv("   ", "/f"), None);
    }

    #[test]
    fn parse_filters_non_applications_and_reads_mime_types() {
        let path = Path::new("/x/a.desktop");
        let text = "# comment\n[Desktop Entry]\nType=Application\nName=Rasm\\sko‘ruvchi\nName[uz]=Boshqa\nExec=eog %U\nMimeType=image/png;Image/JPEG;\n\n[Desktop Action new]\nExec=other\n";
        let entry = parse("a.desktop", path, text).expect("entry");
        assert_eq!(entry.name, "Rasm ko‘ruvchi");
        assert_eq!(entry.exec, "eog %U");
        assert_eq!(entry.mime_types, ["image/png", "image/jpeg"]);
        let with =
            |extra: &str| format!("[Desktop Entry]\nType=Application\nName=A\nExec=a\n{extra}");
        assert!(parse("a", path, &with("Terminal=true")).is_none());
        assert!(parse("a", path, &with("Hidden=true")).is_none());
        assert!(parse("a", path, &with("NoDisplay=true")).is_some_and(|e| e.no_display));
        assert!(parse("a", path, "[Desktop Entry]\nType=Link\nName=A\nExec=a").is_none());
    }

    #[test]
    fn mime_wildcards() {
        assert!(mime_matches("image/*", "image/png"));
        assert!(mime_matches("text/plain", "text/plain"));
        assert!(!mime_matches("image/*", "text/plain"));
        assert!(!mime_matches("text/plain", "text/markdown"));
    }

    #[test]
    fn scan_respects_xdg_precedence_and_subdirectory_ids() {
        let root = tempfile::tempdir().expect("temp");
        let user = root.path().join("user/applications");
        let system = root.path().join("system/applications");
        fs::create_dir_all(user.join("kde")).expect("mkdir");
        fs::create_dir_all(&system).expect("mkdir");
        let app = |name: &str, extra: &str| {
            format!("[Desktop Entry]\nType=Application\nName={name}\nExec=x %f\n{extra}")
        };
        fs::write(user.join("viewer.desktop"), app("User viewer", "")).expect("w");
        fs::write(system.join("viewer.desktop"), app("System viewer", "")).expect("w");
        fs::write(user.join("hidden.desktop"), app("H", "Hidden=true")).expect("w");
        fs::write(system.join("hidden.desktop"), app("Should stay hidden", "")).expect("w");
        fs::write(user.join("kde/okular.desktop"), app("Okular", "")).expect("w");
        fs::write(system.join("notes.txt"), "not desktop").expect("w");

        let entries = scan(&[user, system]);
        let ids: Vec<_> = entries
            .iter()
            .map(|e| (e.id.as_str(), e.name.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                ("kde-okular.desktop", "Okular"),
                ("viewer.desktop", "User viewer")
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn xdg_dirs_defaults_and_overrides() {
        let dirs = application_dirs(None, None, Some("/home/u".into()));
        assert_eq!(
            dirs,
            [
                PathBuf::from("/home/u/.local/share/applications"),
                PathBuf::from("/usr/local/share/applications"),
                PathBuf::from("/usr/share/applications"),
            ]
        );
        let dirs = application_dirs(Some("/d".into()), Some("/a:/b".into()), None);
        assert_eq!(dirs[0], PathBuf::from("/d/applications"));
        assert_eq!(dirs.len(), 3);
    }
}
