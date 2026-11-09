//! Commandor egui interfeysi.
//!
//! Bu crate fayl tizimi, jarayon va tarmoqqa bevosita kirmaydi: taqiqlar `clippy.toml`da,
//! quyidagi `forbid` esa ularni `allow` bilan chetlab o‘tishni imkonsiz qiladi.
#![forbid(clippy::disallowed_methods, clippy::disallowed_types)]

mod fileops;
mod fileops_view;
mod open_with_view;
mod opening;
mod palette;
mod palette_view;
pub mod state;
mod tabs_view;
mod theme;
mod volumes_view;
mod widgets;

use commandor_broker::{AppHandle, EventReceiver};
use commandor_core::{DirectoryTarget, EntryKind, OpenWith, SortDirection, SortField};
use commandor_i18n::I18n;
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Key, KeyboardShortcut, Layout,
    Margin, Modifiers, Rect, RichText, Sense, Shadow, Stroke, pos2, vec2,
};
use fileops_view::{OpAction, name_window, plan_window};
use open_with_view::{ConfirmAction, OpenWithAction, confirm_window, open_with_window};
use palette_view::{PaletteAction, palette_window};
use state::{
    AppState, BrokerStatus, NoticeKind, OpDialog, PAGE_ROWS, PaletteCommand, PaletteItem,
    PanelState, Side, UiEffect,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tabs_view::{StripAction, StripModel, TabInfo, tab_strip};
use theme::{Palette, RADIUS_CARD, ROW_HEIGHT};
use volumes_view::{VolumeAction, volume_bar, volume_menu};
use widgets::{
    fkey_chip, icon_button, key_badge, paint_check, paint_file, paint_folder, paint_sort_arrow,
    pill, primary_button, status_pill, toggle_chip, truncated,
};

/// Native OS papka tanlash dialogi abstraksiyasi.
/// UI disk yoki OS kutubxonalariga to‘g‘ridan-to‘g‘ri kirmaydi; `commandor-app` binary buni ta’minlaydi.
pub trait FolderPicker: Send + Sync + 'static {
    fn pick_folder(&self) -> Option<String>;
}

/// Avtomatik smoke tekshiruvi: broker tayyor bo‘lgach N kadrdan so‘ng oynani yopadi.
#[derive(Debug, Clone)]
pub struct SmokeProbe {
    pub frames_after_ready: u32,
    pub passed: Arc<AtomicBool>,
}

pub struct CommandorApp {
    handle: AppHandle,
    events: EventReceiver,
    i18n: I18n,
    state: AppState,
    smoke: Option<(SmokeProbe, u32)>,
    smoke_started: std::time::Instant,
    picker_service: Option<Arc<dyn FolderPicker>>,
    picker_rx: Option<std::sync::mpsc::Receiver<Option<String>>>,
    theme_installed: bool,
}

/// Panel ichidagi qator balandligi (yo‘l va filtr maydonlari).
const CONTROL_ROW: f32 = 30.0;
/// Jadval ichki chap/o‘ng chekinishi.
const TABLE_PAD: f32 = 10.0;
/// Hajm va sana ustunlari orasidagi bo‘shliq.
const COLUMN_GAP: f32 = 16.0;
/// Sana ustuni ko‘rsatiladigan minimal jadval kengligi.
const DATE_MIN_WIDTH: f32 = 430.0;

/// Jadval ustunlari geometriyasi.
#[derive(Debug, Clone, Copy)]
struct Columns {
    name_w: f32,
    size_w: f32,
    date_w: f32,
}

impl Columns {
    fn for_width(width: f32) -> Self {
        let size_w = 86.0;
        let date_w = if width >= DATE_MIN_WIDTH { 132.0 } else { 0.0 };
        let date_space = if date_w > 0.0 {
            COLUMN_GAP + date_w
        } else {
            0.0
        };
        let name_w = (width - TABLE_PAD * 2.0 - size_w - date_space).max(80.0);
        Self {
            name_w,
            size_w,
            date_w,
        }
    }

    fn name_x(self, rect: Rect) -> f32 {
        rect.min.x + TABLE_PAD
    }

    fn size_right(self, rect: Rect) -> f32 {
        self.name_x(rect) + self.name_w + self.size_w
    }

    fn date_x(self, rect: Rect) -> f32 {
        self.size_right(rect) + COLUMN_GAP
    }
}

impl CommandorApp {
    pub fn new(
        handle: AppHandle,
        events: EventReceiver,
        i18n: I18n,
        smoke: Option<SmokeProbe>,
        folder_picker: Option<Arc<dyn FolderPicker>>,
    ) -> Self {
        let state = AppState::start(&handle);
        Self {
            handle,
            events,
            i18n,
            state,
            smoke: smoke.map(|p| (p, 0)),
            smoke_started: std::time::Instant::now(),
            picker_service: folder_picker,
            picker_rx: None,
            theme_installed: false,
        }
    }

    fn status_text(&self) -> String {
        match &self.state.broker {
            BrokerStatus::Connecting { .. } => self.i18n.t("status-broker-connecting"),
            BrokerStatus::Ready(info) => self.i18n.t_args(
                "status-broker-ready",
                &[
                    ("version", &info.version),
                    ("os", &info.os),
                    ("arch", &info.arch),
                ],
            ),
            BrokerStatus::Failed(err) => self.i18n.t_args(
                "status-broker-error",
                &[("error", &self.i18n.t(err.message_key()))],
            ),
            BrokerStatus::Stopped => self.i18n.t("status-broker-stopped"),
        }
    }

    fn status_color(&self, p: &Palette) -> Color32 {
        match &self.state.broker {
            BrokerStatus::Connecting { .. } => p.warning,
            BrokerStatus::Ready(_) => p.success,
            BrokerStatus::Failed(_) => p.danger,
            BrokerStatus::Stopped => p.text_faint,
        }
    }

    fn file_panel(&mut self, ui: &mut egui::Ui, side: Side) {
        let p = Palette::of(ui);
        let active = self.state.active == side;
        let title = self.i18n.t(match side {
            Side::Left => "panel-left",
            Side::Right => "panel-right",
        });
        let current_volume = self.state.current_volume(side);
        let menu = self.state.volume_menu.filter(|menu| menu.side == side);
        let tabs = &self.state.panels[side.index()];
        let strip = StripModel {
            tabs: tabs
                .tabs
                .iter()
                .map(|tab| TabInfo {
                    title: match tab.title() {
                        "" => "…".to_owned(),
                        title => title.to_owned(),
                    },
                    tooltip: tab
                        .current_path()
                        .unwrap_or(tab.path_input.as_str())
                        .to_owned(),
                    loading: tab.pending.is_some(),
                })
                .collect(),
            current: tabs.current,
            can_open: tabs.can_open(),
            bookmarks: &self.state.bookmarks,
            bookmarked: self.state.is_bookmarked(side),
            can_bookmark: tabs.active().current_path().is_some(),
        };
        let mut strip_action: Option<StripAction> = None;
        let volumes = &self.state.volumes;
        let panel = self.state.panels[side.index()].active_mut();
        let handle = &self.handle;
        let i18n = &self.i18n;
        let mut volume_action: Option<VolumeAction> = None;
        let mut row_action: Option<RowAction> = None;
        let mut anchor = Rect::NOTHING;
        let (stroke, shadow) = if active {
            (
                Stroke::new(1.5, p.accent),
                Shadow {
                    offset: [0, 6],
                    blur: 22,
                    spread: 0,
                    color: p.glow,
                },
            )
        } else {
            (
                Stroke::new(1.0, p.border),
                Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 0,
                    color: p.shadow,
                },
            )
        };
        let frame = egui::Frame::new()
            .fill(p.surface)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(RADIUS_CARD))
            .inner_margin(Margin::same(12))
            .shadow(shadow)
            .show(ui, |ui| {
                ui.set_min_height(ui.available_height());
                ui.set_width(ui.available_width());
                ui.push_id(side.index(), |ui| {
                    panel_header(ui, &p, panel, handle, i18n, &title, active);
                    tab_strip(ui, &p, i18n, &strip, &mut strip_action);
                    ui.add_space(2.0);
                    anchor = volume_bar(
                        ui,
                        &p,
                        i18n,
                        volumes,
                        current_volume,
                        menu.is_some(),
                        side,
                        &mut volume_action,
                    );
                    panel_inputs(ui, panel, handle, i18n);
                    ui.add_space(2.0);
                    // Har tab o‘z scroll holatiga ega bo‘lishi uchun barqaror ID.
                    ui.push_id(("tab", panel.tab_id), |ui| {
                        row_action = panel_body(ui, &p, panel, handle, i18n, active);
                    });
                });
            });
        if let Some(menu) = menu
            && anchor.is_positive()
        {
            volume_menu(
                ui.ctx(),
                &p,
                i18n,
                volumes,
                menu,
                current_volume,
                anchor,
                &title,
                &mut volume_action,
            );
        }
        if ui.input(|i| i.pointer.any_pressed()) && frame.response.contains_pointer() {
            self.state.active = side;
        }
        if let Some(action) = row_action {
            self.state.active = side;
            match action {
                RowAction::Open => self.state.open_cursor(&self.handle),
                RowAction::OpenWith => self.state.open_with_cursor(&self.handle),
            }
        }
        match volume_action {
            Some(VolumeAction::Toggle) => self.state.toggle_volume_menu(&self.handle, side),
            Some(VolumeAction::Choose(index)) => {
                self.state.choose_volume(&self.handle, side, index);
            }
            Some(VolumeAction::Highlight(index)) => self.state.highlight_volume(index),
            Some(VolumeAction::Close) => self.state.close_volume_menu(),
            None => {}
        }
        if let Some(action) = strip_action {
            self.state.active = side;
            let tabs = &mut self.state.panels[side.index()];
            match action {
                StripAction::Select(index) => tabs.select(index),
                StripAction::Close(index) => tabs.close_tab(index),
                StripAction::New => tabs.open_tab(&self.handle),
                StripAction::AddBookmark => self.state.add_bookmark(side),
                StripAction::OpenBookmark(index) => {
                    self.state.open_bookmark(&self.handle, side, index);
                }
                StripAction::RemoveBookmark(index) => self.state.remove_bookmark(index),
            }
        }
    }

    /// Tizim papka dialogini alohida threadda ochadi (UI bloklanmaydi).
    fn start_pick_folder(&mut self) {
        let Some(picker) = &self.picker_service else {
            return;
        };
        if self.picker_rx.is_some() {
            return;
        }
        let picker = Arc::clone(picker);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let picked = picker.pick_folder();
            let _ = tx.send(picked);
        });
        self.picker_rx = Some(rx);
    }

    fn apply_effect(&mut self, effect: Option<UiEffect>) {
        match effect {
            Some(UiEffect::PickFolder) => self.start_pick_folder(),
            None => {}
        }
    }

    fn ops_windows(&mut self, ctx: &egui::Context, p: &Palette) {
        let mut action = None;
        match self.state.ops.dialog.as_mut() {
            Some(OpDialog::Name(dialog)) => {
                let directory = self.state.panels[dialog.side.index()]
                    .active()
                    .current_path()
                    .unwrap_or_default()
                    .to_owned();
                name_window(ctx, p, &self.i18n, dialog, &directory, &mut action);
            }
            Some(OpDialog::Confirm(dialog)) => {
                plan_window(ctx, p, &self.i18n, dialog, &mut action);
            }
            None => return,
        }
        match action {
            Some(OpAction::Submit) => match &self.state.ops.dialog {
                Some(OpDialog::Name(_)) => self.state.submit_name(&self.handle),
                Some(OpDialog::Confirm(_)) => self.state.confirm_plan(&self.handle),
                None => {}
            },
            Some(OpAction::Cancel) => self.state.cancel_op(&self.handle),
            Some(OpAction::Policy(policy)) => self.state.choose_policy(&self.handle, policy),
            None => {}
        }
        if !self.state.ops.is_modal() {
            surrender_focus(ctx);
        }
    }

    fn opening_windows(&mut self, ctx: &egui::Context, p: &Palette) {
        if let Some(file) = self.state.opening.confirm.clone() {
            let mut action = None;
            confirm_window(ctx, p, &self.i18n, &file, &mut action);
            if let Some(action) = action {
                self.state
                    .answer_confirm(&self.handle, action == ConfirmAction::Run);
            }
            return;
        }
        let mut action = None;
        if let Some(menu) = self.state.opening.menu.as_mut() {
            open_with_window(ctx, p, &self.i18n, menu, &mut action);
        }
        match action {
            Some(OpenWithAction::Choose(id)) => {
                self.state.choose_app(&self.handle, OpenWith::App(id));
                surrender_focus(ctx);
            }
            Some(OpenWithAction::SystemChooser) => {
                self.state.choose_app(&self.handle, OpenWith::SystemChooser);
                surrender_focus(ctx);
            }
            Some(OpenWithAction::Highlight(index)) => {
                if let Some(menu) = self.state.opening.menu.as_mut() {
                    menu.highlighted = index;
                }
            }
            Some(OpenWithAction::ToggleAll) => self.state.toggle_show_all(),
            Some(OpenWithAction::Close) => {
                self.state.close_open_with();
                surrender_focus(ctx);
            }
            None => {}
        }
    }

    /// Joriy so‘rov bo‘yicha buyruqlar paneli qatorlari.
    fn palette_items(&self) -> Vec<PaletteItem> {
        let query = self
            .state
            .palette
            .as_ref()
            .map_or("", |palette| palette.query.as_str());
        palette::filter(query, &self.state.bookmarks, |command| {
            self.i18n.t(command.label_key())
        })
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let p = Palette::of(ui);
        ui.horizontal(|ui| {
            logo(ui, &p);
            ui.label(
                RichText::new(self.i18n.t("app-title"))
                    .size(17.0)
                    .color(p.text),
            );
            ui.add_space(10.0);
            if self.picker_service.is_some() {
                let picking = self.picker_rx.is_some();
                let label = self.i18n.t("nav-pick-folder");
                if ui
                    .add_enabled(
                        !picking,
                        egui::Button::new(RichText::new(label).color(p.text))
                            .min_size(vec2(0.0, 28.0)),
                    )
                    .clicked()
                {
                    self.start_pick_folder();
                }
            }
            let palette_label = format!(
                "{}   {}",
                self.i18n.t("palette-title"),
                ui.ctx().format_shortcut(&PALETTE_SHORTCUT)
            );
            if ui
                .add(
                    egui::Button::new(RichText::new(palette_label).color(p.text_weak))
                        .min_size(vec2(0.0, 28.0)),
                )
                .clicked()
            {
                self.state.toggle_palette();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                status_pill(ui, self.status_text(), self.status_color(&p));
            });
        });
    }

    /// Pastki panel; F1/F2 chipi bosilsa shu panel tomoni qaytadi.
    fn bottom_bar(&self, ui: &mut egui::Ui) -> Option<BottomAction> {
        let p = Palette::of(ui);
        let unavailable = self.i18n.t("fkey-unavailable");
        let volumes_tip = self.i18n.t("volumes-open");
        let mut toggled = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            for (key, side) in [
                ("fkey-volumes-left", Side::Left),
                ("fkey-volumes-right", Side::Right),
            ] {
                if fkey_chip(ui, &self.i18n.t(key), true, &volumes_tip).clicked() {
                    toggled = Some(BottomAction::Volumes(side));
                }
            }
            for key in ["fkey-view", "fkey-edit"] {
                let _ = fkey_chip(ui, &self.i18n.t(key), false, &unavailable);
            }
            for (key, tip, action) in [
                ("fkey-copy", "cmd-copy", BottomAction::Copy),
                ("fkey-move", "cmd-move", BottomAction::Move),
                ("fkey-mkdir", "cmd-new-folder", BottomAction::NewFolder),
                ("fkey-trash", "cmd-trash", BottomAction::Trash),
            ] {
                if fkey_chip(ui, &self.i18n.t(key), true, &self.i18n.t(tip)).clicked() {
                    toggled = Some(action);
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(job) = self.state.ops.jobs.first() {
                    if let Some(action) =
                        job_strip(ui, &p, &self.i18n, job, self.state.ops.jobs.len())
                    {
                        toggled = Some(action);
                    }
                    return;
                }
                if let Some(notice) = &self.state.opening.notice {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if icon_button(ui, "✖", &self.i18n.t("notice-dismiss"), true) {
                        toggled = Some(BottomAction::DismissNotice);
                    }
                    let error = error_text(&self.i18n, &notice.error);
                    let text = match notice.kind {
                        NoticeKind::Open => self.i18n.t_args(
                            "notice-open-failed",
                            &[("name", &notice.file), ("error", &error)],
                        ),
                        NoticeKind::Cancelled { done } => self
                            .i18n
                            .t_args("notice-op-cancelled", &[("done", &done.to_string())]),
                        NoticeKind::Operation { failed, total } => self.i18n.t_args(
                            "notice-op-failed",
                            &[
                                ("name", &quoted(&notice.file)),
                                ("failed", &failed.to_string()),
                                ("total", &total.to_string()),
                                ("error", &error),
                            ],
                        ),
                    };
                    let galley = truncated(
                        ui,
                        text,
                        FontId::proportional(12.5),
                        p.danger,
                        ui.available_width().min(520.0),
                    );
                    egui::Frame::new()
                        .fill(p.danger_soft)
                        .corner_radius(CornerRadius::same(7))
                        .inner_margin(Margin::symmetric(10, 4))
                        .show(ui, |ui| ui.label(galley));
                    return;
                }
                let hint = self.i18n.t("hint-tab");
                ui.spacing_mut().item_spacing.x = 6.0;
                // "Tab — ..." ko‘rinishidagi maslahatni klaviatura belgisi bilan ko‘rsatamiz.
                if let Some((key, text)) = hint.split_once(" — ") {
                    ui.label(RichText::new(text).size(12.5).color(p.text_faint));
                    key_badge(ui, key, true);
                } else {
                    ui.label(RichText::new(hint).size(12.5).color(p.text_faint));
                }
            });
        });
        toggled
    }
}

/// Xato matni: `reason` tafsiloti bo‘lsa aniqroq `error-reason-*` matni, aks holda kod matni.
pub(crate) fn error_text(i18n: &I18n, error: &commandor_core::AppError) -> String {
    if let Some(commandor_core::DetailValue::Text(reason)) = error.details.get("reason") {
        let key = format!("error-reason-{}", reason.replace('_', "-"));
        if i18n.has(&key) {
            return i18n.t(&key);
        }
    }
    i18n.t(error.message_key())
}

/// Pastki paneldagi fon vazifasi: progress chizig‘i, matn va bekor qilish tugmasi.
fn job_strip(
    ui: &mut egui::Ui,
    p: &Palette,
    i18n: &I18n,
    job: &state::JobView,
    count: usize,
) -> Option<BottomAction> {
    ui.spacing_mut().item_spacing.x = 8.0;
    let mut action = None;
    let tip = i18n.t("job-cancel");
    if !job.cancelling && icon_button(ui, "✖", &tip, true) {
        action = Some(BottomAction::CancelJob(job.job));
    }
    let key = match (job.cancelling, job.action) {
        (true, _) => "job-cancelling",
        (false, commandor_core::PlanAction::Move) => "job-moving",
        _ => "job-copying",
    };
    let fraction = job.fraction();
    let mut text = i18n.t_args(
        key,
        &[
            ("name", &job.first),
            (
                "more",
                &match job.count.saturating_sub(1) {
                    0 => String::new(),
                    more => format!(" (+{more})"),
                },
            ),
        ],
    );
    if let Some(progress) = &job.progress {
        text.push_str(" · ");
        text.push_str(&format_bytes(progress.done_bytes));
        if let Some(totals) = progress.totals {
            text.push_str(" / ");
            text.push_str(&format_bytes(totals.bytes));
        }
    }
    if count > 1 {
        text.push_str(&i18n.t_args("job-queued", &[("count", &(count - 1).to_string())]));
    }
    let (bar, _) = ui.allocate_exact_size(vec2(120.0, 6.0), Sense::hover());
    ui.painter()
        .rect_filled(bar, CornerRadius::same(3), p.raised);
    let filled = fraction.unwrap_or(0.0);
    if filled > 0.0 {
        let mut fill = bar;
        fill.set_width(bar.width() * filled);
        ui.painter()
            .rect_filled(fill, CornerRadius::same(3), p.accent);
    }
    if let Some(fraction) = fraction {
        ui.label(
            RichText::new(format!("{:.0}%", fraction * 100.0))
                .size(12.5)
                .color(p.text),
        );
    } else {
        ui.add(egui::Spinner::new().size(12.0));
    }
    let galley = truncated(
        ui,
        text,
        FontId::proportional(12.5),
        p.text_weak,
        ui.available_width().min(480.0),
    );
    ui.label(galley);
    action
}

/// Bo‘sh bo‘lmagan nomni «» ichiga oladi (xabar matnlari uchun).
fn quoted(name: &str) -> String {
    if name.is_empty() {
        String::new()
    } else {
        format!(" «{name}»")
    }
}

/// Pastki paneldagi amal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomAction {
    Volumes(Side),
    DismissNotice,
    NewFolder,
    Trash,
    Copy,
    Move,
    CancelJob(commandor_core::JobId),
}

/// Ikki ustunli "panel" belgisidan iborat logotip.
fn logo(ui: &mut egui::Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(8), p.accent);
    let inner = rect.shrink(7.0);
    let half = (inner.width() - 3.0) / 2.0;
    let left = Rect::from_min_size(inner.min, vec2(half, inner.height()));
    let right = Rect::from_min_size(pos2(left.max.x + 3.0, inner.min.y), left.size());
    painter.rect_filled(left, CornerRadius::same(2), p.on_accent);
    painter.rect_filled(
        right,
        CornerRadius::same(2),
        p.on_accent.gamma_multiply(0.55),
    );
}

/// Panel sarlavhasi va navigatsiya tugmalari.
fn panel_header(
    ui: &mut egui::Ui,
    p: &Palette,
    panel: &mut PanelState,
    handle: &AppHandle,
    i18n: &I18n,
    title: &str,
    active: bool,
) {
    let idle = panel.pending.is_none();
    ui.horizontal(|ui| {
        let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        let dot_color = if active { p.accent } else { p.text_faint };
        if active {
            ui.painter()
                .circle_filled(dot.center(), 6.0, p.accent.gamma_multiply(0.25));
        }
        ui.painter().circle_filled(dot.center(), 4.0, dot_color);
        let title_color = if active { p.text } else { p.text_weak };
        ui.label(RichText::new(title).size(15.0).color(title_color));
        if active {
            pill(ui, i18n.t("panel-active"), p.accent, p.accent_soft);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if icon_button(ui, "🔄", &i18n.t("nav-refresh"), idle) {
                panel.refresh(handle);
            }
            let can_up = idle && panel.listing.as_ref().is_some_and(|l| l.location.can_go_up);
            if icon_button(ui, "⬆", &i18n.t("nav-up"), can_up) {
                panel.parent(handle);
            }
            if icon_button(ui, "🏠", &i18n.t("nav-home"), idle) {
                panel.navigate(handle, DirectoryTarget::Home);
            }
            ui.add_space(8.0);
            if icon_button(
                ui,
                "➡",
                &i18n.t("nav-forward"),
                idle && panel.can_go_forward(),
            ) {
                panel.go_forward(handle);
            }
            if icon_button(ui, "⬅", &i18n.t("nav-back"), idle && panel.can_go_back()) {
                panel.go_back(handle);
            }
        });
    });
}

/// O‘ngdan chapga joylashuvli qator: avval tugmalar, qolgan kenglik matn maydoniga.
fn control_row<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), CONTROL_ROW),
        Layout::right_to_left(Align::Center),
        add,
    )
    .inner
}

/// Yo‘l, filtr va yashirin fayllar boshqaruvlari.
fn panel_inputs(ui: &mut egui::Ui, panel: &mut PanelState, handle: &AppHandle, i18n: &I18n) {
    ui.add_enabled_ui(panel.pending.is_none(), |ui| {
        control_row(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let open = primary_button(ui, &i18n.t("nav-open")).clicked();
            let path = ui.add(
                egui::TextEdit::singleline(&mut panel.path_input)
                    .hint_text(i18n.t("nav-path"))
                    .margin(vec2(10.0, 6.0))
                    .desired_width(ui.available_width()),
            );
            let enter = path.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if open || enter {
                path.surrender_focus();
                panel.navigate(handle, DirectoryTarget::Path(panel.path_input.clone()));
            }
        });
        control_row(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if toggle_chip(ui, panel.options.hidden, "👁", &i18n.t("list-hidden")).clicked() {
                panel.options.hidden = !panel.options.hidden;
                panel.refresh(handle);
            }
            let apply = ui
                .add(egui::Button::new(i18n.t("list-apply-filter")).min_size(vec2(0.0, 28.0)))
                .clicked();
            let filter = ui.add(
                egui::TextEdit::singleline(&mut panel.options.filter)
                    .hint_text(format!("🔍  {}", i18n.t("list-filter")))
                    .char_limit(256)
                    .margin(vec2(10.0, 6.0))
                    .desired_width(ui.available_width()),
            );
            let enter = filter.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if apply || enter {
                filter.surrender_focus();
                panel.refresh(handle);
            }
        });
    });
}

/// Markazga joylashgan holat bloki (yuklanmoqda, bo‘sh, xato).
fn centered_state<R>(ui: &mut egui::Ui, height: f32, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        Layout::top_down(Align::Center),
        |ui| {
            ui.set_min_height(height);
            ui.add_space((height * 0.3).min(120.0));
            add(ui)
        },
    )
    .inner
}

/// Jadval yoki holat bloki va pastki statistika qatori.
fn panel_body(
    ui: &mut egui::Ui,
    p: &Palette,
    panel: &mut PanelState,
    handle: &AppHandle,
    i18n: &I18n,
    active: bool,
) -> Option<RowAction> {
    let footer_h = 22.0;
    if panel.pending.is_some() {
        let height = ui.available_height();
        centered_state(ui, height, |ui| {
            ui.add(egui::Spinner::new().size(22.0).color(p.accent));
            ui.label(RichText::new(i18n.t("list-loading")).color(p.text_weak));
        });
        return None;
    }
    if let Some(error) = &panel.error {
        let message = i18n.t(error.message_key());
        let height = ui.available_height();
        let retry = centered_state(ui, height, |ui| {
            ui.label(RichText::new("⚠").size(30.0).color(p.danger));
            egui::Frame::new()
                .fill(p.danger_soft)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.label(RichText::new(message).color(p.danger));
                });
            ui.add_space(4.0);
            primary_button(ui, &i18n.t("list-retry")).clicked()
        });
        if retry {
            panel.refresh(handle);
        }
        return None;
    }
    let Some(listing) = &panel.listing else {
        return None;
    };

    if listing.truncated {
        egui::Frame::new()
            .fill(p.warning_soft)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!("⚠  {}", i18n.t("list-truncated")))
                        .size(12.5)
                        .color(p.warning),
                );
            });
    }
    if listing.skipped_entries > 0 {
        ui.label(
            RichText::new(i18n.t_args(
                "list-skipped",
                &[("count", &listing.skipped_entries.to_string())],
            ))
            .size(12.0)
            .color(p.text_faint),
        );
    }

    let width = ui.available_width();
    let cols = Columns::for_width(width);
    let mut sort_toggle: Option<SortField> = None;
    let mut toggle_entry: Option<commandor_core::EntryId> = None;
    let mut set_cursor: Option<commandor_core::EntryId> = None;
    let mut row_action: Option<RowAction> = None;

    // Jadval sarlavhasi: ustun nomlari saralash tugmasi vazifasini bajaradi.
    let (header, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    ui.painter()
        .rect_filled(header, CornerRadius::same(6), p.surface_alt);
    let header_cols = [
        (
            SortField::Name,
            "list-col-name",
            cols.name_x(header),
            cols.name_w,
            false,
        ),
        (
            SortField::Size,
            "list-col-size",
            cols.size_right(header) - cols.size_w,
            cols.size_w,
            true,
        ),
        (
            SortField::Modified,
            "list-col-modified",
            cols.date_x(header),
            cols.date_w,
            false,
        ),
    ];
    for (index, (field, key, x, w, right_aligned)) in header_cols.into_iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        let rect = Rect::from_min_size(pos2(x, header.min.y), vec2(w, header.height()));
        let response = ui
            .interact(rect, ui.id().with(("sort", index)), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let sorted = panel.options.sort.field == field;
        let color = if sorted {
            p.accent
        } else if response.hovered() {
            p.text
        } else {
            p.text_faint
        };
        let label = i18n.t(key).to_uppercase();
        let (pos, align) = if right_aligned {
            (pos2(rect.max.x, rect.center().y), Align2::RIGHT_CENTER)
        } else {
            (pos2(rect.min.x, rect.center().y), Align2::LEFT_CENTER)
        };
        let text_rect = ui
            .painter()
            .text(pos, align, label, FontId::proportional(11.0), color);
        if sorted {
            // O‘ngga tekislangan ustunda strelka matndan chapda, aks holda o‘ngda.
            let x = if right_aligned {
                text_rect.min.x - 9.0
            } else {
                text_rect.max.x + 9.0
            };
            paint_sort_arrow(
                ui.painter(),
                pos2(x, rect.center().y),
                panel.options.sort.direction == SortDirection::Ascending,
                color,
            );
        }
        if response.clicked() {
            sort_toggle = Some(field);
        }
    }
    ui.add_space(-4.0);

    let list_h =
        (ui.available_height() - footer_h - ui.spacing().item_spacing.y).max(ROW_HEIGHT * 2.0);
    if listing.entries.is_empty() {
        centered_state(ui, list_h, |ui| {
            let (icon, _) = ui.allocate_exact_size(vec2(40.0, 34.0), Sense::hover());
            paint_folder(ui.painter(), icon.center(), 2.2, p.text_faint);
            ui.label(RichText::new(i18n.t("list-empty")).color(p.text_faint));
        });
    } else {
        // Klaviatura bilan siljigan kursor ko‘rinmas joyda bo‘lsa, ro‘yxat unga aylantiriladi.
        let reveal = std::mem::take(&mut panel.reveal_cursor)
            .then(|| {
                listing
                    .entries
                    .iter()
                    .position(|entry| Some(entry.id) == panel.cursor)
            })
            .flatten();
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical()
                .id_salt("files")
                .auto_shrink([false, false])
                .max_height(list_h)
                .show_rows(ui, ROW_HEIGHT, listing.entries.len(), |ui, range| {
                    if let Some(target) = reveal {
                        // `show_rows` faqat ko‘rinadigan qatorlarni joylaydi: maqsad qator joyini hisoblaymiz.
                        let top =
                            ui.cursor().min.y + (target as f32 - range.start as f32) * ROW_HEIGHT;
                        let rect = Rect::from_min_size(
                            pos2(ui.cursor().min.x, top),
                            vec2(1.0, ROW_HEIGHT),
                        );
                        ui.scroll_to_rect(rect, None);
                    }
                    for index in range {
                        let entry = &listing.entries[index];
                        let (rect, row) = ui.allocate_exact_size(
                            vec2(ui.available_width(), ROW_HEIGHT),
                            Sense::click(),
                        );
                        let is_cursor = panel.cursor == Some(entry.id);
                        let is_selected = panel.selected.contains(&entry.id);
                        if ui.is_rect_visible(rect) {
                            paint_row(
                                ui,
                                p,
                                rect,
                                cols,
                                entry,
                                RowState {
                                    stripe: index % 2 == 1,
                                    hovered: row.hovered(),
                                    cursor: is_cursor,
                                    selected: is_selected,
                                    panel_active: active,
                                },
                                i18n,
                            );
                        }
                        if row.clicked() {
                            if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                                toggle_entry = Some(entry.id);
                            }
                            set_cursor = Some(entry.id);
                        }
                        if row.double_clicked() {
                            set_cursor = Some(entry.id);
                            row_action = Some(RowAction::Open);
                        }
                        if row.secondary_clicked() {
                            set_cursor = Some(entry.id);
                        }
                        let is_file = entry.kind != EntryKind::Directory;
                        row.context_menu(|ui| {
                            if ui.button(i18n.t("menu-open")).clicked() {
                                row_action = Some(RowAction::Open);
                            }
                            if is_file && ui.button(i18n.t("menu-open-with")).clicked() {
                                row_action = Some(RowAction::OpenWith);
                            }
                        });
                        if entry.kind == EntryKind::Symlink {
                            row.on_hover_text(i18n.t("list-symlink"));
                        }
                    }
                });
        });
    }

    // Pastki statistika qatori.
    ui.horizontal(|ui| {
        ui.set_height(footer_h);
        ui.label(
            RichText::new(i18n.t_args(
                "list-count",
                &[("count", &listing.entries.len().to_string())],
            ))
            .size(12.0)
            .color(p.text_faint),
        );
        let (sel_count, sel_bytes) = panel.selected_stats();
        if sel_count > 0 {
            pill(
                ui,
                i18n.t_args(
                    "list-selected",
                    &[
                        ("count", &sel_count.to_string()),
                        ("size", &format_bytes(sel_bytes)),
                    ],
                ),
                p.warning,
                p.warning_soft,
            );
        }
    });

    if let Some(field) = sort_toggle {
        panel.toggle_sort(handle, field);
    }
    if let Some(id) = toggle_entry {
        panel.toggle_selected(id);
    }
    if let Some(id) = set_cursor {
        panel.cursor = Some(id);
    }
    row_action
}

/// Jadval qatoridan kelgan, `AppState` darajasida bajariladigan amal (kursor oldindan qo‘yiladi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowAction {
    Open,
    OpenWith,
}

#[derive(Debug, Clone, Copy)]
struct RowState {
    stripe: bool,
    hovered: bool,
    cursor: bool,
    selected: bool,
    panel_active: bool,
}

/// Bitta fayl qatorini chizadi: fon, kursor chizig‘i, ikonka, nom, hajm, sana.
fn paint_row(
    ui: &egui::Ui,
    p: &Palette,
    rect: Rect,
    cols: Columns,
    entry: &commandor_core::FileEntry,
    state: RowState,
    i18n: &I18n,
) {
    let painter = ui.painter();
    let radius = CornerRadius::same(6);
    if state.selected {
        painter.rect_filled(rect, radius, p.warning_soft);
    } else if state.stripe {
        painter.rect_filled(rect, radius, p.row_stripe);
    }
    if state.hovered {
        painter.rect_filled(rect, radius, p.row_hover);
    }
    if state.cursor {
        let fill = if state.panel_active {
            p.accent_soft
        } else {
            p.row_cursor_idle
        };
        painter.rect_filled(rect, radius, fill);
        if state.panel_active {
            let bar = Rect::from_min_size(
                pos2(rect.min.x + 1.0, rect.min.y + 6.0),
                vec2(3.0, rect.height() - 12.0),
            );
            painter.rect_filled(bar, CornerRadius::same(2), p.accent);
        }
    }

    let cy = rect.center().y;
    let name_x = cols.name_x(rect);
    let icon_center = pos2(name_x + 8.0, cy);
    if state.selected {
        paint_check(painter, icon_center, p.warning, p.surface);
    } else {
        match entry.kind {
            EntryKind::Directory => paint_folder(painter, icon_center, 1.0, p.folder),
            EntryKind::Symlink => paint_file(painter, icon_center, p.link, true),
            _ => paint_file(painter, icon_center, p.file, false),
        }
    }

    let name_color = if state.selected { p.warning } else { p.text };
    let name = entry.name.clone();
    let galley = truncated(
        ui,
        name,
        FontId::proportional(14.0),
        name_color,
        cols.name_w - 32.0,
    );
    painter.galley(
        pos2(name_x + 24.0, cy - galley.size().y / 2.0),
        galley,
        name_color,
    );

    let (size_text, size_color) = match entry.size_bytes {
        Some(bytes) => (format_bytes(bytes), p.text_weak),
        None if entry.kind == EntryKind::Directory => (i18n.t("list-directory"), p.text_faint),
        None => (String::new(), p.text_faint),
    };
    painter.text(
        pos2(cols.size_right(rect), cy),
        Align2::RIGHT_CENTER,
        size_text,
        FontId::proportional(12.5),
        size_color,
    );

    if cols.date_w > 0.0
        && let Some(secs) = entry.modified_unix_seconds
    {
        painter.text(
            pos2(cols.date_x(rect), cy),
            Align2::LEFT_CENTER,
            format_timestamp(secs),
            FontId::monospace(12.0),
            p.text_faint,
        );
    }
}

/// Buyruqlar panelini ochish/yopish: Ctrl+Shift+P (macOS’da Cmd+Shift+P).
const PALETTE_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::P);

/// Buyruq yorlig‘i. Panel ro‘yxati va klaviatura ishlovi aynan shu jadvaldan foydalanadi.
fn command_shortcut(command: PaletteCommand) -> Option<KeyboardShortcut> {
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND.plus(Modifiers::SHIFT);
    let (modifiers, key) = match command {
        PaletteCommand::GoHome => (cmd_shift, Key::H),
        PaletteCommand::GoUp => (Modifiers::NONE, Key::Backspace),
        PaletteCommand::GoBack => (Modifiers::ALT, Key::ArrowLeft),
        PaletteCommand::GoForward => (Modifiers::ALT, Key::ArrowRight),
        PaletteCommand::Refresh => (cmd, Key::R),
        PaletteCommand::NewTab => (cmd, Key::T),
        PaletteCommand::CloseTab => (cmd, Key::W),
        PaletteCommand::TogglePanel => (Modifiers::NONE, Key::Tab),
        PaletteCommand::ToggleHidden => (cmd_shift, Key::Period),
        PaletteCommand::SortName => (cmd, Key::Num1),
        PaletteCommand::SortSize => (cmd, Key::Num2),
        PaletteCommand::SortModified => (cmd, Key::Num3),
        PaletteCommand::SelectAll => (cmd, Key::A),
        PaletteCommand::DeselectAll => (cmd_shift, Key::A),
        PaletteCommand::PickFolder => (cmd, Key::O),
        PaletteCommand::AddBookmark => (cmd, Key::D),
        PaletteCommand::Open => (Modifiers::NONE, Key::Enter),
        PaletteCommand::OpenWith => (Modifiers::SHIFT, Key::Enter),
        PaletteCommand::NewFolder => (Modifiers::NONE, Key::F7),
        PaletteCommand::Rename => (Modifiers::SHIFT, Key::F6),
        PaletteCommand::Trash => (Modifiers::NONE, Key::F8),
        PaletteCommand::Copy => (Modifiers::NONE, Key::F5),
        PaletteCommand::Move => (Modifiers::NONE, Key::F6),
    };
    Some(KeyboardShortcut::new(modifiers, key))
}

fn surrender_focus(ctx: &egui::Context) {
    ctx.memory_mut(|m| {
        if let Some(id) = m.focused() {
            m.surrender_focus(id);
        }
    });
}

impl CommandorApp {
    /// Panel ochiq: strelka, Enter va Esc unga tegishli; qolgan yorliqlar to‘xtatiladi.
    fn palette_keys(&mut self, ctx: &egui::Context) {
        let items = self.palette_items();
        for (key, forward) in [(Key::ArrowDown, true), (Key::ArrowUp, false)] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                self.state.step_palette(forward, items.len());
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.state.palette = None;
            surrender_focus(ctx);
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            let highlighted = self.state.palette.as_ref().map_or(0, |p| p.highlighted);
            if let Some(item) = items.get(highlighted).copied() {
                let effect = self.state.run_palette_item(&self.handle, item);
                self.apply_effect(effect);
                surrender_focus(ctx);
            }
        }
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Tab);
            i.consume_key(Modifiers::NONE, Key::F1);
            i.consume_key(Modifiers::NONE, Key::F2);
        });
    }

    /// Fayl amali oynasi ochiq: Enter — tasdiq/yaratish, Esc — bekor qilish.
    fn ops_keys(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.state.cancel_op(&self.handle);
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            match &self.state.ops.dialog {
                Some(OpDialog::Name(_)) => self.state.submit_name(&self.handle),
                Some(OpDialog::Confirm(_)) => self.state.confirm_plan(&self.handle),
                None => {}
            }
        }
        if !self.state.ops.is_modal() {
            surrender_focus(ctx);
        }
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Tab);
            i.consume_key(Modifiers::NONE, Key::F1);
            i.consume_key(Modifiers::NONE, Key::F2);
        });
    }

    /// "Open with" yoki tasdiq oynasi ochiq: navigatsiya tugmalari faqat ularga tegishli.
    fn opening_keys(&mut self, ctx: &egui::Context) {
        let escape = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if self.state.opening.confirm.is_some() {
            // Enter bilan tasodifan ishga tushmasin: faqat tugma bosilganda.
            ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
            if escape {
                self.state.answer_confirm(&self.handle, false);
            }
        } else if escape {
            self.state.close_open_with();
            surrender_focus(ctx);
        } else {
            for (key, forward) in [(Key::ArrowDown, true), (Key::ArrowUp, false)] {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    self.state.step_open_with(forward);
                }
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                self.state.confirm_open_with(&self.handle);
                surrender_focus(ctx);
            }
        }
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Tab);
            i.consume_key(Modifiers::NONE, Key::F1);
            i.consume_key(Modifiers::NONE, Key::F2);
        });
    }

    fn smoke_tick(&mut self, ctx: &egui::Context) {
        let Some((probe, frames)) = self.smoke.as_mut() else {
            return;
        };
        if self.smoke_started.elapsed() > std::time::Duration::from_secs(10) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        if !self.state.is_ready() || !self.state.listings_ready() {
            return;
        }
        *frames += 1;
        if *frames >= probe.frames_after_ready {
            probe.passed.store(true, Ordering::SeqCst);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for CommandorApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_installed {
            theme::install(ctx);
            self.theme_installed = true;
        }
        for event in self.events.drain() {
            self.state.apply(event);
        }
        self.state.pump(&self.handle);
        // Picker threadidan natija kelganmi?
        if let Some(rx) = &self.picker_rx
            && let Ok(result) = rx.try_recv()
        {
            self.picker_rx = None;
            if let Some(path) = result {
                self.state
                    .active_panel_mut()
                    .navigate(&self.handle, DirectoryTarget::Path(path));
            }
        }
        // Buyruqlar paneli: matn kiritilayotganda ham ochiladi/yopiladi.
        if ctx.input_mut(|i| i.consume_shortcut(&PALETTE_SHORTCUT)) {
            self.state.toggle_palette();
            surrender_focus(ctx);
        }
        if self.state.palette.is_some() {
            self.palette_keys(ctx);
            self.smoke_tick(ctx);
            return;
        }
        if self.state.ops.is_modal() {
            self.ops_keys(ctx);
            self.state.pump(&self.handle);
            self.smoke_tick(ctx);
            return;
        }
        if self.state.opening.is_modal() {
            self.opening_keys(ctx);
            self.smoke_tick(ctx);
            return;
        }
        // F1 — A panel, F2 — B panel tomlari ro‘yxati (matn kiritilayotganda ham ishlaydi).
        for (key, side) in [(egui::Key::F1, Side::Left), (egui::Key::F2, Side::Right)] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                self.state.toggle_volume_menu(&self.handle, side);
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
            }
        }
        if self.state.volume_menu.is_some() {
            // Menyu ochiq: strelka, Enter va Esc faqat menyuga tegishli; panel klaviaturasi to‘xtaydi.
            for (key, forward) in [(egui::Key::ArrowDown, true), (egui::Key::ArrowUp, false)] {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                    self.state.step_volume_menu(forward);
                }
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                self.state.confirm_volume_menu(&self.handle);
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.state.close_volume_menu();
            }
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace);
            });
            self.smoke_tick(ctx);
            return;
        }
        if !ctx.text_edit_focused() {
            // Matn kiritilmayotganda Tab tugmasi tugmalarni fokuslamasligi uchun focus navigationni to‘xtatish
            ctx.memory_mut(|m| {
                m.move_focus(egui::FocusDirection::None);
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });

            // Ctrl+Tab / Ctrl+Shift+Tab — panel ichidagi tablar (Shift varianti avval: moslash Shiftni e’tiborsiz qoldiradi).
            let tabs = &mut self.state.panels[self.state.active.index()];
            if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab)) {
                tabs.cycle(false);
            } else if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Tab)) {
                tabs.cycle(true);
            }
            // Buyruq yorliqlari: Shift’li yorliqlar avval tekshiriladi (masalan, Cmd+Shift+A va Cmd+A).
            for with_shift in [true, false] {
                for command in PaletteCommand::ALL {
                    let Some(shortcut) = command_shortcut(command) else {
                        continue;
                    };
                    if shortcut.modifiers.shift == with_shift
                        && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
                    {
                        let effect = self.state.run_command(&self.handle, command);
                        self.apply_effect(effect);
                    }
                }
            }
            // Delete va Cmd/Ctrl+Backspace — F8 (savat) muqobillari.
            if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Delete)
                    || i.consume_key(Modifiers::COMMAND, Key::Backspace)
            }) {
                self.state.start_trash(&self.handle);
            }
            let panel = self.state.active_panel_mut();
            if panel.pending.is_none() && panel.error.is_none() {
                let moves = [
                    (Key::ArrowDown, 1),
                    (Key::ArrowUp, -1),
                    (Key::PageDown, PAGE_ROWS),
                    (Key::PageUp, -PAGE_ROWS),
                ];
                for (key, delta) in moves {
                    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                        panel.move_cursor(delta);
                    }
                }
                for (key, end) in [(Key::Home, false), (Key::End, true)] {
                    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                        panel.cursor_to_edge(end);
                    }
                }
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space))
                    || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Insert))
                {
                    panel.select_current_and_step();
                }
            }
        }
        self.state.pump(&self.handle);
        self.smoke_tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let p = Palette::of(ui);
        egui::Panel::top("commandor-top")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(Margin::symmetric(16, 10)),
            )
            .show(ui, |ui| self.top_bar(ui));

        let toggled = egui::Panel::bottom("commandor-fkeys")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show(ui, |ui| self.bottom_bar(ui));
        match toggled.inner {
            Some(BottomAction::Volumes(side)) => self.state.toggle_volume_menu(&self.handle, side),
            Some(BottomAction::DismissNotice) => self.state.dismiss_notice(),
            Some(BottomAction::NewFolder) => self.state.start_new_folder(),
            Some(BottomAction::Trash) => self.state.start_trash(&self.handle),
            Some(BottomAction::Copy) => self
                .state
                .start_transfer(&self.handle, commandor_core::PlanAction::Copy),
            Some(BottomAction::Move) => self
                .state
                .start_transfer(&self.handle, commandor_core::PlanAction::Move),
            Some(BottomAction::CancelJob(job)) => self.state.cancel_job(&self.handle, job),
            None => {}
        }

        egui::CentralPanel::default_margins()
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(Margin::symmetric(14, 12)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                ui.columns(2, |cols| {
                    self.file_panel(&mut cols[0], Side::Left);
                    self.file_panel(&mut cols[1], Side::Right);
                });
            });

        self.ops_windows(ui.ctx(), &p);
        self.opening_windows(ui.ctx(), &p);

        if self.state.palette.is_some() {
            let items = self.palette_items();
            let mut action = None;
            if let Some(palette) = self.state.palette.as_mut() {
                palette_window(
                    ui.ctx(),
                    &p,
                    &self.i18n,
                    palette,
                    &items,
                    &self.state.bookmarks,
                    command_shortcut,
                    &mut action,
                );
            }
            match action {
                Some(PaletteAction::Run(item)) => {
                    let effect = self.state.run_palette_item(&self.handle, item);
                    self.apply_effect(effect);
                    surrender_focus(ui.ctx());
                }
                Some(PaletteAction::Highlight(index)) => {
                    if let Some(palette) = self.state.palette.as_mut() {
                        palette.highlighted = index;
                    }
                }
                Some(PaletteAction::Close) => {
                    self.state.palette = None;
                    surrender_focus(ui.ctx());
                }
                None => {}
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;

    if bytes < KIB {
        format!("{bytes} B")
    } else if bytes < MIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else if bytes < GIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes < TIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    }
}

/// Unix sekundlaridan `YYYY-MM-DD HH:MM` ko‘rinishidagi sana satri.
/// Tashqi chrono/time kutilmagan holda qo'shimcha bog'liqliksiz hisoblanadi.
fn format_timestamp(secs: u64) -> String {
    // Kunlar soni va kun ichidagi soniyalar
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;

    // 1970-01-01 dan boshlab yil/oy/kunni hisoblash (Grigorian taqvimi)
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

fn days_to_date(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap_year(year);
    let days_in_months: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &dim in &days_in_months {
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }
    let day = days + 1;
    (year, month, day)
}

const fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod formatting_tests {
    use super::*;

    #[test]
    fn format_bytes_correctly() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024 * 1024), "2.0 TiB");
    }

    #[test]
    fn format_timestamp_known_dates() {
        // 1970-01-01 00:00 UTC
        assert_eq!(format_timestamp(0), "1970-01-01 00:00");
        // 2026-11-07 12:30 UTC
        // 20764 kun * 86400 + 12 * 3600 + 30 * 60 = 1794009600 + 43200 + 1800 = 1794054600
        assert_eq!(format_timestamp(1794054600), "2026-11-07 12:30");
        let (y, m, d) = days_to_date(20764);
        assert_eq!((y, m, d), (2026, 11, 7));
    }

    #[test]
    fn palette_commands_are_translated_in_every_language() {
        for lang in commandor_i18n::Lang::ALL {
            let i18n = I18n::new(lang).expect("til yuklanadi");
            for command in PaletteCommand::ALL {
                assert!(i18n.has(command.label_key()), "{lang:?}: {command:?}");
            }
        }
    }

    #[test]
    fn command_shortcuts_are_unique_and_do_not_clash_with_palette() {
        let mut seen = std::collections::HashSet::new();
        for command in PaletteCommand::ALL {
            let Some(shortcut) = command_shortcut(command) else {
                continue;
            };
            assert_ne!(shortcut, PALETTE_SHORTCUT, "{command:?}");
            assert!(seen.insert(shortcut), "takroriy yorliq: {command:?}");
        }
    }
}
