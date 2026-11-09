//! Broker: UI yuboradigan typed `Command`larni qabul qiladi, tekshiradi va servislarga yo‘naltiradi.
//!
//! UI faqat [`AppHandle`] va [`EventReceiver`]ni ko‘radi. Broker o‘z threadida ishlaydi; har
//! hodisadan keyin [`Notifier`] chaqiriladi (GUIda bu `request_repaint`), shuning uchun broker
//! GUI kutubxonasiga bog‘lanmaydi.

use commandor_core::{
    API_VERSION, AppError, AppInfo, Command, Envelope, ErrorCode, Event, Reply, RequestId, Response,
};
use commandor_fs::FileService;
use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded, unbounded};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    start_at(notifier, commandor_fs::home_directory())
}

/// Test/smoke uchun ham ishlatiladi: boshlang‘ich papka kompozitsiya qatlamidan beriladi.
pub fn start_at(
    notifier: impl Notifier,
    home: PathBuf,
) -> std::io::Result<(AppHandle, EventReceiver, BrokerThread)> {
    let (cmd_tx, cmd_rx) = unbounded::<Envelope<Command>>();
    let (evt_tx, evt_rx) = unbounded::<Event>();
    let notifier: Arc<dyn Notifier> = Arc::new(notifier);
    let (file_tx, file_rx) = bounded::<Envelope<Command>>(32);
    let file_events = evt_tx.clone();
    let file_notifier = Arc::clone(&notifier);
    let file_join = thread::Builder::new()
        .name("commandor-files".into())
        .spawn(move || {
            let mut files = FileService::new(home);
            while let Ok(envelope) = file_rx.recv() {
                let reply = match envelope.body {
                    Command::ReadDirectory { target, options } => {
                        files.read(target, options).map(Response::Directory)
                    }
                    Command::ListVolumes => Ok(Response::Volumes(commandor_fs::list_volumes())),
                    _ => Err(ErrorCode::InvalidArgument.into()),
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
    };
    let join = thread::Builder::new()
        .name("commandor-broker".into())
        .spawn(move || {
            broker.run(&cmd_rx, &file_tx);
            drop(file_tx);
            let _ = file_join.join();
        })?;
    let handle = AppHandle {
        commands: cmd_tx,
        next_request: Arc::new(AtomicU64::new(1)),
    };
    Ok((handle, EventReceiver(evt_rx), BrokerThread(join)))
}

struct Broker {
    events: Sender<Event>,
    notifier: Arc<dyn Notifier>,
}

impl Broker {
    fn run(self, commands: &Receiver<Envelope<Command>>, files: &Sender<Envelope<Command>>) {
        while let Ok(envelope) = commands.recv() {
            let request = envelope.request;
            let reply = if envelope.api_version == API_VERSION
                && matches!(
                    &envelope.body,
                    Command::ReadDirectory { .. } | Command::ListVolumes
                ) {
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
