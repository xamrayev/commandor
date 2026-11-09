//! Bajariladigan ko‘rinadigan fayllar: standart dastur bilan ochish ularni ishga tushirishi mumkin.

use std::path::Path;

/// Standart ochish kod bajarishi mumkin bo‘lgan kengaytmalar (kichik harfda).
const RISKY_EXTENSIONS: &[&str] = &[
    // Windows
    "exe",
    "com",
    "bat",
    "cmd",
    "msi",
    "msp",
    "scr",
    "pif",
    "cpl",
    "ps1",
    "psm1",
    "vbs",
    "vbe",
    "js",
    "jse",
    "wsf",
    "wsh",
    "hta",
    "lnk",
    "reg",
    "url",
    "appref-ms",
    "jar",
    // macOS
    "app",
    "command",
    "tool",
    "pkg",
    "mpkg",
    "workflow",
    "scpt",
    "applescript",
    "terminal",
    // Unix skriptlari va Linux
    "sh",
    "bash",
    "zsh",
    "csh",
    "ksh",
    "fish",
    "pl",
    "py",
    "rb",
    "run",
    "bin",
    "appimage",
    "desktop",
];

/// `executable_bit` — Unix’da kamida bitta `x` biti (Windows’da doim `false`).
///
/// Kengaytmasiz, lekin `x` bitli fayl ham xavfli: macOS uni Terminalda ishga tushiradi.
/// Kengaytmali `x` bitli fayl (masalan, exFAT diskidagi hujjat) faqat kengaytma bo‘yicha baholanadi.
pub fn looks_executable(path: &Path, executable_bit: bool) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => RISKY_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()),
        None => executable_bit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_are_safe_and_programs_are_not() {
        for safe in [
            "hisobot.pdf",
            "rasm.PNG",
            "jadval.xlsx",
            "README",
            "arxiv.tar.gz",
        ] {
            assert!(!looks_executable(Path::new(safe), false), "{safe}");
        }
        for risky in [
            "setup.EXE",
            "run.sh",
            "x.command",
            "skript.ps1",
            "a.AppImage",
            "b.jar",
        ] {
            assert!(looks_executable(Path::new(risky), false), "{risky}");
        }
    }

    #[test]
    fn executable_bit_matters_only_without_extension() {
        assert!(looks_executable(Path::new("/tmp/tool"), true));
        assert!(!looks_executable(Path::new("/tmp/tool"), false));
        assert!(!looks_executable(
            Path::new("/Volumes/USB/hujjat.docx"),
            true
        ));
    }
}
