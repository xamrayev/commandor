//! `cargo xtask <vazifa>` — loyiha yordamchilari.
//!
//! `check-boundaries`: workspace crate’larining *to‘g‘ridan-to‘g‘ri* bog‘liqliklari
//! arxitekturadagi yo‘nalishga mosligini tekshiradi (ARCHITECTURE.uz.md, ADR-005).

use serde_json::Value;
use std::collections::BTreeMap;
use std::process::{Command, ExitCode};

/// Har crate uchun ruxsat etilgan to‘g‘ridan-to‘g‘ri bog‘liqliklar (normal + build).
/// Ro‘yxatda yo‘q workspace crate yangi qo‘shilsa, bu yerga qasddan yozilishi kerak.
const RULES: &[(&str, &[&str])] = &[
    ("commandor-core", &["serde", "thiserror", "uuid"]),
    ("commandor-i18n", &["fluent-bundle", "unic-langid"]),
    (
        "commandor-broker",
        &[
            "commandor-core",
            "commandor-fs",
            "commandor-launch",
            "crossbeam-channel",
        ],
    ),
    // sysinfo: tomlar ro‘yxati (ARCHITECTURE.uz.md, fayl tizimi qatlami).
    ("commandor-fs", &["commandor-core", "sysinfo", "trash"]),
    // OS launcher: standart dastur / Open with; plist — macOS dasturlar katalogi.
    ("commandor-launch", &["commandor-core", "plist"]),
    (
        "commandor-ipc",
        &["crossbeam-channel", "serde", "serde_json", "thiserror"],
    ),
    // UI: servis crate’lari, IO va tarmoq kutubxonalari taqiqlangan.
    (
        "commandor-ui",
        &[
            "commandor-broker",
            "commandor-core",
            "commandor-i18n",
            "eframe",
        ],
    ),
    (
        "commandor-app",
        &[
            "commandor-broker",
            "commandor-i18n",
            "commandor-ipc",
            "commandor-ui",
            "eframe",
            "rfd",
            "tempfile",
        ],
    ),
    ("xtask", &["serde_json"]),
];

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("check-boundaries") => check_boundaries(),
        _ => {
            eprintln!("foydalanish: cargo xtask check-boundaries");
            ExitCode::from(2)
        }
    }
}

fn check_boundaries() -> ExitCode {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = match Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            eprintln!(
                "cargo metadata xatosi: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("cargo ishga tushmadi: {e}");
            return ExitCode::FAILURE;
        }
    };
    let metadata: Value = match serde_json::from_slice(&output) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("metadata JSON xatosi: {e}");
            return ExitCode::FAILURE;
        }
    };

    let rules: BTreeMap<&str, &[&str]> = RULES.iter().copied().collect();
    let mut problems = Vec::new();
    let packages = metadata["packages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    for package in packages {
        let name = package["name"].as_str().unwrap_or_default();
        let Some(allowed) = rules.get(name) else {
            problems.push(format!("{name}: xtask RULES ro‘yxatida qoida yo‘q"));
            continue;
        };
        let deps = package["dependencies"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        for dep in deps {
            if dep["kind"].as_str() == Some("dev") {
                continue; // Test bog‘liqliklari binaryga kirmaydi.
            }
            let dep_name = dep["name"].as_str().unwrap_or_default();
            if !allowed.contains(&dep_name) {
                problems.push(format!("{name} → {dep_name}: ruxsat etilmagan bog‘liqlik"));
            }
        }
    }

    if problems.is_empty() {
        eprintln!("modul chegaralari: OK ({} crate)", packages.len());
        ExitCode::SUCCESS
    } else {
        for p in &problems {
            eprintln!("XATO: {p}");
        }
        ExitCode::FAILURE
    }
}
