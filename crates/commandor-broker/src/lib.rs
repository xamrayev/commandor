//! Broker: UI yuboradigan typed `Command`larni qabul qiladi, tekshiradi va servislarga yo‘naltiradi.
//!
//! UI faqat [`AppHandle`] va [`EventReceiver`]ni ko‘radi. Broker o‘z threadida ishlaydi; har
//! hodisadan keyin [`Notifier`] chaqiriladi (GUIda bu `request_repaint`), shuning uchun broker
//! GUI kutubxonasiga bog‘lanmaydi.

use commandor_core::{
    API_VERSION, AppError, AppInfo, Command, Envelope, ErrorCode, Event, OpenWith, Reply,
    RequestId, Response,
};
use commandor_core::{JobId, JobProgress, PlanAction};
use commandor_fs::{FileService, TransferJob};
use commandor_launch::Launcher;
use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded, unbounded};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::time::Instant;

/// Hodisa kelganda UIni uyg‘otish.
pub trait Notifier: Send + Sync + 'static {
    fn notify(&self);
}

impl<F: Fn() + Send + Sync + 'static> Notifier for F {
    fn notify(&self) {
        self()
    }
}

/// UI tarafidagi buyruq yuborish tutqichi. Klonlanadi; hech qachon bloklamaydi.
#[derive(Clone, Debug)]
pub struct AppHandle {
    commands: Sender<Envelope<Command>>,
    next_request: Arc<AtomicU64>,
}

impl AppHandle {
    /// Buyruqni navbatga qo‘yadi va darhol `RequestId` qaytaradi. Javob `Event::Reply` bilan keladi.
    /// Broker to‘xtagan bo‘lsa `None`.
    pub fn dispatch(&self, command: Command) -> Option<RequestId> {
        let request = RequestId::from_raw(self.next_request.fetch_add(1, Ordering::Relaxed));
        self.commands
            .send(Envelope {
                request,
                api_version: API_VERSION,
                body: command,
            })
            .ok()?;
        Some(request)
    }
}

/// UI tarafidagi hodisa qabul qiluvchi.
#[derive(Debug)]
pub struct EventReceiver(Receiver<Event>);

impl EventReceiver {
    /// Kadr ichida chaqiriladi: kutmaydi, faqat tayyor hodisalarni beradi.
    pub fn drain(&self) -> Vec<Event> {
        let mut out = Vec::new();
        loop {
            match self.0.try_recv() {
                Ok(event) => out.push(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return out,
            }
        }
    }

    /// Testlar va GUIsiz rejim uchun.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Event> {
        self.0.recv_timeout(timeout).ok()
    }
}

/// Broker threadi egasi. Barcha `AppHandle`lar tashlanganda broker to‘xtaydi; `join` kutadi.
#[derive(Debug)]
pub struct BrokerThread(JoinHandle<()>);

impl BrokerThread {
    pub fn join(self) {
        // Broker ichidagi panic GUIni yiqitmasligi kerak; natija bu yerda e’tiborsiz.
        let _ = self.0.join();
    }
}

/// Brokerni alohida threadda ishga tushiradi.
pub fn start(notifier: impl Notifier) -> std::io::Result<(AppHandle, EventReceiver, BrokerThread)> {
    start_with(
        notifier,
        commandor_fs::home_directory(),
        commandor_settings::default_path(),
    )
}

/// Test/smoke uchun: boshlang‘ich papka beriladi, sozlamalar saqlanmaydi (foydalanuvchi fayliga tegilmaydi).
pub fn start_at(
    notifier: impl Notifier,
    home: PathBuf,
) -> std::io::Result<(AppHandle, EventReceiver, BrokerThread)> {
    start_with(notifier, home, None)
}

/// `settings: None` — sozlamalar faqat xotirada (o‘qish sukut, yozish no-op).
pub fn start_with(
    notifier: impl Notifier,
    home: PathBuf,
    settings: Option<PathBuf>,
) -> std::io::Result<(AppHandle, EventReceiver, BrokerThread)> {
    let (cmd_tx, cmd_rx) = unbounded::<Envelope<Command>>();
    let (evt_tx, evt_rx) = unbounded::<Event>();
    let notifier: Arc<dyn Notifier> = Arc::new(notifier);
    let (file_tx, file_rx) = bounded::<Envelope<Command>>(32);
    let file_events = evt_tx.clone();
    let file_notifier = Arc::clone(&notifier);
    // Launcher threadi: tashqi dasturni ishga tushiradi va dasturlar katalogini o‘qiydi.
    // Yo‘l faqat file threaddagi grantdan keladi; listing bu ish tufayli kutib qolmaydi.
    let (launch_tx, launch_rx) = bounded::<(RequestId, LaunchJob)>(16);
    let launch_events = evt_tx.clone();
    let launch_notifier = Arc::clone(&notifier);
    let launch_join = thread::Builder::new()
        .name("commandor-launch".into())
        .spawn(move || {
            let mut launcher = Launcher::new();
            while let Ok((request, job)) = launch_rx.recv() {
                let reply = match job {
                    LaunchJob::Open {
                        path,
                        with,
                        allow_executable,
                    } => launcher
                        .open(&path, with, allow_executable)
                        .map(Response::Opened),
                    LaunchJob::List { path } => {
                        Ok(Response::OpenWithApps(launcher.list_open_with(&path)))
                    }
                };
                if launch_events.send(Event::Reply { request, reply }).is_err() {
                    break;
                }
                launch_notifier.notify();
            }
        })?;
    // Jobs threadi: nusxalash/ko‘chirish navbati. Bekor qilish belgilari umumiy jadvalda.
    let jobs: JobTable = Arc::new(Mutex::new(HashMap::new()));
    let (jobs_tx, jobs_rx) = bounded::<(JobId, TransferJob, Arc<AtomicBool>)>(64);
    let jobs_events = evt_tx.clone();
    let jobs_notifier = Arc::clone(&notifier);
    let jobs_table = Arc::clone(&jobs);
    let jobs_join = thread::Builder::new()
        .name("commandor-jobs".into())
        .spawn(move || {
            while let Ok((job, transfer, cancel)) = jobs_rx.recv() {
                run_job(job, transfer, &cancel, &jobs_events, jobs_notifier.as_ref());
                lock(&jobs_table).remove(&job);
            }
        })?;
    let file_jobs = Arc::clone(&jobs);
    let file_join = thread::Builder::new()
        .name("commandor-files".into())
        .spawn(move || {
            let mut files = FileService::new(home);
            let settings = commandor_settings::SettingsStore::new(settings);
            while let Ok(envelope) = file_rx.recv() {
                let reply = match envelope.body {
                    Command::ReadDirectory { target, options } => {
                        Some(files.read(target, options).map(Response::Directory))
                    }
                    Command::ListVolumes => {
                        Some(Ok(Response::Volumes(commandor_fs::list_volumes())))
                    }
                    Command::OpenEntry {
                        location,
                        entry,
                        with,
                        allow_executable,
                    } => match files.resolve_file(location, entry) {
                        Ok(path) => forward(
                            &launch_tx,
                            envelope.request,
                            LaunchJob::Open {
                                path,
                                with,
                                allow_executable,
                            },
                        ),
                        Err(error) => Some(Err(error)),
                    },
                    Command::ListOpenWith { location, entry } => {
                        match files.resolve_file(location, entry) {
                            Ok(path) => {
                                forward(&launch_tx, envelope.request, LaunchJob::List { path })
                            }
                            Err(error) => Some(Err(error)),
                        }
                    }
                    Command::CreatePlan(request) => {
                        Some(files.create_plan(request).map(Response::Plan))
                    }
                    Command::ExecutePlan { plan, hash } => Some(
                        if matches!(
                            files.plan_action(plan),
                            Some(PlanAction::Copy | PlanAction::Move)
                        ) {
                            files
                                .start_transfer(plan, hash)
                                .and_then(|transfer| enqueue(&jobs_tx, &file_jobs, transfer))
                                .map(Response::JobStarted)
                        } else {
                            files.execute_plan(plan, hash).map(Response::Executed)
                        },
                    ),
                    Command::LoadSettings => {
                        Some(Ok(Response::Settings(Box::new(settings.load()))))
                    }
                    Command::SaveSettings(value) => {
                        Some(settings.save(&value).map(|()| Response::SettingsSaved))
                    }
                    Command::DiscardPlan { plan } => {
                        files.discard_plan(plan);
                        Some(Ok(Response::Discarded))
                    }
                    _ => Some(Err(ErrorCode::InvalidArgument.into())),
                };
                // `None`: javobni launcher threadi yuboradi.
                let Some(reply) = reply else {
                    continue;
                };
                if file_events
                    .send(Event::Reply {
                        request: envelope.request,
                        reply,
                    })
                    .is_err()
                {
                    break;
                }
                file_notifier.notify();
            }
        })?;
    let broker = Broker {
        events: evt_tx,
        notifier,
        jobs: Arc::clone(&jobs),
    };
    let join = thread::Builder::new()
        .name("commandor-broker".into())
        .spawn(move || {
            broker.run(&cmd_rx, &file_tx);
            // UI yopildi: ishlayotgan vazifalar bekor qilinadi (yarim fayl qolmaydi), keyin kutiladi.
            for cancel in lock(&jobs).values() {
                cancel.store(true, Ordering::Relaxed);
            }
            drop(file_tx);
            let _ = file_join.join();
            // File thread `launch_tx`/`jobs_tx`ni tashlagach launcher va jobs ham to‘xtaydi.
            let _ = launch_join.join();
            let _ = jobs_join.join();
        })?;
    let handle = AppHandle {
        commands: cmd_tx,
        next_request: Arc::new(AtomicU64::new(1)),
    };
    Ok((handle, EventReceiver(evt_rx), BrokerThread(join)))
}

/// File threaddan launcher threadga uzatiladigan ish (yo‘l grant orqali aniqlangan).
enum LaunchJob {
    Open {
        path: PathBuf,
        with: OpenWith,
        allow_executable: bool,
    },
    List {
        path: PathBuf,
    },
}

/// Launcher navbatiga qo‘yadi. Navbat to‘lgan yoki to‘xtagan bo‘lsa darhol xato javobi.
fn forward(
    launch: &Sender<(RequestId, LaunchJob)>,
    request: RequestId,
    job: LaunchJob,
) -> Option<Reply> {
    match launch.try_send((request, job)) {
        Ok(()) => None,
        Err(crossbeam_channel::TrySendError::Full(_)) => Some(Err(ErrorCode::RateLimited.into())),
        Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
            Some(Err(ErrorCode::Internal.into()))
        }
    }
}

type JobTable = Arc<Mutex<HashMap<JobId, Arc<AtomicBool>>>>;

/// Panic bilan zaharlangan mutex ham ishlatiladi: jadvalda faqat bekor qilish belgilari bor.
fn lock(table: &JobTable) -> std::sync::MutexGuard<'_, HashMap<JobId, Arc<AtomicBool>>> {
    table
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Vazifani navbatga qo‘yadi; navbat to‘lsa `RateLimited` (reja iste’mol qilingan).
fn enqueue(
    jobs_tx: &Sender<(JobId, TransferJob, Arc<AtomicBool>)>,
    table: &JobTable,
    transfer: TransferJob,
) -> Result<JobId, AppError> {
    let job = JobId::new();
    let cancel = Arc::new(AtomicBool::new(false));
    lock(table).insert(job, Arc::clone(&cancel));
    if jobs_tx.try_send((job, transfer, cancel)).is_err() {
        lock(table).remove(&job);
        return Err(ErrorCode::RateLimited.into());
    }
    Ok(job)
}

/// Progress ko‘pi bilan har 100 ms’da yuboriladi; oxirida `JobFinished`.
fn run_job(
    job: JobId,
    transfer: TransferJob,
    cancel: &AtomicBool,
    events: &Sender<Event>,
    notifier: &dyn Notifier,
) {
    const EVERY: Duration = Duration::from_millis(100);
    let action = transfer.action();
    let totals = transfer.totals();
    let mut last = Instant::now() - EVERY;
    let report = transfer.run(cancel, &mut |progress| {
        if last.elapsed() >= EVERY {
            last = Instant::now();
            let _ = events.send(Event::JobProgress(JobProgress {
                job,
                action,
                done_items: progress.done_items,
                done_bytes: progress.done_bytes,
                totals,
                current: progress.current.clone(),
            }));
            notifier.notify();
        }
    });
    let _ = events.send(Event::JobFinished {
        job,
        result: Ok(report),
    });
    notifier.notify();
}

struct Broker {
    events: Sender<Event>,
    notifier: Arc<dyn Notifier>,
    jobs: JobTable,
}

impl Broker {
    fn run(self, commands: &Receiver<Envelope<Command>>, files: &Sender<Envelope<Command>>) {
        while let Ok(envelope) = commands.recv() {
            let request = envelope.request;
            let reply = if let Command::CancelJob { job } = &envelope.body
                && envelope.api_version == API_VERSION
            {
                match lock(&self.jobs).get(job) {
                    Some(cancel) => {
                        cancel.store(true, Ordering::Relaxed);
                        Ok(Response::CancelRequested)
                    }
                    None => Err(ErrorCode::NotFound.into()),
                }
            } else if envelope.api_version == API_VERSION
                && matches!(
                    &envelope.body,
                    Command::ReadDirectory { .. }
                        | Command::ListVolumes
                        | Command::OpenEntry { .. }
                        | Command::ListOpenWith { .. }
                        | Command::CreatePlan(_)
                        | Command::ExecutePlan { .. }
                        | Command::DiscardPlan { .. }
                        | Command::LoadSettings
                        | Command::SaveSettings(_)
                )
            {
                if files.try_send(envelope).is_ok() {
                    continue;
                }
                Err(ErrorCode::RateLimited.into())
            } else {
                handle(envelope)
            };
            if self.events.send(Event::Reply { request, reply }).is_err() {
                return; // UI yopilgan.
            }
            self.notifier.notify();
        }
    }
}

fn handle(envelope: Envelope<Command>) -> Reply {
    if envelope.api_version != API_VERSION {
        return Err(AppError::new(ErrorCode::UnsupportedApiVersion));
    }
    match envelope.body {
        Command::Ping => Ok(Response::Pong),
        Command::GetAppInfo => Ok(Response::AppInfo(AppInfo {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            api_version: API_VERSION,
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
        })),
        #[allow(unreachable_patterns)]
        // `Command` #[non_exhaustive]; yangi variantlar qo‘shilganda.
        _ => Err(AppError::new(ErrorCode::Internal)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    const WAIT: Duration = Duration::from_secs(5);

    #[test]
    fn dispatch_returns_reply_with_same_request_id_and_notifies() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&count);
        let (handle, events, thread) = start(move || {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .expect("start");

        let first = handle.dispatch(Command::Ping).expect("dispatch");
        let second = handle.dispatch(Command::GetAppInfo).expect("dispatch");
        assert!(second > first, "RequestId monoton o‘sadi");

        let Some(Event::Reply { request, reply }) = events.recv_timeout(WAIT) else {
            panic!("javob kelmadi")
        };
        assert_eq!((request, reply), (first, Ok(Response::Pong)));

        let Some(Event::Reply {
            request,
            reply: Ok(Response::AppInfo(info)),
        }) = events.recv_timeout(WAIT)
        else {
            panic!("AppInfo kelmadi")
        };
        assert_eq!(request, second);
        assert_eq!(info.api_version, API_VERSION);
        assert_eq!(info.os, std::env::consts::OS);

        drop(handle);
        thread.join();
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn list_volumes_is_answered_by_file_thread() {
        let (handle, events, thread) = start(|| {}).expect("start");
        let request = handle.dispatch(Command::ListVolumes).expect("dispatch");
        let Some(Event::Reply {
            request: got,
            reply: Ok(Response::Volumes(volumes)),
        }) = events.recv_timeout(WAIT)
        else {
            panic!("Volumes javobi kelmadi")
        };
        assert_eq!(got, request);
        assert!(!volumes.is_empty(), "kamida tizim tomi bo‘ladi");
        drop(handle);
        thread.join();
    }

    #[test]
    fn wrong_api_version_is_rejected() {
        let reply = handle(Envelope {
            request: RequestId::from_raw(1),
            api_version: API_VERSION + 1,
            body: Command::Ping,
        });
        assert_eq!(
            reply.map_err(|e| e.code),
            Err(ErrorCode::UnsupportedApiVersion)
        );
    }

    #[test]
    fn broker_stops_when_all_handles_dropped() {
        let (handle, events, thread) = start(|| {}).expect("start");
        let clone = handle.clone();
        drop(handle);
        assert!(clone.dispatch(Command::Ping).is_some());
        drop(clone);
        thread.join();
        assert_eq!(events.drain().len(), 1);
    }
}
