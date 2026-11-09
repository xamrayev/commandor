//! Haqiqiy `commandor --worker diag` child jarayoni bilan supervisor sinovlari (NFR-06).

use commandor_ipc::protocol::DIAG_ENV;
use commandor_ipc::{
    Supervisor, SupervisorError, WorkerConfig, WorkerKind, WorkerRequest, WorkerResponse,
};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

fn config(diag: bool) -> WorkerConfig {
    let mut c = WorkerConfig::new(
        PathBuf::from(env!("CARGO_BIN_EXE_commandor")),
        WorkerKind::Diag,
    );
    if diag {
        c.envs.push((DIAG_ENV.into(), "1".into()));
    }
    c.request_timeout = Duration::from_secs(5);
    c
}

#[test]
fn ping_and_unicode_echo_over_real_process() {
    let mut s = Supervisor::new(config(false));
    assert_eq!(
        s.request(WorkerRequest::Ping).expect("ping"),
        WorkerResponse::Pong
    );
    let text = "Hisobot — o‘zbekcha ✓".to_owned();
    assert_eq!(
        s.request(WorkerRequest::Echo(text.clone())).expect("echo"),
        WorkerResponse::Echo(text)
    );
    assert_eq!(s.spawn_count(), 1, "bitta jarayon qayta ishlatiladi");
}

#[test]
fn diagnostic_requests_are_rejected_without_env_flag() {
    let mut s = Supervisor::new(config(false));
    assert!(matches!(
        s.request(WorkerRequest::Abort),
        Ok(WorkerResponse::Rejected(_))
    ));
    assert_eq!(
        s.request(WorkerRequest::Ping).expect("ping"),
        WorkerResponse::Pong
    );
}

#[test]
fn crash_is_reported_and_next_request_restarts_worker() {
    let mut s = Supervisor::new(config(true));
    assert_eq!(
        s.request(WorkerRequest::Ping).expect("ping"),
        WorkerResponse::Pong
    );
    assert!(matches!(
        s.request(WorkerRequest::Abort),
        Err(SupervisorError::Crashed)
    ));
    assert_eq!(
        s.request(WorkerRequest::Ping).expect("ping after restart"),
        WorkerResponse::Pong
    );
    assert_eq!(s.spawn_count(), 2);
}

#[test]
fn hung_worker_times_out_and_is_replaced() {
    let mut c = config(true);
    c.request_timeout = Duration::from_millis(300);
    let mut s = Supervisor::new(c);
    let started = Instant::now();
    assert!(matches!(
        s.request(WorkerRequest::Sleep { millis: 10_000 }),
        Err(SupervisorError::Timeout)
    ));
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "cancel/timeout tez aks etadi (NFR-07)"
    );
    assert_eq!(
        s.request(WorkerRequest::Ping).expect("ping"),
        WorkerResponse::Pong
    );
}

#[test]
fn restart_limit_stops_crash_loop() {
    let mut c = config(true);
    c.max_restarts = 1;
    let mut s = Supervisor::new(c);
    assert!(matches!(
        s.request(WorkerRequest::Abort),
        Err(SupervisorError::Crashed)
    ));
    assert!(matches!(
        s.request(WorkerRequest::Abort),
        Err(SupervisorError::Crashed)
    ));
    assert!(matches!(
        s.request(WorkerRequest::Ping),
        Err(SupervisorError::RestartLimit)
    ));
}

#[test]
fn unknown_worker_kind_and_arguments_exit_with_usage_code() {
    let bin = env!("CARGO_BIN_EXE_commandor");
    let status = Command::new(bin)
        .args(["--worker", "no-such"])
        .status()
        .expect("run");
    assert_eq!(status.code(), Some(2));
    let status = Command::new(bin).arg("--bogus").status().expect("run");
    assert_eq!(status.code(), Some(2));
}
