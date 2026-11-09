//! `commandor` — GUI; `commandor --worker <kind>` — izolyatsiyalangan worker jarayoni;
//! `commandor --smoke` — oynani ochib, broker tayyor bo‘lgach o‘zi yopiladigan tekshiruv.

use commandor_i18n::{I18n, Lang};
use commandor_ipc::WorkerKind;
use commandor_ui::{CommandorApp, FolderPicker, SmokeProbe};
use eframe::egui;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct NativeFolderPicker;

impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&self) -> Option<String> {
        rfd::FileDialog::new()
            .pick_folder()
            .map(|p| p.to_string_lossy().into_owned())
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["--worker", kind] => run_worker(kind),
        ["--smoke"] => run_gui(true),
        [] => run_gui(false),
        _ => {
            eprintln!("foydalanish: commandor [--smoke | --worker <kind>]");
            ExitCode::from(2)
        }
    }
}

fn run_worker(kind: &str) -> ExitCode {
    let Some(kind) = WorkerKind::parse(kind) else {
        eprintln!("noma’lum worker turi: {kind}");
        return ExitCode::from(2);
    };
    match commandor_ipc::worker::run_stdio(kind) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("worker xatosi: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_gui(smoke: bool) -> ExitCode {
    // Smoke shaxsiy kataloglarni o‘qimaydi; TempDir ilova yopilguncha tirik qoladi.
    let fixture = if smoke {
        let result = tempfile::tempdir().and_then(|dir| {
            std::fs::create_dir(dir.path().join("O‘zbek papka"))?;
            std::fs::write(dir.path().join("hisobot.txt"), "Sintetik smoke fixture")?;
            std::fs::write(dir.path().join(".hidden"), "hidden fixture")?;
            Ok(dir)
        });
        match result {
            Ok(dir) => Some(dir),
            Err(_) => {
                eprintln!("SMOKE FAILED: fixture yaratilmadi");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };
    let i18n = match I18n::new(Lang::Uz) {
        Ok(i18n) => i18n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let title = i18n.t("app-title");
    let passed = Arc::new(AtomicBool::new(false));
    let probe = smoke.then(|| SmokeProbe {
        frames_after_ready: 3,
        passed: Arc::clone(&passed),
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([480.0, 320.0]),
        ..Default::default()
    };

    let mut broker_thread = None;
    let result = eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            let notifier = move || ctx.request_repaint();
            let (handle, events, thread) = if let Some(dir) = &fixture {
                commandor_broker::start_at(notifier, dir.path().to_path_buf())?
            } else {
                commandor_broker::start(notifier)?
            };
            broker_thread = Some(thread);
            let picker: Option<Arc<dyn FolderPicker>> = if smoke {
                None
            } else {
                Some(Arc::new(NativeFolderPicker))
            };
            Ok(Box::new(CommandorApp::new(
                handle, events, i18n, probe, picker,
            )))
        }),
    );
    // Ilova (va undagi AppHandle) tashlangan; broker o‘zi to‘xtaydi.
    if let Some(thread) = broker_thread {
        thread.join();
    }

    match result {
        Err(e) => {
            eprintln!("GUI xatosi: {e}");
            ExitCode::FAILURE
        }
        Ok(()) if smoke && passed.load(Ordering::SeqCst) => {
            eprintln!("SMOKE OK");
            ExitCode::SUCCESS
        }
        Ok(()) if smoke => {
            eprintln!("SMOKE FAILED: broker va ikkala panel tayyor holatiga yetmadi");
            ExitCode::FAILURE
        }
        Ok(()) => ExitCode::SUCCESS,
    }
}
