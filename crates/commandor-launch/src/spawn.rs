//! Tashqi dasturni ishga tushirish: faqat argument massivi, shell yo‘q, stdio ulanmaydi.

use commandor_core::{AppError, ErrorCode};
use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Launcher (`open`, `xdg-open`) natijasini kutish muddati; undan uzoq ishlasa fonda yig‘iladi.
const EXIT_WAIT: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(40);

/// Qanday natija kutiladi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wait {
    /// Exit kodini qisqa kutish: noldan farqli kod — mos dastur yo‘q.
    ExitCode,
    /// Faqat spawn muvaffaqiyati (dasturning o‘zi yoki exit kodi ishonchsiz launcher).
    SpawnOnly,
}

fn spawn_error(error: &std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::EngineNotInstalled,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::Internal,
    }
    .into()
}

/// Jarayon tugashini fonda kutadi (Unix’da zombie qolmasin).
fn reap(mut child: Child) {
    let _ = thread::Builder::new()
        .name("commandor-reaper".into())
        .spawn(move || {
            let _ = child.wait();
        });
}

fn wait_with_deadline(child: &mut Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL),
            _ => return None,
        }
    }
}

pub(crate) fn launch(program: &Path, args: &[OsString], wait: Wait) -> Result<(), AppError> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    launch_command(command, wait)
}

pub(crate) fn launch_command(mut command: Command, wait: Wait) -> Result<(), AppError> {
    let mut child = command.spawn().map_err(|error| spawn_error(&error))?;
    if wait == Wait::SpawnOnly {
        reap(child);
        return Ok(());
    }
    match wait_with_deadline(&mut child, EXIT_WAIT) {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err(ErrorCode::NoApplication.into()),
        None => {
            reap(child);
            Ok(())
        }
    }
}

/// Qisqa so‘rov buyrug‘i (`xdg-mime query ...`) stdout’i; xato yoki timeoutda `None`.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))] // faqat Linux/BSD `xdg-mime`
pub(crate) fn capture(program: &Path, args: &[&std::ffi::OsStr]) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let Some(status) = wait_with_deadline(&mut child, EXIT_WAIT) else {
        let _ = child.kill();
        reap(child);
        return None;
    };
    if !status.success() {
        return None;
    }
    let mut out = String::new();
    child
        .stdout
        .take()?
        .take(64 * 1024)
        .read_to_string(&mut out)
        .ok()?;
    let out = out.trim().to_owned();
    (!out.is_empty()).then_some(out)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_map_to_outcomes_and_missing_program_is_reported() {
        let ok = launch(
            Path::new("/bin/sh"),
            &["-c".into(), "exit 0".into()],
            Wait::ExitCode,
        );
        assert_eq!(ok, Ok(()));
        let fail = launch(
            Path::new("/bin/sh"),
            &["-c".into(), "exit 1".into()],
            Wait::ExitCode,
        );
        assert_eq!(fail.expect_err("fail").code, ErrorCode::NoApplication);
        let missing = launch(
            Path::new("/nonexistent/commandor-launcher"),
            &[],
            Wait::SpawnOnly,
        );
        assert_eq!(
            missing.expect_err("missing").code,
            ErrorCode::EngineNotInstalled
        );
    }

    #[test]
    fn arguments_are_not_interpreted_by_a_shell() {
        // `$(...)` va `;` oddiy matn bo‘lib qoladi: printf ularni o‘zgarishsiz chiqaradi.
        let text = "a; echo pwned $(id)";
        let out = capture(
            Path::new("/usr/bin/printf"),
            &["%s".as_ref(), text.as_ref()],
        );
        assert_eq!(out.as_deref(), Some(text));
    }

    #[test]
    fn capture_times_out_and_kills_slow_commands() {
        let started = Instant::now();
        let out = capture(Path::new("/bin/sleep"), &["30".as_ref()]);
        assert_eq!(out, None);
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}
