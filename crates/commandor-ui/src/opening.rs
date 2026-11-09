//! Faylni tizim dasturida ochish, "Open with" menyusi, bajariladigan fayl tasdig‘i va xabarlar.
//! GUIsiz test qilinadi; disk va jarayonga faqat broker orqali murojaat qilinadi.

use crate::palette::matches;
use crate::state::{AppState, Dispatch, Side};
use commandor_core::{
    AppChoice, AppError, Command, EntryId, EntryKind, ErrorCode, LocationId, OpenOutcome, OpenWith,
    OpenWithList, Reply, RequestId, Response,
};

/// Ochilayotgan fayl (listingdagi opaque ID va ko‘rsatish uchun nom).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRef {
    pub location: LocationId,
    pub entry: EntryId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InFlight {
    request: RequestId,
    file: FileRef,
}

/// "Open with" menyusi.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenWithMenu {
    pub file: FileRef,
    pub pending: Option<RequestId>,
    pub list: Option<OpenWithList>,
    pub error: Option<AppError>,
    pub query: String,
    pub highlighted: usize,
    /// Tavsiya etilmagan dasturlar ham ko‘rsatiladimi.
    pub show_all: bool,
}

impl OpenWithMenu {
    /// Ko‘rinadigan dasturlar: so‘rov bo‘yicha; tavsiya etilgani bo‘lmasa yoki `show_all` bo‘lsa — hammasi.
    pub fn visible(&self) -> Vec<&AppChoice> {
        let Some(list) = &self.list else {
            return Vec::new();
        };
        let any_recommended = list.apps.iter().any(|app| app.recommended);
        let searching = !self.query.trim().is_empty();
        list.apps
            .iter()
            .filter(|app| self.show_all || searching || !any_recommended || app.recommended)
            .filter(|app| matches(&self.query, &app.name))
            .collect()
    }

    pub fn has_hidden_apps(&self) -> bool {
        self.list.as_ref().is_some_and(|list| {
            !self.show_all
                && list.apps.iter().any(|app| app.recommended)
                && list.apps.iter().any(|app| !app.recommended)
        })
    }
}

/// Xabar turi: matn kaliti shunga qarab tanlanadi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    /// Faylni ochib bo‘lmadi.
    Open,
    /// Fayl amali: `total` tadan `failed` tasi bajarilmadi.
    Operation { failed: usize, total: usize },
    /// Fon vazifasi bekor qilindi: `done` ta bajarildi, qolgani tegilmadi.
    Cancelled { done: usize },
}

/// Foydalanuvchiga ko‘rsatiladigan xabar (masalan, ochilmadi).
#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub file: String,
    pub error: AppError,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OpeningState {
    in_flight: Vec<InFlight>,
    pub menu: Option<OpenWithMenu>,
    /// Bajariladigan faylni ishga tushirishni tasdiqlash kutilmoqda.
    pub confirm: Option<FileRef>,
    pub notice: Option<Notice>,
}

impl OpeningState {
    pub fn is_modal(&self) -> bool {
        self.menu.is_some() || self.confirm.is_some()
    }

    pub fn is_busy(&self) -> bool {
        !self.in_flight.is_empty()
    }
}

impl AppState {
    /// Faol paneldagi kursor ostidagi fayl (papka emas).
    pub fn cursor_file(&self, side: Side) -> Option<FileRef> {
        let panel = self.panel(side);
        let listing = panel.listing.as_ref()?;
        let entry = listing
            .entries
            .iter()
            .find(|entry| Some(entry.id) == panel.cursor)?;
        (entry.kind != EntryKind::Directory).then(|| FileRef {
            location: listing.location.id,
            entry: entry.id,
            name: entry.name.clone(),
        })
    }

    /// Enter / ikki marta bosish: papkaga kiradi, faylni tizim dasturida ochadi.
    pub fn open_cursor(&mut self, dispatch: &impl Dispatch) {
        let side = self.active;
        match self.cursor_file(side) {
            Some(file) => self.request_open(dispatch, file, OpenWith::Default, false),
            None => self.panel_mut(side).open_selected(dispatch),
        }
    }

    pub fn request_open(
        &mut self,
        dispatch: &impl Dispatch,
        file: FileRef,
        with: OpenWith,
        allow_executable: bool,
    ) {
        self.opening.notice = None;
        let command = Command::OpenEntry {
            location: file.location,
            entry: file.entry,
            with,
            allow_executable,
        };
        match dispatch.dispatch(command) {
            Some(request) => self.opening.in_flight.push(InFlight { request, file }),
            None => {
                self.opening.notice = Some(Notice {
                    kind: NoticeKind::Open,
                    file: file.name,
                    error: ErrorCode::Internal.into(),
                });
            }
        }
    }

    /// Shift+Enter / kontekst menyu: kursordagi fayl uchun "Open with" menyusi.
    pub fn open_with_cursor(&mut self, dispatch: &impl Dispatch) {
        if let Some(file) = self.cursor_file(self.active) {
            self.show_open_with(dispatch, file);
        }
    }

    pub fn show_open_with(&mut self, dispatch: &impl Dispatch, file: FileRef) {
        self.palette = None;
        self.volume_menu = None;
        self.opening.notice = None;
        let pending = dispatch.dispatch(Command::ListOpenWith {
            location: file.location,
            entry: file.entry,
        });
        self.opening.menu = Some(OpenWithMenu {
            error: pending.is_none().then(|| ErrorCode::Internal.into()),
            file,
            pending,
            list: None,
            query: String::new(),
            highlighted: 0,
            show_all: false,
        });
    }

    pub fn close_open_with(&mut self) {
        self.opening.menu = None;
    }

    pub fn step_open_with(&mut self, forward: bool) {
        let Some(menu) = self.opening.menu.as_mut() else {
            return;
        };
        let len = menu.visible().len();
        if len == 0 {
            menu.highlighted = 0;
            return;
        }
        let current = menu.highlighted.min(len - 1);
        menu.highlighted = if forward {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
    }

    pub fn toggle_show_all(&mut self) {
        if let Some(menu) = self.opening.menu.as_mut() {
            menu.show_all = !menu.show_all;
            menu.highlighted = 0;
        }
    }

    /// Enter: belgilangan dasturda ochadi.
    pub fn confirm_open_with(&mut self, dispatch: &impl Dispatch) {
        let Some(menu) = &self.opening.menu else {
            return;
        };
        let visible = menu.visible();
        if let Some(app) = visible.get(menu.highlighted.min(visible.len().saturating_sub(1))) {
            let id = app.id;
            self.choose_app(dispatch, OpenWith::App(id));
        }
    }

    /// Ro‘yxatdagi dastur yoki tizim dialogi bilan ochadi va menyuni yopadi.
    pub fn choose_app(&mut self, dispatch: &impl Dispatch, with: OpenWith) {
        if let Some(menu) = self.opening.menu.take() {
            self.request_open(dispatch, menu.file, with, false);
        }
    }

    /// Bajariladigan fayl tasdig‘i: `true` — ishga tushirish, `false` — bekor qilish.
    pub fn answer_confirm(&mut self, dispatch: &impl Dispatch, run: bool) {
        if let Some(file) = self.opening.confirm.take()
            && run
        {
            self.request_open(dispatch, file, OpenWith::Default, true);
        }
    }

    pub fn dismiss_notice(&mut self) {
        self.opening.notice = None;
    }

    /// Ochishga oid javobni qo‘llaydi; boshqa so‘rovga tegishli bo‘lsa javobni qaytarib beradi.
    pub(crate) fn apply_opening(&mut self, request: RequestId, reply: Reply) -> Option<Reply> {
        if let Some(index) = self
            .opening
            .in_flight
            .iter()
            .position(|flight| flight.request == request)
        {
            let file = self.opening.in_flight.remove(index).file;
            match reply {
                Ok(Response::Opened(OpenOutcome::Launched)) => {}
                Ok(Response::Opened(OpenOutcome::NeedsConfirmation)) => {
                    self.opening.confirm = Some(file);
                }
                Ok(_) => {
                    self.opening.notice = Some(Notice {
                        kind: NoticeKind::Open,
                        file: file.name,
                        error: ErrorCode::Internal.into(),
                    });
                }
                Err(error) => {
                    self.opening.notice = Some(Notice {
                        kind: NoticeKind::Open,
                        file: file.name,
                        error,
                    });
                }
            }
            return None;
        }
        if let Some(menu) = self.opening.menu.as_mut()
            && menu.pending == Some(request)
        {
            menu.pending = None;
            match reply {
                Ok(Response::OpenWithApps(list)) => menu.list = Some(list),
                Ok(_) => menu.error = Some(ErrorCode::Internal.into()),
                Err(error) => menu.error = Some(error),
            }
            return None;
        }
        Some(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PaletteCommand;
    use commandor_core::{AppId, DirectoryListing, Event, FileEntry, Location, Response};
    use std::cell::RefCell;

    struct Recorder(RefCell<Vec<Command>>);
    impl Dispatch for Recorder {
        fn dispatch(&self, command: Command) -> Option<RequestId> {
            let mut log = self.0.borrow_mut();
            log.push(command);
            Some(RequestId::from_raw(1000 + log.len() as u64))
        }
    }

    fn rec() -> Recorder {
        Recorder(RefCell::new(Vec::new()))
    }

    fn last(rec: &Recorder) -> Command {
        rec.0.borrow().last().cloned().expect("command")
    }

    fn entry(id: u64, name: &str, kind: EntryKind) -> FileEntry {
        FileEntry {
            id: EntryId::from_raw(id),
            name: name.into(),
            kind,
            size_bytes: Some(1),
            modified_unix_seconds: None,
        }
    }

    /// A panelda `/docs` ochilgan: `sub/` papka va `hisobot.pdf` fayli.
    fn ready(rec: &Recorder) -> AppState {
        let mut s = AppState::start(rec);
        let request = s.panel(Side::Left).pending.expect("pending");
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Directory(DirectoryListing {
                location: Location {
                    id: LocationId::from_raw(7),
                    display_path: "/docs".into(),
                    can_go_up: true,
                },
                entries: vec![
                    entry(1, "sub", EntryKind::Directory),
                    entry(2, "hisobot.pdf", EntryKind::File),
                ],
                truncated: false,
                limit: 10,
                skipped_entries: 0,
            })),
        });
        s
    }

    fn reply(s: &mut AppState, request: RequestId, reply: Reply) {
        s.apply(Event::Reply { request, reply });
    }

    fn app(id: u64, name: &str, recommended: bool) -> AppChoice {
        AppChoice {
            id: AppId::from_raw(id),
            name: name.into(),
            detail: String::new(),
            recommended,
            is_default: false,
        }
    }

    #[test]
    fn enter_navigates_into_directories_and_opens_files_with_default_app() {
        let rec = rec();
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.open_cursor(&rec);
        assert!(matches!(last(&rec), Command::ReadDirectory { .. }));

        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(2));
        s.run_command(&rec, PaletteCommand::Open);
        assert_eq!(
            last(&rec),
            Command::OpenEntry {
                location: LocationId::from_raw(7),
                entry: EntryId::from_raw(2),
                with: OpenWith::Default,
                allow_executable: false,
            }
        );
        assert!(s.opening.is_busy());
        let request = RequestId::from_raw(1000 + rec.0.borrow().len() as u64);
        reply(&mut s, request, Ok(Response::Opened(OpenOutcome::Launched)));
        assert!(!s.opening.is_busy());
        assert_eq!(s.opening.notice, None);
    }

    #[test]
    fn failure_becomes_a_notice_and_executable_needs_explicit_confirmation() {
        let rec = rec();
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(2));
        s.open_cursor(&rec);
        let request = RequestId::from_raw(1000 + rec.0.borrow().len() as u64);
        reply(&mut s, request, Err(ErrorCode::NoApplication.into()));
        let notice = s.opening.notice.clone().expect("notice");
        assert_eq!(notice.file, "hisobot.pdf");
        assert_eq!(notice.error.code, ErrorCode::NoApplication);

        s.open_cursor(&rec);
        assert_eq!(
            s.opening.notice, None,
            "yangi urinish eski xabarni tozalaydi"
        );
        let request = RequestId::from_raw(1000 + rec.0.borrow().len() as u64);
        reply(
            &mut s,
            request,
            Ok(Response::Opened(OpenOutcome::NeedsConfirmation)),
        );
        assert!(s.opening.confirm.is_some() && s.opening.is_modal());

        let sent = rec.0.borrow().len();
        s.answer_confirm(&rec, false);
        assert_eq!(
            rec.0.borrow().len(),
            sent,
            "bekor qilish hech narsa yubormaydi"
        );
        assert!(s.opening.confirm.is_none());

        s.opening.confirm = Some(FileRef {
            location: LocationId::from_raw(7),
            entry: EntryId::from_raw(2),
            name: "hisobot.pdf".into(),
        });
        s.answer_confirm(&rec, true);
        assert!(matches!(
            last(&rec),
            Command::OpenEntry {
                with: OpenWith::Default,
                allow_executable: true,
                ..
            }
        ));
    }

    #[test]
    fn open_with_menu_lists_recommended_first_filters_and_chooses() {
        let rec = rec();
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.open_with_cursor(&rec);
        assert!(s.opening.menu.is_none(), "papka uchun menyu yo‘q");

        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(2));
        s.toggle_palette();
        s.run_command(&rec, PaletteCommand::OpenWith);
        assert!(s.palette.is_none());
        assert!(matches!(last(&rec), Command::ListOpenWith { .. }));
        let pending = s
            .opening
            .menu
            .as_ref()
            .and_then(|m| m.pending)
            .expect("pending");
        reply(
            &mut s,
            pending,
            Ok(Response::OpenWithApps(OpenWithList {
                apps: vec![
                    app(1, "Preview", true),
                    app(2, "Skim", true),
                    app(3, "TextEdit", false),
                ],
                system_chooser: false,
            })),
        );
        let names = |s: &AppState| -> Vec<String> {
            let menu = s.opening.menu.as_ref().expect("menu");
            menu.visible().iter().map(|a| a.name.clone()).collect()
        };
        assert_eq!(names(&s), ["Preview", "Skim"]);
        assert!(
            s.opening
                .menu
                .as_ref()
                .is_some_and(OpenWithMenu::has_hidden_apps)
        );
        s.toggle_show_all();
        assert_eq!(names(&s), ["Preview", "Skim", "TextEdit"]);
        s.toggle_show_all();
        if let Some(menu) = s.opening.menu.as_mut() {
            menu.query = "text".into();
        }
        assert_eq!(names(&s), ["TextEdit"], "qidiruv barcha dasturlarda");
        if let Some(menu) = s.opening.menu.as_mut() {
            menu.query.clear();
        }
        s.step_open_with(false);
        assert_eq!(s.opening.menu.as_ref().map(|m| m.highlighted), Some(1));
        s.confirm_open_with(&rec);
        assert!(s.opening.menu.is_none());
        assert_eq!(
            last(&rec),
            Command::OpenEntry {
                location: LocationId::from_raw(7),
                entry: EntryId::from_raw(2),
                with: OpenWith::App(AppId::from_raw(2)),
                allow_executable: false,
            }
        );
    }

    #[test]
    fn open_with_errors_and_stale_replies() {
        let rec = rec();
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(2));
        s.open_with_cursor(&rec);
        let pending = s
            .opening
            .menu
            .as_ref()
            .and_then(|m| m.pending)
            .expect("pending");
        let before = s.clone();
        reply(&mut s, RequestId::from_raw(1), Ok(Response::Pong));
        assert_eq!(s.opening, before.opening, "begona javob menyuga tegmaydi");
        reply(&mut s, pending, Err(ErrorCode::StaleReference.into()));
        let menu = s.opening.menu.as_ref().expect("menu");
        assert_eq!(
            menu.error.as_ref().map(|e| e.code),
            Some(ErrorCode::StaleReference)
        );
        assert!(menu.visible().is_empty());
        s.close_open_with();
        assert!(!s.opening.is_modal());
    }

    #[test]
    fn without_recommendations_all_apps_are_visible() {
        let menu = OpenWithMenu {
            file: FileRef {
                location: LocationId::from_raw(1),
                entry: EntryId::from_raw(1),
                name: "x.qwe".into(),
            },
            pending: None,
            list: Some(OpenWithList {
                apps: vec![app(1, "A", false), app(2, "B", false)],
                system_chooser: true,
            }),
            error: None,
            query: String::new(),
            highlighted: 0,
            show_all: false,
        };
        assert_eq!(menu.visible().len(), 2);
        assert!(!menu.has_hidden_apps());
    }
}
