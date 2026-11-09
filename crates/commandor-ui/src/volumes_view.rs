//! Tomlar (disklar) paneli: yo‘l ustidagi chiplar va F1/F2 bilan ochiladigan ro‘yxat.

use crate::format_bytes;
use crate::state::{Side, VolumeMenu, VolumesState};
use crate::theme::Palette;
use crate::widgets::{key_badge, truncated};
use commandor_core::{Volume, VolumeKind};
use commandor_i18n::I18n;
use eframe::egui::{
    self, Align2, CornerRadius, CursorIcon, FontId, Margin, Order, Pos2, Rect, Response, Sense,
    Shadow, Stroke, StrokeKind, Ui, pos2, vec2,
};

/// Tomlar paneli foydalanuvchi amali; holatga `lib.rs` qo‘llaydi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VolumeAction {
    Toggle,
    Choose(usize),
    Highlight(usize),
    Close,
}

const CHIP_HEIGHT: f32 = 26.0;
const MENU_WIDTH: f32 = 360.0;
const MENU_ROW: f32 = 50.0;

/// Tom turiga mos rang.
pub(crate) fn kind_color(p: &Palette, kind: VolumeKind) -> egui::Color32 {
    match kind {
        VolumeKind::System => p.accent,
        VolumeKind::Internal => p.text_weak,
        VolumeKind::Removable => p.folder,
        VolumeKind::Network => p.link,
        VolumeKind::Other => p.text_faint,
    }
}

fn kind_key(kind: VolumeKind) -> &'static str {
    match kind {
        VolumeKind::System => "volume-kind-system",
        VolumeKind::Internal => "volume-kind-internal",
        VolumeKind::Removable => "volume-kind-removable",
        VolumeKind::Network => "volume-kind-network",
        VolumeKind::Other => "volume-kind-other",
    }
}

/// Vektor disk ikonkasi: turiga qarab ulagich yoki tarmoq belgisi qo‘shiladi.
pub(crate) fn paint_disk(
    painter: &egui::Painter,
    center: Pos2,
    scale: f32,
    kind: VolumeKind,
    color: egui::Color32,
) {
    let body = Rect::from_center_size(center, vec2(16.0 * scale, 10.0 * scale));
    let radius = CornerRadius::same((2.5 * scale) as u8);
    painter.rect(
        body,
        radius,
        color.gamma_multiply(0.22),
        Stroke::new(1.3 * scale.max(1.0), color),
        StrokeKind::Inside,
    );
    // Holat chirog‘i va ajratish chizig‘i.
    painter.circle_filled(
        pos2(body.max.x - 3.5 * scale, body.center().y + 1.5 * scale),
        1.3 * scale,
        color,
    );
    painter.hline(
        (body.min.x + 3.0 * scale)..=(body.max.x - 7.0 * scale),
        body.center().y + 1.5 * scale,
        Stroke::new(1.0, color.gamma_multiply(0.7)),
    );
    match kind {
        VolumeKind::Removable => {
            let plug = Rect::from_min_size(
                pos2(body.min.x - 3.0 * scale, body.center().y - 2.0 * scale),
                vec2(3.0 * scale, 4.0 * scale),
            );
            painter.rect_filled(plug, CornerRadius::same(1), color);
        }
        VolumeKind::Network => {
            let top = pos2(body.center().x, body.max.y);
            let bottom = top + vec2(0.0, 3.0 * scale);
            painter.line_segment([top, bottom], Stroke::new(1.2, color));
            painter.hline(
                (bottom.x - 5.0 * scale)..=(bottom.x + 5.0 * scale),
                bottom.y,
                Stroke::new(1.2, color),
            );
        }
        VolumeKind::System | VolumeKind::Internal | VolumeKind::Other => {}
    }
}

/// Pastga qaragan kichik burchak (chevron).
fn paint_chevron(painter: &egui::Painter, center: Pos2, open: bool, color: egui::Color32) {
    let (a, b) = if open { (2.0, -2.0) } else { (-2.0, 2.0) };
    let stroke = Stroke::new(1.5, color);
    painter.line_segment([center + vec2(-4.0, a), center + vec2(0.0, b)], stroke);
    painter.line_segment([center + vec2(0.0, b), center + vec2(4.0, a)], stroke);
}

fn volume_tooltip(volume: &Volume, i18n: &I18n) -> String {
    let free = i18n.t_args(
        "volumes-free",
        &[
            ("free", &format_bytes(volume.available_bytes)),
            ("total", &format_bytes(volume.total_bytes)),
        ],
    );
    let mut text = format!(
        "{}\n{} · {} · {}",
        volume.mount_path,
        i18n.t(kind_key(volume.kind)),
        volume.file_system,
        free
    );
    if volume.read_only {
        text.push_str(" · ");
        text.push_str(&i18n.t("volumes-read-only"));
    }
    text
}

/// Ro‘yxatni ochuvchi tugma: disk ikonkasi, chevron va F1/F2 belgisi.
fn menu_button(ui: &mut Ui, p: &Palette, open: bool, key: &str, tooltip: &str) -> Response {
    let key_galley =
        ui.painter()
            .layout_no_wrap(key.to_owned(), FontId::monospace(10.5), p.text_weak);
    let width = 10.0 + 18.0 + 6.0 + 8.0 + 8.0 + key_galley.size().x + 8.0 + 8.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, CHIP_HEIGHT), Sense::click());
    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke) = if open {
            (p.accent_soft, p.accent)
        } else if response.hovered() {
            (p.raised_hover, p.border_strong)
        } else {
            (p.raised, p.border)
        };
        painter.rect(
            rect,
            CornerRadius::same(7),
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
        let cy = rect.center().y;
        let icon_color = if open { p.accent } else { p.text };
        paint_disk(
            painter,
            pos2(rect.min.x + 10.0 + 9.0, cy),
            1.0,
            VolumeKind::Internal,
            icon_color,
        );
        paint_chevron(
            painter,
            pos2(rect.min.x + 10.0 + 18.0 + 6.0 + 4.0, cy),
            open,
            icon_color,
        );
        let badge = Rect::from_min_size(
            pos2(rect.max.x - 8.0 - key_galley.size().x - 8.0, cy - 8.0),
            vec2(key_galley.size().x + 8.0, 16.0),
        );
        painter.rect(
            badge,
            CornerRadius::same(4),
            p.surface,
            Stroke::new(1.0, p.border_strong),
            StrokeKind::Inside,
        );
        painter.galley(
            badge.center() - key_galley.size() / 2.0,
            key_galley,
            p.text_weak,
        );
    }
    response
}

/// Bitta tom chipi: ikonka + nom; joriy tom accent bilan ajratiladi.
fn volume_chip(ui: &mut Ui, p: &Palette, volume: &Volume, current: bool) -> Response {
    let text_color = if current { p.text } else { p.text_weak };
    let galley = truncated(
        ui,
        volume.name.clone(),
        FontId::proportional(12.5),
        text_color,
        150.0,
    );
    let width = 8.0 + 18.0 + 6.0 + galley.size().x + 10.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, CHIP_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke) = if current {
            (p.accent_soft, p.accent)
        } else if response.hovered() {
            (p.raised_hover, p.border_strong)
        } else {
            (p.surface_alt, p.border)
        };
        painter.rect(
            rect,
            CornerRadius::same((CHIP_HEIGHT / 2.0) as u8),
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
        let cy = rect.center().y;
        paint_disk(
            painter,
            pos2(rect.min.x + 8.0 + 9.0, cy),
            0.85,
            volume.kind,
            kind_color(p, volume.kind),
        );
        painter.galley(
            pos2(rect.min.x + 8.0 + 18.0 + 6.0, cy - galley.size().y / 2.0),
            galley,
            text_color,
        );
    }
    response
}

/// Yo‘l maydoni ustidagi tomlar qatori. Ro‘yxat tugmasining joyini ham qaytaradi
/// (popup shu tugma ostida ochiladi).
#[allow(clippy::too_many_arguments)]
pub(crate) fn volume_bar(
    ui: &mut Ui,
    p: &Palette,
    i18n: &I18n,
    volumes: &VolumesState,
    current: Option<usize>,
    menu_open: bool,
    side: Side,
    action: &mut Option<VolumeAction>,
) -> Rect {
    let key = match side {
        Side::Left => "F1",
        Side::Right => "F2",
    };
    let tooltip = format!("{} ({key})", i18n.t("volumes-open"));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let button = menu_button(ui, p, menu_open, key, &tooltip);
        if button.clicked() {
            *action = Some(VolumeAction::Toggle);
        }
        egui::ScrollArea::horizontal()
            .id_salt("volume-chips")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for (index, volume) in volumes.list.iter().enumerate() {
                        let chip = volume_chip(ui, p, volume, current == Some(index))
                            .on_hover_text(volume_tooltip(volume, i18n));
                        if chip.clicked() {
                            *action = Some(VolumeAction::Choose(index));
                        }
                    }
                });
            });
        button.rect
    })
    .inner
}

/// Band qilingan joy ulushi ko‘rsatkichi.
fn usage_bar(painter: &egui::Painter, p: &Palette, rect: Rect, volume: &Volume) {
    painter.rect_filled(rect, CornerRadius::same(2), p.raised);
    if volume.total_bytes == 0 {
        return;
    }
    let used = volume.total_bytes.saturating_sub(volume.available_bytes) as f64;
    let ratio = (used / volume.total_bytes as f64).clamp(0.0, 1.0) as f32;
    let color = if ratio > 0.95 {
        p.danger
    } else if ratio > 0.85 {
        p.warning
    } else {
        p.accent
    };
    let fill = Rect::from_min_size(rect.min, vec2(rect.width() * ratio, rect.height()));
    painter.rect_filled(fill, CornerRadius::same(2), color);
}

fn menu_row(
    ui: &mut Ui,
    p: &Palette,
    i18n: &I18n,
    volume: &Volume,
    highlighted: bool,
    current: bool,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), MENU_ROW), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    let radius = CornerRadius::same(8);
    if highlighted {
        painter.rect_filled(rect, radius, p.accent_soft);
        let bar = Rect::from_min_size(
            pos2(rect.min.x + 1.0, rect.min.y + 10.0),
            vec2(3.0, rect.height() - 20.0),
        );
        painter.rect_filled(bar, CornerRadius::same(2), p.accent);
    } else if response.hovered() {
        painter.rect_filled(rect, radius, p.row_hover);
    }

    // Ikonka plitkasi.
    let tile = Rect::from_center_size(pos2(rect.min.x + 28.0, rect.center().y), vec2(34.0, 34.0));
    let color = kind_color(p, volume.kind);
    painter.rect_filled(tile, CornerRadius::same(8), color.gamma_multiply(0.14));
    paint_disk(painter, tile.center(), 1.15, volume.kind, color);

    let text_x = tile.max.x + 12.0;
    let right_w = 110.0;
    let text_w = rect.max.x - text_x - right_w - 12.0;
    let name_color = p.text;
    let name = truncated(
        ui,
        volume.name.clone(),
        FontId::proportional(14.0),
        name_color,
        text_w,
    );
    let name_h = name.size().y;
    painter.galley(pos2(text_x, rect.min.y + 8.0), name, name_color);
    if current {
        painter.circle_filled(
            pos2(text_x - 6.0, rect.min.y + 8.0 + name_h / 2.0),
            2.5,
            p.success,
        );
    }
    let mut detail = format!("{} · {}", i18n.t(kind_key(volume.kind)), volume.mount_path);
    if volume.read_only {
        detail.push_str(" · ");
        detail.push_str(&i18n.t("volumes-read-only"));
    }
    let detail = truncated(ui, detail, FontId::proportional(11.5), p.text_faint, text_w);
    painter.galley(
        pos2(text_x, rect.max.y - 8.0 - detail.size().y),
        detail,
        p.text_faint,
    );

    // O‘ng tomonda: bo‘sh joy va band qilingan ulush.
    let right = rect.max.x - 12.0;
    painter.text(
        pos2(right, rect.min.y + 15.0),
        Align2::RIGHT_CENTER,
        format!(
            "{} / {}",
            format_bytes(volume.available_bytes),
            format_bytes(volume.total_bytes)
        ),
        FontId::proportional(11.0),
        p.text_weak,
    );
    let bar = Rect::from_min_size(pos2(right - 90.0, rect.max.y - 15.0), vec2(90.0, 4.0));
    usage_bar(painter, p, bar, volume);
    response.on_hover_text(volume_tooltip(volume, i18n))
}

pub(crate) fn hint(ui: &mut Ui, p: &Palette, keys: &[&str], text: &str) {
    for key in keys {
        key_badge(ui, key, true);
    }
    ui.label(egui::RichText::new(text).size(11.5).color(p.text_faint));
    ui.add_space(6.0);
}

/// F1/F2 yoki tugma bilan ochilgan tomlar ro‘yxati (popup).
#[allow(clippy::too_many_arguments)]
pub(crate) fn volume_menu(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    volumes: &VolumesState,
    menu: VolumeMenu,
    current: Option<usize>,
    anchor: Rect,
    panel_title: &str,
    action: &mut Option<VolumeAction>,
) {
    let area = egui::Area::new(egui::Id::new(("volume-menu", menu.side.index())))
        .order(Order::Foreground)
        .fixed_pos(anchor.left_bottom() + vec2(0.0, 6.0))
        .constrain(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.surface)
                .stroke(Stroke::new(1.0, p.border_strong))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(8))
                .shadow(Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: p.shadow,
                })
                .show(ui, |ui| {
                    ui.set_width(MENU_WIDTH);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(i18n.t("volumes-title"))
                                .size(13.0)
                                .color(p.text),
                        );
                        ui.label(
                            egui::RichText::new(format!("· {panel_title}"))
                                .size(12.0)
                                .color(p.text_faint),
                        );
                        if volumes.pending.is_some() {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add(egui::Spinner::new().size(12.0).color(p.text_faint));
                                },
                            );
                        }
                    });
                    ui.add_space(4.0);
                    if volumes.list.is_empty() {
                        let text = if let Some(error) = &volumes.error {
                            (i18n.t(error.message_key()), p.danger)
                        } else if volumes.pending.is_some() {
                            (i18n.t("volumes-loading"), p.text_weak)
                        } else {
                            (i18n.t("volumes-empty"), p.text_faint)
                        };
                        ui.add_space(10.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(text.0).color(text.1));
                        });
                        ui.add_space(10.0);
                    }
                    for (index, volume) in volumes.list.iter().enumerate() {
                        let row = menu_row(
                            ui,
                            p,
                            i18n,
                            volume,
                            menu.highlighted == index,
                            current == Some(index),
                        );
                        if row.clicked() {
                            *action = Some(VolumeAction::Choose(index));
                        } else if row.hovered()
                            && menu.highlighted != index
                            && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                        {
                            // Faqat sichqoncha harakatlanganda: klaviatura tanlovini bosib ketmaydi.
                            *action = Some(VolumeAction::Highlight(index));
                        }
                    }
                    ui.add_space(4.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.spacing_mut().item_spacing.x = 4.0;
                        hint(ui, p, &["↑", "↓"], &i18n.t("volumes-hint-select"));
                        hint(ui, p, &["Enter"], &i18n.t("volumes-hint-open"));
                        hint(ui, p, &["Esc"], &i18n.t("volumes-hint-close"));
                    });
                });
        });
    // Tashqariga bosilsa yopiladi (tugmaning o‘zi `Toggle` beradi).
    let menu_rect = area.response.rect;
    let clicked_outside = ctx.input(|i| {
        i.pointer.any_pressed()
            && i.pointer
                .interact_pos()
                .is_some_and(|pos| !menu_rect.contains(pos) && !anchor.contains(pos))
    });
    if clicked_outside && action.is_none() {
        *action = Some(VolumeAction::Close);
    }
}
