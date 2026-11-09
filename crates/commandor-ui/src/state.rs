//! GUIsiz test qilinadigan UI holati. Faqat broker hodisalari bilan yangilanadi.

use commandor_core::{
    AppError, AppInfo, Command, DirectoryListing, DirectoryTarget, EntryId, EntryKind, ErrorCode,
    Event, ListOptions, Reply, RequestId, Response, SortDirection, SortField, Volume,
    volume_index_for_path,
};
use std::collections::BTreeSet;

pub use crate::palette::{PaletteCommand, PaletteItem, PaletteState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
    pub const fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrokerStatus {
    Connecting { pending: RequestId },
    Ready(AppInfo),
    Failed(AppError),
    Stopped,
}

/// Bir paneldagi tablar soni chegarasi.
pub const MAX_TABS: usize = 12;
/// Xatcho‘plar soni chegarasi.
pub const MAX_BOOKMARKS: usize = 50;
/// PageUp/PageDown bir bosishda o‘tadigan qatorlar.
pub const PAGE_ROWS: isize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub active: Side,
    pub broker: BrokerStatus,
    pub panels: [PanelTabs; 2],
    pub volumes: VolumesState,
    /// Ochiq tomlar menyusi (F1 — A panel, F2 — B panel).
    pub volume_menu: Option<VolumeMenu>,
    /// Xatcho‘plar: mutlaq papka yo‘llari. Hozircha faqat sessiya ichida (saqlash — P3-07).
    pub bookmarks: Vec<String>,
    /// Ochiq buyruqlar paneli.
    pub palette: Option<PaletteState>,
}

/// Holatdan tashqari, UI bajaradigan natija (masalan, OS dialogi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEffect {
    PickFolder,
}

/// Bitta panelning tablari; har tab o‘z listing, tarix va tanloviga ega.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelTabs {
    pub tabs: Vec<PanelState>,
    pub current: usize,
    /// Keyingi tabga beriladigan barqaror ID (UI holati, masalan scroll, shunga bog‘lanadi).
    next_tab_id: u64,
}

impl PanelTabs {
    pub fn start(dispatch: &impl Dispatch) -> Self {
        let mut panel = PanelState::default();
        panel.navigate(dispatch, DirectoryTarget::Home);
        Self {
            tabs: vec![panel],
            current: 0,
            next_tab_id: 1,
        }
    }

    pub fn active(&self) -> &PanelState {
        &self.tabs[self.current]
    }

    pub fn active_mut(&mut self) -> &mut PanelState {
        &mut self.tabs[self.current]
    }

    pub fn can_open(&self) -> bool {
        self.tabs.len() < MAX_TABS
    }

    pub fn can_close(&self) -> bool {
        self.tabs.len() > 1
    }

    /// Joriy papka va ko‘rinish sozlamalari bilan yangi tabni joriy tabdan keyin ochadi.
    pub fn open_tab(&mut self, dispatch: &impl Dispatch) {
        if !self.can_open() {
            return;
        }
        let source = self.active();
        let target = source
            .listing
            .as_ref()
            .map_or(DirectoryTarget::Home, |listing| {
                DirectoryTarget::Path(listing.location.display_path.clone())
            });
        let mut tab = PanelState {
            options: source.options.clone(),
            tab_id: self.next_tab_id,
            ..PanelState::default()
        };
        self.next_tab_id += 1;
        tab.navigate(dispatch, target);
        self.current += 1;
        self.tabs.insert(self.current, tab);
    }

    /// Tabni yopadi; oxirgi tab yopilmaydi. Yopilgan tabning kutilayotgan javobi keyin e’tiborsiz qoladi.
    pub fn close_tab(&mut self, index: usize) {
        if !self.can_close() || index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if index < self.current || self.current >= self.tabs.len() {
            self.current -= 1;
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.current = index;
        }
    }

    /// Ctrl+Tab / Ctrl+Shift+Tab: tablar bo‘ylab aylanma o‘tish.
    pub fn cycle(&mut self, forward: bool) {
        let len = self.tabs.len();
        self.current = if forward {
            (self.current + 1) % len
        } else {
            (self.current + len - 1) % len
        };
    }
}

/// Yo‘lning oxirgi qismi (tab nomi uchun). Ildiz yo‘llar o‘zicha qaytadi: `/`, `C:`.
pub fn path_label(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        return path;
    }
    match trimmed.rfind(['/', '\\']) {
        Some(index) => &trimmed[index + 1..],
        None => trimmed,
    }
}

/// Broker qaytargan tomlar ro‘yxati.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VolumesState {
    pub list: Vec<Volume>,
    pub pending: Option<RequestId>,
    pub error: Option<AppError>,
}

/// Tomlar menyusi holati: qaysi panel uchun va qaysi qator belgilangan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeMenu {
    pub side: Side,
    pub highlighted: usize,
    /// Foydalanuvchi strelka bilan yurganmi; yurmagan bo‘lsa yangi ro‘yxatda joriy tom belgilanadi.
    pub touched: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PanelState {
    pub listing: Option<DirectoryListing>,
    pub pending: Option<RequestId>,
    pub error: Option<AppError>,
    pub path_input: String,
    pub options: ListOptions,
    /// Kursordagi faol element (fokuslangan yozuv)
    pub cursor: Option<EntryId>,
    /// Belgilangan/tanlangan elementlar to‘plami (multi-select)
    pub selected: BTreeSet<EntryId>,
    /// Navigatsiya tarixi
    pub history: Vec<DirectoryTarget>,
    pub history_index: usize,
    /// Kursor klaviatura bilan siljidi: jadval uni ko‘rinadigan joyga aylantirishi kerak.
    pub reveal_cursor: bool,
    /// Panel ichida barqaror tab ID (indeks tab yopilganda o‘zgaradi, bu esa yo‘q).
    pub tab_id: u64,
}

impl PanelState {
    /// Tab sarlavhasi: joriy papka nomi (yuklanmagan bo‘lsa — kiritilgan yo‘l).
    pub fn title(&self) -> &str {
        let path = self
            .listing
            .as_ref()
            .map_or(self.path_input.as_str(), |listing| {
                listing.location.display_path.as_str()
            });
        path_label(path)
    }

    /// Joriy papkaning mutlaq yo‘li (listing kelgan bo‘lsa).
    pub fn current_path(&self) -> Option<&str> {
        self.listing
            .as_ref()
            .map(|listing| listing.location.display_path.as_str())
    }

    pub fn select_all(&mut self) {
        if let Some(listing) = &self.listing {
            self.selected = listing.entries.iter().map(|entry| entry.id).collect();
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn load(&mut self, dispatch: &impl Dispatch, target: DirectoryTarget) {
        self.error = None;
        self.pending = dispatch.dispatch(Command::ReadDirectory {
            target,
            options: self.options.clone(),
        });
        if self.pending.is_none() {
            self.error = Some(ErrorCode::Internal.into());
        }
    }

    /// Yangi manzilga o‘tish va tarixga qo‘shish.
    pub fn navigate(&mut self, dispatch: &impl Dispatch, target: DirectoryTarget) {
        self.cursor = None;
        self.selected.clear();
        if !self.history.is_empty() && self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(target.clone());
        const MAX_HISTORY: usize = 100;
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        self.history_index = self.history.len().saturating_sub(1);
        self.load(dispatch, target);
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self, dispatch: &impl Dispatch) {
        if self.can_go_back() {
            self.history_index -= 1;
            let target = self.history[self.history_index].clone();
            self.load(dispatch, target);
        }
    }

    pub fn go_forward(&mut self, dispatch: &impl Dispatch) {
        if self.can_go_forward() {
            self.history_index += 1;
            let target = self.history[self.history_index].clone();
            self.load(dispatch, target);
        }
    }

    pub fn refresh(&mut self, dispatch: &impl Dispatch) {
        let target = self.listing.as_ref().map_or(DirectoryTarget::Home, |l| {
            DirectoryTarget::Refresh(l.location.id)
        });
        self.load(dispatch, target);
    }

    pub fn parent(&mut self, dispatch: &impl Dispatch) {
        if let Some(listing) = &self.listing
            && listing.location.can_go_up
        {
            self.navigate(dispatch, DirectoryTarget::Parent(listing.location.id));
        }
    }

    pub fn open_selected(&mut self, dispatch: &impl Dispatch) {
        let Some(listing) = &self.listing else {
            return;
        };
        if let Some(entry) = listing
            .entries
            .iter()
            .find(|entry| Some(entry.id) == self.cursor)
            && entry.kind == EntryKind::Directory
        {
            self.navigate(
                dispatch,
                DirectoryTarget::Child {
                    location: listing.location.id,
                    entry: entry.id,
                },
            );
        }
    }

    pub fn step_selection(&mut self, forward: bool) {
        self.move_cursor(if forward { 1 } else { -1 });
    }

    /// Kursorni `delta` qator siljitadi (chegarada to‘xtaydi). Kursor bo‘lmasa birinchi qatorga qo‘yiladi.
    pub fn move_cursor(&mut self, delta: isize) {
        let Some(listing) = &self.listing else {
            return;
        };
        let last = listing.entries.len().saturating_sub(1);
        if listing.entries.is_empty() {
            return;
        }
        let index = listing
            .entries
            .iter()
            .position(|entry| Some(entry.id) == self.cursor);
        let next = index.map_or(0, |i| i.saturating_add_signed(delta).min(last));
        self.cursor = Some(listing.entries[next].id);
        self.reveal_cursor = true;
    }

    /// Home/End: birinchi yoki oxirgi qator.
    pub fn cursor_to_edge(&mut self, end: bool) {
        let Some(listing) = &self.listing else {
            return;
        };
        let entry = if end {
            listing.entries.last()
        } else {
            listing.entries.first()
        };
        if let Some(entry) = entry {
            self.cursor = Some(entry.id);
            self.reveal_cursor = true;
        }
    }

    pub fn toggle_selected(&mut self, id: EntryId) {
        if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
    }

    /// Kursordagi elementni tanlash/tanlovni bekor qilish va keyingisiga o‘tish (Space/Insert).
    pub fn select_current_and_step(&mut self) {
        if let Some(id) = self.cursor {
            self.toggle_selected(id);
            self.step_selection(true);
        }
    }

    /// Tanlangan fayllar soni va umumiy hajmi baytda.
    pub fn selected_stats(&self) -> (usize, u64) {
        let Some(listing) = &self.listing else {
            return (0, 0);
        };
        let mut count = 0;
        let mut total_bytes = 0;
        for entry in &listing.entries {
            if self.selected.contains(&entry.id) {
                count += 1;
                total_bytes += entry.size_bytes.unwrap_or(0);
            }
        }
        (count, total_bytes)
    }

    /// Ustun bo‘yicha saralashni almashtirish.
    pub fn toggle_sort(&mut self, dispatch: &impl Dispatch, field: SortField) {
        if self.options.sort.field == field {
            self.options.sort.direction = match self.options.sort.direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.options.sort.field = field;
            self.options.sort.direction = SortDirection::Ascending;
        }
        self.refresh(dispatch);
    }

    fn apply(&mut self, reply: Reply) {
        self.pending = None;
        match reply {
            Ok(Response::Directory(listing)) => {
                self.path_input.clone_from(&listing.location.display_path);
                // Tarixdagi joriy elementni mustahkam mutlaq yo‘l bilan mustahkamlash
                if let Some(target) = self.history.get_mut(self.history_index) {
                    *target = DirectoryTarget::Path(listing.location.display_path.clone());
                }
                // Saralash yoki yangilanishda mavjud tanlov va kursorni yangi ro‘yxatda bor bo‘lsa saqlab qolish
                let new_ids: BTreeSet<EntryId> =
                    listing.entries.iter().map(|entry| entry.id).collect();
                self.selected.retain(|id| new_ids.contains(id));
                if let Some(cursor) = self.cursor
                    && !new_ids.contains(&cursor)
                {
                    self.cursor = None;
                }
                self.listing = Some(listing);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
            _ => self.error = Some(ErrorCode::Internal.into()),
        }
    }
}

/// Brokerga buyruq yuborish abstraksiyasi (testda soxta implementatsiya).
pub trait Dispatch {
    fn dispatch(&self, command: Command) -> Option<RequestId>;
}

impl Dispatch for commandor_broker::AppHandle {
    fn dispatch(&self, command: Command) -> Option<RequestId> {
        commandor_broker::AppHandle::dispatch(self, command)
    }
}

impl AppState {
    pub fn start(dispatch: &impl Dispatch) -> Self {
        let broker = match dispatch.dispatch(Command::GetAppInfo) {
            Some(pending) => BrokerStatus::Connecting { pending },
            None => BrokerStatus::Stopped,
        };
        let panels = [PanelTabs::start(dispatch), PanelTabs::start(dispatch)];
        let mut state = Self {
            active: Side::Left,
            broker,
            panels,
            volumes: VolumesState::default(),
            volume_menu: None,
            bookmarks: Vec::new(),
            palette: None,
        };
        state.request_volumes(dispatch);
        state
    }

    /// Panelning faol tabi.
    pub fn panel(&self, side: Side) -> &PanelState {
        self.panels[side.index()].active()
    }

    pub fn panel_mut(&mut self, side: Side) -> &mut PanelState {
        self.panels[side.index()].active_mut()
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        self.panel_mut(self.active)
    }

    /// Panelning joriy papkasini xatcho‘plarga qo‘shadi (takror qo‘shilmaydi).
    pub fn add_bookmark(&mut self, side: Side) {
        let Some(path) = self.panel(side).current_path() else {
            return;
        };
        if self.bookmarks.iter().any(|mark| mark == path) || self.bookmarks.len() >= MAX_BOOKMARKS {
            return;
        }
        let path = path.to_owned();
        self.bookmarks.push(path);
    }

    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    pub fn is_bookmarked(&self, side: Side) -> bool {
        self.panel(side)
            .current_path()
            .is_some_and(|path| self.bookmarks.iter().any(|mark| mark == path))
    }

    /// Panelni xatcho‘p papkasiga o‘tkazadi (tarixga yoziladi).
    pub fn open_bookmark(&mut self, dispatch: &impl Dispatch, side: Side, index: usize) {
        let Some(path) = self.bookmarks.get(index).cloned() else {
            return;
        };
        self.active = side;
        self.panel_mut(side)
            .navigate(dispatch, DirectoryTarget::Path(path));
    }

    /// Buyruqlar panelini ochadi/yopadi; tomlar menyusi yopiladi.
    pub fn toggle_palette(&mut self) {
        if self.palette.take().is_none() {
            self.volume_menu = None;
            self.palette = Some(PaletteState::default());
        }
    }

    /// Belgilangan qatorni `len` ta mos qator ichida aylanma siljitadi.
    pub fn step_palette(&mut self, forward: bool, len: usize) {
        let Some(palette) = self.palette.as_mut() else {
            return;
        };
        if len == 0 {
            palette.highlighted = 0;
            return;
        }
        let current = palette.highlighted.min(len - 1);
        palette.highlighted = if forward {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
    }

    /// Panel qatorini bajaradi va panelni yopadi.
    pub fn run_palette_item(
        &mut self,
        dispatch: &impl Dispatch,
        item: PaletteItem,
    ) -> Option<UiEffect> {
        self.palette = None;
        match item {
            PaletteItem::Command(command) => self.run_command(dispatch, command),
            PaletteItem::Bookmark(index) => {
                self.open_bookmark(dispatch, self.active, index);
                None
            }
        }
    }

    /// Faol panelga buyruqni qo‘llaydi. Tashqi dialog kerak bo‘lsa `UiEffect` qaytadi.
    pub fn run_command(
        &mut self,
        dispatch: &impl Dispatch,
        command: PaletteCommand,
    ) -> Option<UiEffect> {
        let side = self.active;
        let panel = self.panels[side.index()].active_mut();
        match command {
            PaletteCommand::GoHome => panel.navigate(dispatch, DirectoryTarget::Home),
            PaletteCommand::GoUp => panel.parent(dispatch),
            PaletteCommand::GoBack => panel.go_back(dispatch),
            PaletteCommand::GoForward => panel.go_forward(dispatch),
            PaletteCommand::Refresh => panel.refresh(dispatch),
            PaletteCommand::ToggleHidden => {
                panel.options.hidden = !panel.options.hidden;
                panel.refresh(dispatch);
            }
            PaletteCommand::SortName => panel.toggle_sort(dispatch, SortField::Name),
            PaletteCommand::SortSize => panel.toggle_sort(dispatch, SortField::Size),
            PaletteCommand::SortModified => panel.toggle_sort(dispatch, SortField::Modified),
            PaletteCommand::SelectAll => panel.select_all(),
            PaletteCommand::DeselectAll => panel.clear_selection(),
            PaletteCommand::NewTab => self.panels[side.index()].open_tab(dispatch),
            PaletteCommand::CloseTab => {
                let tabs = &mut self.panels[side.index()];
                tabs.close_tab(tabs.current);
            }
            PaletteCommand::TogglePanel => self.toggle_active(),
            PaletteCommand::AddBookmark => self.add_bookmark(side),
            PaletteCommand::PickFolder => return Some(UiEffect::PickFolder),
        }
        None
    }

    /// Tomlar ro‘yxatini brokerdan qayta so‘raydi (eski ro‘yxat javob kelguncha ko‘rinib turadi).
    pub fn request_volumes(&mut self, dispatch: &impl Dispatch) {
        self.volumes.pending = dispatch.dispatch(Command::ListVolumes);
        if self.volumes.pending.is_none() {
            self.volumes.error = Some(ErrorCode::Internal.into());
        }
    }

    /// Panelning joriy papkasi joylashgan tom indeksi.
    pub fn current_volume(&self, side: Side) -> Option<usize> {
        let listing = self.panel(side).listing.as_ref()?;
        volume_index_for_path(&self.volumes.list, &listing.location.display_path)
    }

    /// F1/F2: shu panel menyusi ochiq bo‘lsa yopadi, aks holda ochadi va panelni faol qiladi.
    pub fn toggle_volume_menu(&mut self, dispatch: &impl Dispatch, side: Side) {
        if self.volume_menu.is_some_and(|menu| menu.side == side) {
            self.volume_menu = None;
            return;
        }
        self.active = side;
        self.palette = None;
        self.volume_menu = Some(VolumeMenu {
            side,
            highlighted: self.current_volume(side).unwrap_or(0),
            touched: false,
        });
        self.request_volumes(dispatch);
    }

    pub fn close_volume_menu(&mut self) {
        self.volume_menu = None;
    }

    /// Yuqori/pastki strelka: belgilangan qatorni aylanma tarzda siljitadi.
    pub fn step_volume_menu(&mut self, forward: bool) {
        let len = self.volumes.list.len();
        let Some(menu) = self.volume_menu.as_mut() else {
            return;
        };
        if len == 0 {
            return;
        }
        let current = menu.highlighted.min(len - 1);
        menu.highlighted = if forward {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
        menu.touched = true;
    }

    /// Belgilangan qatorni sichqoncha bilan o‘zgartirish.
    pub fn highlight_volume(&mut self, index: usize) {
        if let Some(menu) = self.volume_menu.as_mut()
            && index < self.volumes.list.len()
        {
            menu.highlighted = index;
            menu.touched = true;
        }
    }

    /// Enter: belgilangan tomga o‘tadi va menyuni yopadi.
    pub fn confirm_volume_menu(&mut self, dispatch: &impl Dispatch) {
        if let Some(menu) = self.volume_menu {
            self.choose_volume(dispatch, menu.side, menu.highlighted);
        }
    }

    /// Panelni tom ildiziga o‘tkazadi (tarixga yoziladi). Noto‘g‘ri indeks e’tiborsiz qoladi.
    pub fn choose_volume(&mut self, dispatch: &impl Dispatch, side: Side, index: usize) {
        let Some(volume) = self.volumes.list.get(index) else {
            return;
        };
        let target = DirectoryTarget::Path(volume.mount_path.clone());
        self.active = side;
        self.volume_menu = None;
        self.panel_mut(side).navigate(dispatch, target);
    }

    pub fn toggle_active(&mut self) {
        self.active = self.active.other();
    }

    /// Hodisani qo‘llaydi. Kutilmagan yoki eskirgan `RequestId` javobi holatni o‘zgartirmaydi.
    pub fn apply(&mut self, event: Event) {
        // `Event` #[non_exhaustive]: boshqa hodisalar keyingi bosqichlarda qo‘shiladi.
        let Event::Reply { request, reply } = event else {
            return;
        };
        // Fondagi tablar ham o‘z javobini oladi; yopilgan tab javobi hech kimga tegmaydi.
        for panel in self.panels.iter_mut().flat_map(|tabs| tabs.tabs.iter_mut()) {
            if panel.pending == Some(request) {
                panel.apply(reply);
                return;
            }
        }
        if self.volumes.pending == Some(request) {
            self.volumes.pending = None;
            match reply {
                Ok(Response::Volumes(list)) => {
                    self.volumes.list = list;
                    self.volumes.error = None;
                }
                Ok(_) => self.volumes.error = Some(ErrorCode::Internal.into()),
                Err(error) => self.volumes.error = Some(error),
            }
            let len = self.volumes.list.len();
            if let Some(mut menu) = self.volume_menu {
                if !menu.touched {
                    menu.highlighted = self.current_volume(menu.side).unwrap_or(0);
                }
                menu.highlighted = menu.highlighted.min(len.saturating_sub(1));
                self.volume_menu = Some(menu);
            }
            return;
        }
        let BrokerStatus::Connecting { pending } = self.broker else {
            return;
        };
        if request != pending {
            return;
        }
        self.broker = match reply {
            Ok(Response::AppInfo(info)) => BrokerStatus::Ready(info),
            Ok(_) => BrokerStatus::Failed(AppError::new(ErrorCode::Internal)),
            Err(err) => BrokerStatus::Failed(err),
        };
    }

    pub const fn is_ready(&self) -> bool {
        matches!(self.broker, BrokerStatus::Ready(_))
    }

    pub fn listings_ready(&self) -> bool {
        self.panels.iter().all(|tabs| {
            let panel = tabs.active();
            panel.listing.is_some() && panel.pending.is_none() && panel.error.is_none()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Fake(Cell<u64>, bool);
    impl Dispatch for Fake {
        fn dispatch(&self, _: Command) -> Option<RequestId> {
            self.1.then(|| {
                self.0.set(self.0.get() + 1);
                RequestId::from_raw(self.0.get())
            })
        }
    }

    fn info() -> AppInfo {
        AppInfo {
            version: "0.1.0".into(),
            api_version: 1,
            os: "test".into(),
            arch: "x".into(),
        }
    }

    #[test]
    fn tab_toggles_between_independent_panels() {
        let mut s = AppState::start(&Fake(Cell::new(0), true));
        assert_eq!(s.active, Side::Left);
        s.toggle_active();
        assert_eq!(s.active, Side::Right);
        s.toggle_active();
        assert_eq!(s.active, Side::Left);
    }

    #[test]
    fn matching_reply_makes_ready_and_stale_reply_is_ignored() {
        let mut s = AppState::start(&Fake(Cell::new(0), true));
        s.apply(Event::Reply {
            request: RequestId::from_raw(99),
            reply: Ok(Response::AppInfo(info())),
        });
        assert!(!s.is_ready(), "begona javob qabul qilinmaydi");
        s.apply(Event::Reply {
            request: RequestId::from_raw(1),
            reply: Ok(Response::AppInfo(info())),
        });
        assert_eq!(s.broker, BrokerStatus::Ready(info()));
    }

    #[test]
    fn error_reply_and_stopped_broker_are_distinct_states() {
        let mut s = AppState::start(&Fake(Cell::new(0), true));
        s.apply(Event::Reply {
            request: RequestId::from_raw(1),
            reply: Err(AppError::new(ErrorCode::UnsupportedApiVersion)),
        });
        assert!(
            matches!(s.broker, BrokerStatus::Failed(ref e) if e.code == ErrorCode::UnsupportedApiVersion)
        );
        let stopped = AppState::start(&Fake(Cell::new(0), false));
        assert_eq!(stopped.broker, BrokerStatus::Stopped);
    }

    fn listing(id: u64, path: &str) -> DirectoryListing {
        DirectoryListing {
            location: commandor_core::Location {
                id: commandor_core::LocationId::from_raw(id),
                display_path: path.into(),
                can_go_up: true,
            },
            entries: vec![],
            truncated: false,
            limit: 5000,
            skipped_entries: 0,
        }
    }

    #[test]
    fn independent_panels_ignore_stale_and_out_of_order_results() {
        let dispatch = Fake(Cell::new(0), true);
        let mut state = AppState::start(&dispatch);
        let left_old = state.panels[0].tabs[0].pending.expect("pending");
        let right = state.panels[1].tabs[0].pending.expect("pending");
        state.panels[0].tabs[0].load(&dispatch, DirectoryTarget::Path("/new".into()));
        let left_new = state.panels[0].tabs[0].pending.expect("pending");
        state.apply(Event::Reply {
            request: right,
            reply: Ok(Response::Directory(listing(2, "/right"))),
        });
        state.apply(Event::Reply {
            request: left_new,
            reply: Ok(Response::Directory(listing(3, "/new"))),
        });
        state.apply(Event::Reply {
            request: left_old,
            reply: Ok(Response::Directory(listing(1, "/old"))),
        });
        assert_eq!(state.panels[0].tabs[0].path_input, "/new");
        assert_eq!(state.panels[1].tabs[0].path_input, "/right");
        assert!(state.listings_ready());
    }

    #[test]
    fn failed_navigation_keeps_last_good_location_and_error_is_not_empty_folder() {
        let dispatch = Fake(Cell::new(0), true);
        let mut state = AppState::start(&dispatch);
        let id = state.panels[0].tabs[0].pending.expect("pending");
        state.apply(Event::Reply {
            request: id,
            reply: Ok(Response::Directory(listing(1, "/good"))),
        });
        state.panels[0].tabs[0].load(&dispatch, DirectoryTarget::Path("/missing".into()));
        let id = state.panels[0].tabs[0].pending.expect("pending");
        state.apply(Event::Reply {
            request: id,
            reply: Err(ErrorCode::NotFound.into()),
        });
        assert_eq!(state.panels[0].tabs[0].path_input, "/good");
        assert_eq!(
            state.panels[0].tabs[0].error.as_ref().expect("error").code,
            ErrorCode::NotFound
        );
        assert!(state.panels[0].tabs[0].listing.is_some());
        assert!(!state.listings_ready());
    }

    fn test_entry(id: u64, name: &str, kind: EntryKind) -> commandor_core::FileEntry {
        commandor_core::FileEntry {
            id: EntryId::from_raw(id),
            name: name.into(),
            kind,
            size_bytes: Some(123),
            modified_unix_seconds: Some(1794054600),
        }
    }

    #[test]
    fn step_selection_navigates_within_bounds() {
        let mut panel = PanelState::default();
        let mut list = listing(1, "/test");
        list.entries = vec![
            test_entry(10, "dir", EntryKind::Directory),
            test_entry(11, "file1.txt", EntryKind::File),
            test_entry(12, "file2.txt", EntryKind::File),
        ];
        panel.listing = Some(list);

        assert_eq!(panel.cursor, None);
        // Boshlang'ich qadam birinchi elementni tanlaydi
        panel.step_selection(true);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(10)));
        // Keyingi element
        panel.step_selection(true);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(11)));
        panel.step_selection(true);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(12)));
        // Chegaradan oshmaydi
        panel.step_selection(true);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(12)));
        // Orqaga qaytish
        panel.step_selection(false);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(11)));
        panel.step_selection(false);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(10)));
        // Yuqori chegaradan oshmaydi
        panel.step_selection(false);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(10)));
    }

    #[test]
    fn open_selected_only_opens_directories() {
        let dispatch = Fake(Cell::new(0), true);
        let mut panel = PanelState::default();
        let mut list = listing(1, "/test");
        list.entries = vec![
            test_entry(10, "dir", EntryKind::Directory),
            test_entry(11, "file.txt", EntryKind::File),
        ];
        panel.listing = Some(list);

        // Fayl tanlanganda open_selected dispatch qilmaydi
        panel.cursor = Some(EntryId::from_raw(11));
        panel.open_selected(&dispatch);
        assert!(panel.pending.is_none());

        // Papka tanlanganda open_selected child papkani ochadi
        panel.cursor = Some(EntryId::from_raw(10));
        panel.open_selected(&dispatch);
        assert!(panel.pending.is_some());
    }

    #[test]
    fn multi_selection_and_stats() {
        let mut panel = PanelState::default();
        let mut list = listing(1, "/test");
        list.entries = vec![
            test_entry(10, "dir", EntryKind::Directory),
            test_entry(11, "file1.txt", EntryKind::File),
            test_entry(12, "file2.txt", EntryKind::File),
        ];
        panel.listing = Some(list);

        assert_eq!(panel.selected_stats(), (0, 0));

        // Space/Insert bilan tanlash va keyingisiga o‘tish
        panel.cursor = Some(EntryId::from_raw(11));
        panel.select_current_and_step();
        assert!(panel.selected.contains(&EntryId::from_raw(11)));
        assert_eq!(panel.cursor, Some(EntryId::from_raw(12)));
        assert_eq!(panel.selected_stats(), (1, 123));

        // Ikkinchi faylni tanlash
        panel.select_current_and_step();
        assert_eq!(panel.selected_stats(), (2, 246));

        // Qayta bosganda tanlov bekor qilinishi
        panel.toggle_selected(EntryId::from_raw(11));
        assert_eq!(panel.selected_stats(), (1, 123));
    }

    #[test]
    fn history_navigation_back_and_forward() {
        let dispatch = Fake(Cell::new(0), true);
        let mut panel = PanelState::default();

        assert!(!panel.can_go_back());
        assert!(!panel.can_go_forward());

        panel.navigate(&dispatch, DirectoryTarget::Home);
        assert!(!panel.can_go_back());

        panel.navigate(&dispatch, DirectoryTarget::Path("/folder1".into()));
        assert!(panel.can_go_back());
        assert!(!panel.can_go_forward());

        panel.navigate(&dispatch, DirectoryTarget::Path("/folder2".into()));
        assert_eq!(panel.history_index, 2);

        // Orqaga qaytish
        panel.go_back(&dispatch);
        assert_eq!(panel.history_index, 1);
        assert!(panel.can_go_forward());

        // Yana orqaga
        panel.go_back(&dispatch);
        assert_eq!(panel.history_index, 0);
        assert!(!panel.can_go_back());

        // Oldinga o'tish
        panel.go_forward(&dispatch);
        assert_eq!(panel.history_index, 1);
    }

    #[test]
    fn toggle_sort_changes_field_and_direction() {
        let dispatch = Fake(Cell::new(0), true);
        let mut panel = PanelState::default();

        assert_eq!(panel.options.sort.field, SortField::Name);
        assert_eq!(panel.options.sort.direction, SortDirection::Ascending);

        // Shu ustun qayta bosilsa yo'nalish o'zgaradi
        panel.toggle_sort(&dispatch, SortField::Name);
        assert_eq!(panel.options.sort.direction, SortDirection::Descending);

        // Boshqa ustun bosilsa yo'nalish Ascending bo'ladi
        panel.toggle_sort(&dispatch, SortField::Size);
        assert_eq!(panel.options.sort.field, SortField::Size);
        assert_eq!(panel.options.sort.direction, SortDirection::Ascending);
    }

    #[test]
    fn refresh_preserves_valid_selection_and_cursor() {
        let dispatch = Fake(Cell::new(0), true);
        let mut panel = PanelState::default();
        let mut list = listing(1, "/test");
        list.entries = vec![
            test_entry(10, "dir", EntryKind::Directory),
            test_entry(11, "file1.txt", EntryKind::File),
            test_entry(12, "file2.txt", EntryKind::File),
        ];
        panel.apply(Ok(Response::Directory(list.clone())));
        panel.cursor = Some(EntryId::from_raw(11));
        panel.selected.insert(EntryId::from_raw(11));
        panel.selected.insert(EntryId::from_raw(12));

        // Yangilanishda (masalan, sort yoki refresh) mavjud elementlar saqlanadi
        let mut reordered = list.clone();
        reordered.entries.reverse();
        panel.apply(Ok(Response::Directory(reordered)));

        assert_eq!(panel.cursor, Some(EntryId::from_raw(11)));
        assert_eq!(panel.selected.len(), 2);
        assert!(panel.selected.contains(&EntryId::from_raw(11)));
        assert!(panel.selected.contains(&EntryId::from_raw(12)));

        // Agar element ro'yxatdan yo'qolsa, tanlovdan ham chiqariladi
        let mut partial = list;
        partial.entries.truncate(1); // faqat 10 qoldi
        panel.apply(Ok(Response::Directory(partial)));
        assert_eq!(panel.cursor, None);
        assert_eq!(panel.selected.len(), 0);

        // Yangi katalogga navigate qilganda esa tanlov tozalanadi
        panel.cursor = Some(EntryId::from_raw(10));
        panel.selected.insert(EntryId::from_raw(10));
        panel.navigate(&dispatch, DirectoryTarget::Path("/other".into()));
        assert_eq!(panel.cursor, None);
        assert_eq!(panel.selected.len(), 0);
    }

    /// Yuborilgan buyruqlarni yozib boruvchi soxta dispatch.
    struct Recorder(std::cell::RefCell<Vec<Command>>);
    impl Dispatch for Recorder {
        fn dispatch(&self, command: Command) -> Option<RequestId> {
            let mut log = self.0.borrow_mut();
            log.push(command);
            Some(RequestId::from_raw(log.len() as u64))
        }
    }

    fn vol(name: &str, mount: &str) -> Volume {
        Volume {
            name: name.into(),
            mount_path: mount.into(),
            kind: commandor_core::VolumeKind::Internal,
            file_system: "apfs".into(),
            total_bytes: 100,
            available_bytes: 40,
            read_only: false,
        }
    }

    fn with_volumes(rec: &Recorder) -> AppState {
        let mut s = AppState::start(rec);
        let request = s.volumes.pending.expect("volumes pending");
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Volumes(vec![
                vol("Macintosh HD", "/"),
                vol("yojik HD", "/Volumes/yojik HD"),
                vol("USB", "/Volumes/USB"),
            ])),
        });
        s
    }

    #[test]
    fn start_requests_volume_list_and_reply_fills_it() {
        let rec = Recorder(std::cell::RefCell::new(Vec::new()));
        let s = with_volumes(&rec);
        assert!(rec.0.borrow().contains(&Command::ListVolumes));
        assert_eq!(s.volumes.list.len(), 3);
        assert!(s.volumes.pending.is_none());
    }

    #[test]
    fn f_keys_toggle_menu_for_their_panel_and_activate_it() {
        let rec = Recorder(std::cell::RefCell::new(Vec::new()));
        let mut s = with_volumes(&rec);
        s.toggle_volume_menu(&rec, Side::Right);
        assert_eq!(s.active, Side::Right);
        assert_eq!(s.volume_menu.map(|m| m.side), Some(Side::Right));
        s.toggle_volume_menu(&rec, Side::Left);
        assert_eq!(
            s.volume_menu.map(|m| m.side),
            Some(Side::Left),
            "boshqa panel menyusiga o‘tadi"
        );
        s.toggle_volume_menu(&rec, Side::Left);
        assert!(
            s.volume_menu.is_none(),
            "o‘sha tugma qayta bosilsa yopiladi"
        );
    }

    #[test]
    fn arrows_wrap_and_enter_navigates_to_mount_path() {
        let rec = Recorder(std::cell::RefCell::new(Vec::new()));
        let mut s = with_volumes(&rec);
        s.toggle_volume_menu(&rec, Side::Left);
        s.step_volume_menu(false);
        assert_eq!(
            s.volume_menu.map(|m| m.highlighted),
            Some(2),
            "yuqoriga aylanadi"
        );
        s.step_volume_menu(true);
        s.step_volume_menu(true);
        assert_eq!(s.volume_menu.map(|m| m.highlighted), Some(1));
        s.confirm_volume_menu(&rec);
        assert!(s.volume_menu.is_none());
        assert_eq!(
            rec.0.borrow().last(),
            Some(&Command::ReadDirectory {
                target: DirectoryTarget::Path("/Volumes/yojik HD".into()),
                options: ListOptions::default(),
            })
        );
        assert_eq!(
            s.panels[0].tabs[0].history.last(),
            Some(&DirectoryTarget::Path("/Volumes/yojik HD".into()))
        );
    }

    #[test]
    fn menu_highlights_current_volume_of_panel() {
        let rec = Recorder(std::cell::RefCell::new(Vec::new()));
        let mut s = with_volumes(&rec);
        let id = s.panels[1].tabs[0].pending.expect("pending");
        s.apply(Event::Reply {
            request: id,
            reply: Ok(Response::Directory(listing(5, "/Volumes/USB/photos"))),
        });
        assert_eq!(s.current_volume(Side::Right), Some(2));
        s.toggle_volume_menu(&rec, Side::Right);
        assert_eq!(s.volume_menu.map(|m| m.highlighted), Some(2));
        // Yangilangan ro‘yxat kelganda ham (foydalanuvchi yurmagan) joriy tom belgilanib qoladi.
        let request = s.volumes.pending.expect("refresh pending");
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Volumes(vec![
                vol("Macintosh HD", "/"),
                vol("USB", "/Volumes/USB"),
            ])),
        });
        assert_eq!(s.volume_menu.map(|m| m.highlighted), Some(1));
    }

    #[test]
    fn invalid_volume_index_and_empty_list_are_harmless() {
        let rec = Recorder(std::cell::RefCell::new(Vec::new()));
        let mut s = AppState::start(&rec);
        s.toggle_volume_menu(&rec, Side::Left);
        s.step_volume_menu(true);
        let before = rec.0.borrow().len();
        s.confirm_volume_menu(&rec);
        assert_eq!(
            rec.0.borrow().len(),
            before,
            "bo‘sh ro‘yxatda navigatsiya yo‘q"
        );
        s.choose_volume(&rec, Side::Left, 42);
        assert_eq!(rec.0.borrow().len(), before);
    }

    fn rec() -> Recorder {
        Recorder(std::cell::RefCell::new(Vec::new()))
    }

    /// Faol tabning kutilayotgan so‘roviga listing javobini beradi.
    fn answer(s: &mut AppState, side: Side, id: u64, path: &str, entries: usize) {
        let request = s.panel(side).pending.expect("pending");
        let mut list = listing(id, path);
        list.entries = (0..entries as u64)
            .map(|i| test_entry(100 + i, &format!("f{i}"), EntryKind::File))
            .collect();
        s.apply(Event::Reply {
            request,
            reply: Ok(Response::Directory(list)),
        });
    }

    #[test]
    fn path_label_handles_roots_and_separators() {
        assert_eq!(path_label("/Users/me/docs"), "docs");
        assert_eq!(path_label("/Users/me/docs/"), "docs");
        assert_eq!(path_label("/"), "/");
        assert_eq!(path_label("C:\\"), "C:");
        assert_eq!(path_label("D:\\Projects\\app"), "app");
        assert_eq!(path_label("relative"), "relative");
        assert_eq!(path_label(""), "");
    }

    #[test]
    fn new_tab_copies_location_and_options_and_is_independent() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        answer(&mut s, Side::Left, 1, "/work/project", 2);
        s.panel_mut(Side::Left).options.hidden = true;
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(100));

        s.run_command(&rec, PaletteCommand::NewTab);
        let tabs = &s.panels[0];
        assert_eq!((tabs.tabs.len(), tabs.current), (2, 1));
        let fresh = s.panel(Side::Left);
        assert!(fresh.options.hidden, "ko‘rinish sozlamalari ko‘chadi");
        assert_eq!(fresh.cursor, None, "kursor va tanlov ko‘chmaydi");
        assert_eq!(
            rec.0.borrow().last(),
            Some(&Command::ReadDirectory {
                target: DirectoryTarget::Path("/work/project".into()),
                options: fresh.options.clone(),
            })
        );

        // Yangi tabda boshqa papkaga o‘tish birinchi tabga ta’sir qilmaydi.
        answer(&mut s, Side::Left, 2, "/work/project", 0);
        s.panel_mut(Side::Left)
            .navigate(&rec, DirectoryTarget::Path("/tmp".into()));
        answer(&mut s, Side::Left, 3, "/tmp", 0);
        assert_eq!(s.panels[0].tabs[0].title(), "project");
        assert_eq!(s.panels[0].tabs[0].cursor, Some(EntryId::from_raw(100)));
        assert_eq!(s.panel(Side::Left).title(), "tmp");
        assert_eq!(s.panel(Side::Right).title(), "", "B panel tegilmagan");
    }

    #[test]
    fn background_tab_receives_its_reply_and_closed_tab_reply_is_ignored() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        answer(&mut s, Side::Left, 1, "/a", 0);
        s.panels[0].open_tab(&rec);
        let background = s.panel(Side::Left).pending.expect("pending");
        s.panels[0].select(0);
        s.apply(Event::Reply {
            request: background,
            reply: Ok(Response::Directory(listing(2, "/a"))),
        });
        assert!(
            s.panels[0].tabs[1].listing.is_some(),
            "fondagi tab yangilandi"
        );

        s.panels[0].open_tab(&rec);
        let closed = s.panel(Side::Left).pending.expect("pending");
        let before = s.clone();
        s.panels[0].close_tab(1);
        let after_close = s.clone();
        s.apply(Event::Reply {
            request: closed,
            reply: Ok(Response::Directory(listing(3, "/zzz"))),
        });
        assert_eq!(s, after_close, "yopilgan tab javobi holatni o‘zgartirmaydi");
        assert_ne!(before, after_close);
    }

    #[test]
    fn closing_tabs_keeps_a_sensible_current_and_last_tab_stays() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        let tabs = &mut s.panels[1];
        tabs.open_tab(&rec);
        tabs.open_tab(&rec);
        tabs.open_tab(&rec);
        assert_eq!((tabs.tabs.len(), tabs.current), (4, 3));
        tabs.select(1);
        tabs.close_tab(0);
        assert_eq!(tabs.current, 0, "chapdagi yopilsa indeks siljiydi");
        tabs.select(2);
        tabs.close_tab(2);
        assert_eq!(
            (tabs.tabs.len(), tabs.current),
            (2, 1),
            "oxirgisi yopilsa chapga"
        );
        tabs.close_tab(5);
        assert_eq!(tabs.tabs.len(), 2, "noto‘g‘ri indeks e’tiborsiz");
        tabs.close_tab(1);
        tabs.close_tab(0);
        assert_eq!((tabs.tabs.len(), tabs.current), (1, 0), "oxirgi tab qoladi");
        tabs.cycle(true);
        assert_eq!(tabs.current, 0);
    }

    #[test]
    fn tab_limit_and_cycle_wraps() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        let tabs = &mut s.panels[0];
        for _ in 0..MAX_TABS + 3 {
            tabs.open_tab(&rec);
        }
        assert_eq!(tabs.tabs.len(), MAX_TABS);
        assert!(!tabs.can_open());
        assert_eq!(tabs.current, MAX_TABS - 1);
        tabs.cycle(true);
        assert_eq!(tabs.current, 0);
        tabs.cycle(false);
        assert_eq!(tabs.current, MAX_TABS - 1);
    }

    #[test]
    fn bookmarks_are_unique_removable_and_navigate() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        s.add_bookmark(Side::Left);
        assert!(s.bookmarks.is_empty(), "listing kelmaguncha qo‘shilmaydi");
        answer(&mut s, Side::Left, 1, "/Users/me/docs", 0);
        s.add_bookmark(Side::Left);
        s.run_command(&rec, PaletteCommand::AddBookmark);
        assert_eq!(s.bookmarks, vec!["/Users/me/docs".to_owned()]);
        assert!(s.is_bookmarked(Side::Left));
        assert!(!s.is_bookmarked(Side::Right));

        s.open_bookmark(&rec, Side::Right, 0);
        assert_eq!(s.active, Side::Right);
        assert_eq!(
            s.panel(Side::Right).history.last(),
            Some(&DirectoryTarget::Path("/Users/me/docs".into()))
        );
        let sent = rec.0.borrow().len();
        s.open_bookmark(&rec, Side::Right, 7);
        assert_eq!(rec.0.borrow().len(), sent);

        s.remove_bookmark(3);
        s.remove_bookmark(0);
        assert!(s.bookmarks.is_empty());
    }

    #[test]
    fn palette_toggles_steps_and_runs_items_on_active_panel() {
        let rec = rec();
        let mut s = AppState::start(&rec);
        answer(&mut s, Side::Left, 1, "/a", 3);
        answer(&mut s, Side::Right, 2, "/b", 2);
        s.toggle_volume_menu(&rec, Side::Left);
        s.toggle_palette();
        assert!(s.palette.is_some());
        assert!(s.volume_menu.is_none(), "palette tomlar menyusini yopadi");

        s.step_palette(false, 4);
        assert_eq!(s.palette.as_ref().map(|p| p.highlighted), Some(3));
        s.step_palette(true, 4);
        assert_eq!(s.palette.as_ref().map(|p| p.highlighted), Some(0));

        s.active = Side::Right;
        let effect = s.run_palette_item(&rec, PaletteItem::Command(PaletteCommand::SelectAll));
        assert_eq!(effect, None);
        assert!(s.palette.is_none(), "bajarilgach yopiladi");
        assert_eq!(s.panel(Side::Right).selected.len(), 2);
        assert!(s.panel(Side::Left).selected.is_empty());
        s.run_command(&rec, PaletteCommand::DeselectAll);
        assert!(s.panel(Side::Right).selected.is_empty());

        assert_eq!(
            s.run_command(&rec, PaletteCommand::PickFolder),
            Some(UiEffect::PickFolder)
        );
        s.run_command(&rec, PaletteCommand::ToggleHidden);
        assert!(s.panel(Side::Right).options.hidden);
        assert!(s.panel(Side::Right).pending.is_some());
        s.run_command(&rec, PaletteCommand::TogglePanel);
        assert_eq!(s.active, Side::Left);

        s.toggle_palette();
        s.toggle_volume_menu(&rec, Side::Right);
        assert!(s.palette.is_none(), "tomlar menyusi palette’ni yopadi");
    }

    #[test]
    fn cursor_moves_by_page_and_to_edges_and_requests_reveal() {
        let mut panel = PanelState::default();
        let mut list = listing(1, "/many");
        list.entries = (0..50)
            .map(|i| test_entry(i, &format!("f{i}"), EntryKind::File))
            .collect();
        panel.listing = Some(list);
        panel.move_cursor(PAGE_ROWS);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(0)), "avval birinchi");
        assert!(panel.reveal_cursor);
        panel.move_cursor(PAGE_ROWS);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(20)));
        panel.move_cursor(1000);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(49)));
        panel.move_cursor(-PAGE_ROWS);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(29)));
        panel.move_cursor(-1000);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(0)));
        panel.cursor_to_edge(true);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(49)));
        panel.cursor_to_edge(false);
        assert_eq!(panel.cursor, Some(EntryId::from_raw(0)));
    }
}
