//! macOS: `.app` paketlari va UTI e’lonlaridan "Open with" ro‘yxati.
//!
//! LaunchServices FFI ishlatilmaydi (`unsafe` taqiqlangan): `Info.plist`dagi
//! `CFBundleDocumentTypes` va tizim/dasturlar e’lon qilgan `UT*TypeDeclarations` o‘qiladi.
//! Tahlil OSga bog‘liq emas, shuning uchun sintetik fixturelar bilan hamma joyda testlanadi.

use plist::{Dictionary, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Har qanday faylga mos keladigan umumiy turlar: ular "tavsiya" hisoblanmaydi.
const GENERIC_TYPES: &[&str] = &["public.data", "public.item", "public.content", "*"];
const MAX_APPS: usize = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MacApp {
    pub path: PathBuf,
    pub name: String,
    /// E’lon qilingan kengaytmalar va UTIlar (kichik harfda).
    pub extensions: BTreeSet<String>,
    pub types: BTreeSet<String>,
}

/// Bitta fayl uchun dastur mosligi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Fit {
    /// Aniq tur yoki kengaytma e’lon qilingan.
    Specific,
    /// Faqat umumiy tur (`public.data`) orqali.
    Generic,
    None,
}

#[derive(Debug, Default)]
pub(crate) struct Catalog {
    pub apps: Vec<MacApp>,
    /// Kengaytma → UTIlar.
    ext_types: BTreeMap<String, BTreeSet<String>>,
    /// UTI → u mos keladigan (conforms to) UTIlar.
    conforms: BTreeMap<String, BTreeSet<String>>,
}

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.to_lowercase()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_string)
            .map(str::to_lowercase)
            .collect(),
        _ => Vec::new(),
    }
}

fn read_dict(path: &Path) -> Option<Dictionary> {
    Value::from_file(path).ok()?.into_dictionary()
}

impl Catalog {
    /// Standart joylar: tizim turlari va dastur papkalari.
    pub(crate) fn system() -> Self {
        let core = PathBuf::from("/System/Library/CoreServices/CoreTypes.bundle/Contents");
        let mut type_files = vec![core.join("Info.plist")];
        if let Ok(entries) = fs::read_dir(core.join("Library")) {
            type_files.extend(
                entries
                    .flatten()
                    .map(|entry| entry.path().join("Contents/Info.plist")),
            );
        }
        let mut dirs = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
        ];
        if let Some(home) = std::env::home_dir() {
            dirs.push(home.join("Applications"));
        }
        Self::scan(&dirs, &type_files)
    }

    /// `app_dirs` ichidagi `.app`lar (bir daraja ichki papkalar bilan) va `type_files`dagi UTI e’lonlari.
    pub(crate) fn scan(app_dirs: &[PathBuf], type_files: &[PathBuf]) -> Self {
        let mut catalog = Self::default();
        for file in type_files {
            if let Some(dict) = read_dict(file) {
                catalog.add_type_declarations(&dict);
            }
        }
        let mut seen = BTreeSet::new();
        for dir in app_dirs {
            catalog.scan_dir(dir, 0, &mut seen);
        }
        catalog
            .apps
            .sort_by_key(|app| (app.name.to_lowercase(), app.path.clone()));
        catalog
    }

    fn scan_dir(&mut self, dir: &Path, depth: u8, seen: &mut BTreeSet<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if self.apps.len() >= MAX_APPS {
                return;
            }
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if !kind.is_dir() {
                continue; // symlinklar ham kuzatilmaydi
            }
            let is_app = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("app"));
            if is_app {
                if seen.insert(path.clone())
                    && let Some(app) = self.read_app(&path)
                {
                    self.apps.push(app);
                }
            } else if depth < 1 {
                self.scan_dir(&path, depth + 1, seen);
            }
        }
    }

    fn read_app(&mut self, path: &Path) -> Option<MacApp> {
        let info = read_dict(&path.join("Contents/Info.plist"))?;
        self.add_type_declarations(&info);
        let name = path.file_stem()?.to_string_lossy().into_owned();
        let mut extensions = BTreeSet::new();
        let mut types = BTreeSet::new();
        for doc in info
            .get("CFBundleDocumentTypes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_dictionary)
        {
            if doc.get("CFBundleTypeRole").and_then(Value::as_string) == Some("None") {
                continue;
            }
            extensions.extend(strings(doc.get("CFBundleTypeExtensions")));
            types.extend(strings(doc.get("LSItemContentTypes")));
        }
        Some(MacApp {
            path: path.to_path_buf(),
            name,
            extensions,
            types,
        })
    }

    fn add_type_declarations(&mut self, dict: &Dictionary) {
        for key in ["UTExportedTypeDeclarations", "UTImportedTypeDeclarations"] {
            for decl in dict
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_dictionary)
            {
                let Some(id) = decl
                    .get("UTTypeIdentifier")
                    .and_then(Value::as_string)
                    .map(str::to_lowercase)
                else {
                    continue;
                };
                let parents = strings(decl.get("UTTypeConformsTo"));
                self.conforms.entry(id.clone()).or_default().extend(parents);
                let tags = decl
                    .get("UTTypeTagSpecification")
                    .and_then(Value::as_dictionary);
                for ext in strings(tags.and_then(|tags| tags.get("public.filename-extension"))) {
                    self.ext_types.entry(ext).or_default().insert(id.clone());
                }
            }
        }
    }

    /// Kengaytma turlari va ularning barcha ota turlari (conformance yopilishi).
    pub(crate) fn types_for_extension(&self, ext: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut stack: Vec<String> = self
            .ext_types
            .get(&ext.to_lowercase())
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        while let Some(uti) = stack.pop() {
            if out.insert(uti.clone()) {
                stack.extend(self.conforms.get(&uti).into_iter().flatten().cloned());
            }
        }
        out
    }

    pub(crate) fn fit(app: &MacApp, ext: Option<&str>, file_types: &BTreeSet<String>) -> Fit {
        let ext = ext.map(str::to_lowercase);
        let specific_ext = ext.as_ref().is_some_and(|ext| app.extensions.contains(ext));
        let specific_type = app
            .types
            .iter()
            .any(|uti| file_types.contains(uti) && !GENERIC_TYPES.contains(&uti.as_str()));
        if specific_ext || specific_type {
            return Fit::Specific;
        }
        let generic = app.extensions.contains("*")
            || app.types.iter().any(|uti| {
                GENERIC_TYPES.contains(&uti.as_str())
                    && (uti == "public.item" || uti == "public.data" || file_types.contains(uti))
            });
        if generic { Fit::Generic } else { Fit::None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plist_types(decls: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict><key>UTExportedTypeDeclarations</key><array>{decls}</array></dict></plist>"#
        )
    }

    fn decl(id: &str, conforms: &str, ext: &str) -> String {
        format!(
            r#"<dict><key>UTTypeIdentifier</key><string>{id}</string>
<key>UTTypeConformsTo</key><string>{conforms}</string>
<key>UTTypeTagSpecification</key><dict><key>public.filename-extension</key><array><string>{ext}</string></array></dict></dict>"#
        )
    }

    fn write_app(dir: &Path, name: &str, doc_types: &str) {
        let contents = dir.join(format!("{name}.app/Contents"));
        fs::create_dir_all(&contents).expect("mkdir");
        fs::write(
            contents.join("Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict><key>CFBundleName</key><string>{name}</string>
<key>CFBundleDocumentTypes</key><array>{doc_types}</array></dict></plist>"#
            ),
        )
        .expect("write");
    }

    fn doc(role: &str, key: &str, value: &str) -> String {
        format!(
            "<dict><key>CFBundleTypeRole</key><string>{role}</string><key>{key}</key><array><string>{value}</string></array></dict>"
        )
    }

    fn fixture() -> (tempfile::TempDir, Catalog) {
        let dir = tempfile::tempdir().expect("temp");
        let types = dir.path().join("types.plist");
        let decls = [
            decl("public.data", "public.item", "zzz-unused"),
            decl("public.image", "public.data", "zzz-unused2"),
            decl("public.png", "public.image", "png"),
            decl("com.adobe.pdf", "public.data", "pdf"),
        ]
        .concat();
        fs::write(&types, plist_types(&decls)).expect("write");
        let apps = dir.path().join("Applications");
        write_app(
            &apps,
            "Preview",
            &doc("Viewer", "LSItemContentTypes", "public.image"),
        );
        write_app(
            &apps,
            "Reader",
            &doc("Editor", "LSItemContentTypes", "com.adobe.pdf"),
        );
        write_app(
            &apps,
            "HexFiend",
            &doc("Editor", "LSItemContentTypes", "public.data"),
        );
        write_app(
            &apps,
            "Legacy",
            &doc("Viewer", "CFBundleTypeExtensions", "PNG"),
        );
        write_app(
            &apps,
            "Ghost",
            &doc("None", "LSItemContentTypes", "public.png"),
        );
        // Bir daraja ichki papka ham ko‘riladi; buzilgan plist e’tiborsiz qoladi.
        write_app(
            &apps.join("Suite"),
            "Inner",
            &doc("Viewer", "LSItemContentTypes", "public.png"),
        );
        let broken = apps.join("Broken.app/Contents");
        fs::create_dir_all(&broken).expect("mkdir");
        fs::write(broken.join("Info.plist"), "not a plist").expect("write");
        let catalog = Catalog::scan(&[apps], &[types]);
        (dir, catalog)
    }

    #[test]
    fn scans_apps_sorted_and_skips_broken_bundles() {
        let (_dir, catalog) = fixture();
        let names: Vec<_> = catalog.apps.iter().map(|app| app.name.as_str()).collect();
        assert_eq!(
            names,
            ["Ghost", "HexFiend", "Inner", "Legacy", "Preview", "Reader"]
        );
    }

    #[test]
    fn conformance_closure_and_fit() {
        let (_dir, catalog) = fixture();
        let png = catalog.types_for_extension("PNG");
        assert!(png.contains("public.png") && png.contains("public.image"));
        assert!(png.contains("public.data") && png.contains("public.item"));
        let fit = |name: &str, ext: &str| {
            let app = catalog
                .apps
                .iter()
                .find(|app| app.name == name)
                .expect("app");
            Catalog::fit(app, Some(ext), &catalog.types_for_extension(ext))
        };
        assert_eq!(fit("Preview", "png"), Fit::Specific, "ota tur orqali");
        assert_eq!(fit("Legacy", "png"), Fit::Specific, "kengaytma orqali");
        assert_eq!(fit("Inner", "png"), Fit::Specific);
        assert_eq!(fit("Reader", "png"), Fit::None);
        assert_eq!(fit("Reader", "pdf"), Fit::Specific);
        assert_eq!(fit("HexFiend", "png"), Fit::Generic);
        assert_eq!(
            fit("Ghost", "png"),
            Fit::None,
            "Role=None hisobga olinmaydi"
        );
        assert_eq!(fit("Preview", "noma’lum"), Fit::None);
    }
}
