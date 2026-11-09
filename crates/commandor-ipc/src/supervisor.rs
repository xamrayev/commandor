//! Worker jarayonini boshqaruvchi supervisor: spawn, handshake, timeout, crash va restart.
//!
//! Worker crash yoki timeout bo‘lsa supervisor xato qaytaradi, asosiy jarayon ishlashda davom
//! etadi (NFR-06). Keyingi so‘rov workerni qayta ishga tushiradi (`max_restarts` doirasida).

use crate::frame::{FrameError, read_frame, write_frame};
use crate::protocol::{PROTOCOL_VERSION, WorkerKind, WorkerMessage, WorkerRequest, WorkerResponse};
use crossbeam_channel::{Receiver, RecvTimeoutError, bounded};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Odatda `std::env::current_exe()` — bitta binary, `--worker <kind>` argumenti bilan.
    pub program: PathBuf,
    pub kind: WorkerKind,
    pub envs: Vec<(OsString, OsString)>,
    pub request_timeout: Duration,
    pub handshake_timeout: Duration,
    pub max_restarts: u32,
}

impl WorkerConfig {
    pub fn new(program: PathBuf, kind: WorkerKind) -> Self {
        Self {
            program,
            kind,
            envs: Vec::new(),
            request_timeout: Duration::from_secs(30),
            handshake_timeout: Duration::from_secs(10),
            max_restarts: 3,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("workerni ishga tushirib bo‘lmadi: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("handshake muvaffaqiyatsiz: {0}")]
    Handshake(String),
    #[error("worker so‘rov vaqtida tugadi (crash)")]
    Crashed,
    #[error("worker javobi vaqt chegarasidan oshdi; jarayon to‘xtatildi")]
    Timeout,
    #[error("qayta ishga tushirish chegarasi tugadi")]
    RestartLimit,
    #[error("aloqa xatosi: {0}")]
    Frame(#[from] FrameError),
}

struct Running {
    child: Child,
    stdin: ChildStdin,
    messages: Receiver<Result<WorkerMessage, FrameError>>,
}

impl Running {
    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Supervisor {
    config: WorkerConfig,
    running: Option<Running>,
    spawns: u32,
    next_id: u64,
}

impl Supervisor {
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            running: None,
            spawns: 0,
            next_id: 1,
        }
    }

    /// Hozirgacha necha marta worker ishga tushirilgan.
    pub fn spawn_count(&self) -> u32 {
        self.spawns
    }

    pub fn request(&mut self, request: WorkerRequest) -> Result<WorkerResponse, SupervisorError> {
        if self.running.is_none() {
            self.spawn()?;
        }
        let id = self.next_id;
        self.next_id += 1;
        let timeout = self.config.request_timeout;
        let Some(running) = self.running.as_mut() else {
            return Err(SupervisorError::Crashed);
        };
        let outcome = Self::exchange(running, id, request, timeout);
        if outcome.is_err() {
            // Crash/timeout/aloqa xatosidan keyin jarayon holatiga ishonib bo‘lmaydi.
            self.reset();
        }
        outcome
    }

    fn exchange(
        running: &mut Running,
        id: u64,
        request: WorkerRequest,
        timeout: Duration,
    ) -> Result<WorkerResponse, SupervisorError> {
        write_frame(&mut running.stdin, &WorkerMessage::Request { id, request }).map_err(|e| {
            match e {
                FrameError::Io(_) | FrameError::Closed => SupervisorError::Crashed,
                other => other.into(),
            }
        })?;
        Self::wait_response(&running.messages, id, timeout)
    }

    fn wait_response(
        messages: &Receiver<Result<WorkerMessage, FrameError>>,
        id: u64,
        timeout: Duration,
    ) -> Result<WorkerResponse, SupervisorError> {
        let started = Instant::now();
        loop {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .ok_or(SupervisorError::Timeout)?;
            match messages.recv_timeout(remaining) {
                Ok(Ok(WorkerMessage::Response { id: got, response })) if got == id => {
                    return Ok(response);
                }
                Ok(Ok(_)) => continue, // Eski/kutilmagan xabar: e’tiborsiz.
                Ok(Err(_)) | Err(RecvTimeoutError::Disconnected) => {
                    return Err(SupervisorError::Crashed);
                }
                Err(RecvTimeoutError::Timeout) => return Err(SupervisorError::Timeout),
            }
        }
    }

    fn reset(&mut self) {
        if let Some(running) = self.running.take() {
            running.kill();
        }
    }

    fn spawn(&mut self) -> Result<(), SupervisorError> {
        if self.spawns > self.config.max_restarts {
            return Err(SupervisorError::RestartLimit);
        }
        self.spawns += 1;
        let mut child = Command::new(&self.config.program)
            .arg("--worker")
            .arg(self.config.kind.as_str())
            .envs(self.config.envs.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(SupervisorError::Spawn)?;

        let (Some(stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            return Err(SupervisorError::Handshake("stdio olinmadi".into()));
        };
        let (tx, rx) = bounded(64);
        thread::Builder::new()
            .name("commandor-worker-reader".into())
            .spawn(move || {
                loop {
                    let msg = read_frame::<_, WorkerMessage>(&mut stdout);
                    let stop = msg.is_err();
                    if tx.send(msg).is_err() || stop {
                        return;
                    }
                }
            })
            .map_err(SupervisorError::Spawn)?;

        let running = Running {
            child,
            stdin,
            messages: rx,
        };
        match running.messages.recv_timeout(self.config.handshake_timeout) {
            Ok(Ok(WorkerMessage::Hello {
                protocol_version,
                kind,
            })) if protocol_version == PROTOCOL_VERSION && kind == self.config.kind => {
                self.running = Some(running);
                Ok(())
            }
            other => {
                running.kill();
                Err(SupervisorError::Handshake(format!(
                    "kutilmagan javob: {other:?}"
                )))
            }
        }
    }
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_messages_cannot_extend_response_deadline() {
        let (tx, rx) = bounded(64);
        let producer = thread::spawn(move || {
            for _ in 0..20 {
                let _ = tx.send(Ok(WorkerMessage::Response {
                    id: 99,
                    response: WorkerResponse::Pong,
                }));
                thread::sleep(Duration::from_millis(10));
            }
            let _ = tx.send(Ok(WorkerMessage::Response {
                id: 1,
                response: WorkerResponse::Pong,
            }));
        });
        assert!(matches!(
            Supervisor::wait_response(&rx, 1, Duration::from_millis(30)),
            Err(SupervisorError::Timeout)
        ));
        producer.join().expect("producer");
    }
}
