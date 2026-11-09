//! Sozlamalar va sessiyani tiklash/saqlash (P3-07). GUIsiz test qilinadi.
//!
//! - Ishga tushganda `LoadSettings` so‘raladi; javob kelmaguncha hech narsa saqlanmaydi
//!   (aks holda mavjud fayl sukut qiymatlar bilan bosib yozilardi).
//! - Tiklash `pump` orqali (`apply`da dispatch yo‘q).
//! - `autosave` holat surati oxirgi saqlangandan farq qilsagina yozadi.

use crate::opening::{Notice, NoticeKind};
use crate::state::{AppState, Dispatch, MAX_BOOKMARKS, MAX_TABS, PanelState, PanelTabs, Side};
use commandor_core::{
    AppError, Command, DetailValue, DirectoryTarget, ErrorCode, ListOptions, PanelSession, Reply,
    RequestId, Response, SETTINGS_VERSION, Session, Settings, TabSession, UiSettings,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsState {
    /// Fayl o‘qildi (yoki o‘qib bo‘lmasligi aniq bo‘ldi).
    pub loaded: bool,
    /// O‘qishda IO xatosi bo‘lsa saqlash o‘chiriladi (o‘qilmagan faylni bosib yozmaslik uchun).
    pub saving: bool,
    pub restore_session: bool,
    pending_load: Option<RequestId>,
    pending_save: Option<RequestId>,
    last_saved: Option<Settings>,
    restore: Option<Settings>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            loaded: false,
            saving: true,
            restore_session: true,
            pending_load: None,
            pending_save: None,
            last_saved: None,
            restore: None,
        }
    }
}

fn reason(error: &AppError) -> Option<&str> {
    match error.details.get("reason") {
        Some(DetailValue::Text(text)) => Some(text),
        _ => None,
    }
}

fn valid_path(path: &str) -> bool {
    !path.is_empty() && !path.contains('\0') && std::path::Path::new(path).is_absolute()
}

impl PanelState {
    /// Saqlash uchun yo‘l: listing bo‘lmasa (yuklanmoqda/xato) tarixdagi joriy yo‘l.
    fn session_path(&self) -> Option<String> {
        if let Some(path) = self.current_path() {
            return Some(path.to_owned());
        }
        match self.history.get(self.history_index) {
            Some(DirectoryTarget::Path(path)) => Some(path.clone()),
            _ => None,
        }
    }
}

fn panel_session(tabs: &PanelTabs) -> PanelSession {
    let mut session = PanelSession::default();
    for (index, tab) in tabs.tabs.iter().enumerate() {
        let Some(path) = tab.session_path() else {
            continue;
        };
        if index == tabs.current {
            session.current = session.tabs.len();
        }
        session.tabs.push(TabSession {
            path,
            hidden: tab.options.hidden,
            sort: tab.options.sort,
        });
    }
    session
}

impl AppState {
    pub(crate) fn request_settings(&mut self, dispatch: &impl Dispatch) {
        self.settings.pending_load = dispatch.dispatch(Command::LoadSettings);
        if self.settings.pending_load.is_none() {
            // Broker yo‘q: saqlash ham bo‘lmaydi.
            self.settings.loaded = true;
            self.settings.saving = false;
        }
    }

    /// Hozirgi holatdan sozlama surati (filtr kabi vaqtinchalik qiymatlarsiz).
    pub fn snapshot(&self) -> Settings {
        Settings {
            version: SETTINGS_VERSION,
            ui: UiSettings {
                restore_session: self.settings.restore_session,
            },
            bookmarks: self.bookmarks.clone(),
            session: Session {
                active: self.active.index() as u8,
                left: panel_session(&self.panels[0]),
                right: panel_session(&self.panels[1]),
            },
        }
    }

    /// O‘zgarish bo‘lsa saqlaydi. Yuklanmaguncha yoki saqlash o‘chiq bo‘lsa hech narsa qilmaydi.
    pub fn autosave(&mut self, dispatch: &impl Dispatch) -> bool {
        self.save_if_changed(dispatch, false)
    }

    /// Chiqishda: oldingi saqlash javobini kutmasdan oxirgi holat yuboriladi.
    pub fn flush_settings(&mut self, dispatch: &impl Dispatch) -> bool {
        self.save_if_changed(dispatch, true)
    }

    fn save_if_changed(&mut self, dispatch: &impl Dispatch, force: bool) -> bool {
        if !self.settings.loaded
            || !self.settings.saving
            || (self.settings.pending_save.is_some() && !force)
        {
            return false;
        }
        let snapshot = self.snapshot();
        if self.settings.last_saved.as_ref() == Some(&snapshot) {
            return false;
        }
        self.settings.pending_save =
            dispatch.dispatch(Command::SaveSettings(Box::new(snapshot.clone())));
        self.settings.last_saved = Some(snapshot);
        self.settings.pending_save.is_some()
    }

    /// Yuklangan sozlamani qo‘llaydi: xatcho‘plar va (yoqilgan bo‘lsa) tablar.
    pub(crate) fn pump_settings(&mut self, dispatch: &impl Dispatch) {
        let Some(settings) = self.settings.restore.take() else {
            return;
        };
        self.settings.restore_session = settings.ui.restore_session;
        let mut bookmarks: Vec<String> = Vec::new();
        for path in settings.bookmarks {
            if valid_path(&path) && !bookmarks.contains(&path) && bookmarks.len() < MAX_BOOKMARKS {
                bookmarks.push(path);
            }
        }
        self.bookmarks = bookmarks;
        if !settings.ui.restore_session {
            return;
        }
        for (side, session) in [
            (Side::Left, &settings.session.left),
            (Side::Right, &settings.session.right),
        ] {
            let tabs: Vec<(String, ListOptions)> = session
                .tabs
                .iter()
                .filter(|tab| valid_path(&tab.path))
                .take(MAX_TABS)
                .map(|tab| {
                    let options = ListOptions {
                        hidden: tab.hidden,
                        sort: tab.sort,
                        ..ListOptions::default()
                    };
                    (tab.path.clone(), options)
                })
                .collect();
            if let Some(restored) = PanelTabs::restore(dispatch, tabs, session.current) {
                self.panels[side.index()] = restored;
            }
        }
        self.active = if settings.session.active == 1 {
            Side::Right
        } else {
            Side::Left
        };
    }

    fn settings_notice(&mut self, error: AppError) {
        self.opening.notice = Some(Notice {
            kind: NoticeKind::Settings,
            file: String::new(),
            error,
        });
    }

    /// Sozlamalarga oid javobni qo‘llaydi; boshqasiga tegishli bo‘lsa qaytaradi.
    pub(crate) fn apply_settings(&mut self, request: RequestId, reply: Reply) -> Option<Reply> {
        if self.settings.pending_load == Some(request) {
            self.settings.pending_load = None;
            self.settings.loaded = true;
            match reply {
                Ok(Response::Settings(load)) => {
                    if let Some(problem) = load.problem {
                        // Buzilgan fayl zaxiraga olingan — saqlash xavfsiz; IO xatosida — yo‘q.
                        let parse = matches!(
                            reason(&problem),
                            Some("settings_invalid" | "settings_too_large")
                        );
                        self.settings.saving = parse;
                        self.settings_notice(problem);
                    }
                    self.settings.last_saved = Some(load.settings.clone());
                    self.settings.restore = Some(load.settings);
                }
                Ok(_) => {
                    self.settings.saving = false;
                    self.settings_notice(ErrorCode::Internal.into());
                }
                Err(error) => {
                    self.settings.saving = false;
                    self.settings_notice(error);
                }
            }
            return None;
        }
        if self.settings.pending_save == Some(request) {
            self.settings.pending_save = None;
            if let Err(error) = reply {
                // Har soniyada xato ko‘rsatmaslik uchun saqlash shu sessiyada to‘xtatiladi.
                self.settings.saving = false;
                self.settings_notice(error);
            }
            return None;
        }
        Some(reply)
    }
}

impl PanelTabs {
    /// Saqlangan tablardan panel; bo‘sh ro‘yxat bo‘lsa `None` (joriy panel qoladi).
    pub(crate) fn restore(
        dispatch: &impl Dispatch,
        tabs: Vec<(String, ListOptions)>,
        current: usize,
    ) -> Option<Self> {
        if tabs.is_empty() {
            return None;
        }
        let mut restored = Self::start_empty();
        for (path, options) in tabs {
            restored.push_tab(dispatch, DirectoryTarget::Path(path), options);
        }
        restored.current = current.min(restored.tabs.len() - 1);
        Some(restored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commandor_core::{
        DirectoryListing, Event, Location, LocationId, SettingsLoad, SortCriteria, SortDirection,
        SortField,
    };
    use std::cell::RefCell;

    struct Recorder(RefCell<Vec<Command>>);
    impl Dispatch for Recorder {
        fn dispatch(&self, command: Command) -> Option<RequestId> {
            let mut log = self.0.borrow_mut();
            log.push(command);
            Some(RequestId::from_raw(log.len() as u64))
        }
    }

    fn id_of(rec: &Recorder, pred: impl Fn(&Command) -> bool) -> RequestId {
        let log = rec.0.borrow();
        let index = log.iter().rposition(pred).expect("command");
        RequestId::from_raw(index as u64 + 1)
    }

    fn saved() -> Settings {
        let mut settings = Settings {
            bookmarks: vec!["/b".into(), "/b".into(), "nisbiy".into(), "/c".into()],
            ..Settings::default()
        };
        settings.session.active = 1;
        settings.session.left = PanelSession {
            current: 5,
            tabs: vec![
                TabSession {
                    path: "/ish".into(),
                    ..TabSession::default()
                },
                TabSession {
                    path: String::new(),
                    ..TabSession::default()
                },
                TabSession {
                    path: "/ish/rasm".into(),
                    hidden: true,
                    sort: SortCriteria {
                        field: SortField::Modified,
                        direction: SortDirection::Descending,
                    },
                },
            ],
        };
        settings
    }

    fn load(s: &mut AppState, rec: &Recorder, settings: Settings) {
        let request = id_of(rec, |c| matches!(c, Command::LoadSettings));
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Settings(Box::new(SettingsLoad {
                settings,
                problem: None,
            }))),
        });
    }

    #[test]
    fn nothing_is_saved_before_settings_are_loaded() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        assert!(rec.0.borrow().contains(&Command::LoadSettings));
        s.bookmarks.push("/yangi".into());
        assert!(!s.autosave(&rec), "yuklanmaguncha fayl bosib yozilmaydi");
    }

    #[test]
    fn restore_validates_tabs_bookmarks_and_active_side() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        load(&mut s, &rec, saved());
        s.pump(&rec);
        assert_eq!(
            s.bookmarks,
            ["/b", "/c"],
            "takror va nisbiy yo‘llar tashlanadi"
        );
        assert_eq!(s.active, Side::Right);
        let left = &s.panels[0];
        assert_eq!(left.tabs.len(), 2, "bo‘sh yo‘lli tab tashlanadi");
        assert_eq!(left.current, 1, "indeks chegaralanadi");
        assert!(left.tabs[1].options.hidden);
        assert_eq!(left.tabs[1].options.sort.field, SortField::Modified);
        assert_ne!(left.tabs[0].tab_id, left.tabs[1].tab_id);
        let log = rec.0.borrow();
        assert!(log.iter().any(|c| matches!(c, Command::ReadDirectory {
            target: DirectoryTarget::Path(p), ..
        } if p == "/ish/rasm")));
        drop(log);
        assert_eq!(
            s.panels[1].tabs.len(),
            1,
            "o‘ng panel sessiyasi bo‘sh — tegilmaydi"
        );
    }

    #[test]
    fn autosave_writes_only_changes_and_waits_for_previous_save() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        load(&mut s, &rec, Settings::default());
        s.pump(&rec);
        // Boshlang‘ich holat (uy papkasi hali yuklanmoqda) suratga tushadi — bir marta saqlanadi.
        let first = s.autosave(&rec);
        let saves = |rec: &Recorder| {
            rec.0
                .borrow()
                .iter()
                .filter(|c| matches!(c, Command::SaveSettings(_)))
                .count()
        };
        let before = saves(&rec);
        assert!(
            !s.autosave(&rec),
            "oldingi saqlash javobi kutilmoqda yoki o‘zgarish yo‘q"
        );
        if first {
            let request = id_of(&rec, |c| matches!(c, Command::SaveSettings(_)));
            s.apply(Event::Reply {
                request,
                reply: Ok(Response::SettingsSaved),
            });
        }
        assert!(!s.autosave(&rec), "o‘zgarish yo‘q");
        s.bookmarks.push("/yangi".into());
        assert!(s.autosave(&rec));
        assert_eq!(saves(&rec), before + 1);
        let Some(Command::SaveSettings(settings)) = rec.0.borrow().last().cloned() else {
            panic!("save");
        };
        assert_eq!(settings.bookmarks, ["/yangi"]);
    }

    #[test]
    fn flush_on_exit_does_not_wait_for_previous_save() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        load(&mut s, &rec, Settings::default());
        s.pump(&rec);
        s.bookmarks.push("/a".into());
        assert!(s.autosave(&rec));
        s.bookmarks.push("/b".into());
        assert!(!s.autosave(&rec), "oddiy avto-saqlash javobni kutadi");
        assert!(s.flush_settings(&rec), "chiqishda kutilmaydi");
        let Some(Command::SaveSettings(settings)) = rec.0.borrow().last().cloned() else {
            panic!("save");
        };
        assert_eq!(settings.bookmarks, ["/a", "/b"]);
    }

    #[test]
    fn snapshot_uses_listing_path_and_skips_filter() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        let request = s.panel(Side::Left).pending.expect("pending");
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Directory(DirectoryListing {
                location: Location {
                    id: LocationId::from_raw(1),
                    display_path: "/Users/me".into(),
                    can_go_up: true,
                },
                entries: vec![],
                truncated: false,
                limit: 10,
                skipped_entries: 0,
            })),
        });
        s.panel_mut(Side::Left).options.filter = "vaqtinchalik".into();
        let snap = s.snapshot();
        assert_eq!(snap.session.left.tabs[0].path, "/Users/me");
        assert!(
            snap.session.right.tabs.is_empty(),
            "yo‘li noma’lum tab saqlanmaydi"
        );
        assert!(!toml_like(&snap).contains("vaqtinchalik"));
    }

    fn toml_like(settings: &Settings) -> String {
        format!("{settings:?}")
    }

    #[test]
    fn io_error_on_load_disables_saving_but_broken_file_does_not() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        let request = id_of(&rec, |c| matches!(c, Command::LoadSettings));
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Settings(Box::new(SettingsLoad {
                settings: Settings::default(),
                problem: Some(ErrorCode::PermissionDenied.into()),
            }))),
        });
        assert!(s.settings.loaded && !s.settings.saving);
        assert_eq!(
            s.opening.notice.as_ref().map(|n| n.kind),
            Some(NoticeKind::Settings)
        );
        s.bookmarks.push("/x".into());
        assert!(!s.autosave(&rec));

        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        let request = id_of(&rec, |c| matches!(c, Command::LoadSettings));
        let broken = AppError::new(ErrorCode::InvalidArgument)
            .with_detail("reason", DetailValue::Text("settings_invalid".into()));
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Settings(Box::new(SettingsLoad {
                settings: Settings::default(),
                problem: Some(broken),
            }))),
        });
        assert!(
            s.settings.saving,
            "buzilgan fayl zaxiralangan — saqlash mumkin"
        );
    }
}
