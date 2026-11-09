//! Buyruqlar paneli (Ctrl/Cmd+Shift+P): qidiruv maydoni va mos buyruqlar ro‘yxati.

use crate::palette::{PaletteCommand, PaletteItem, PaletteState};
use crate::tabs_view::paint_star;
use crate::theme::Palette;
use crate::volumes_view::hint;
use crate::widgets::truncated;
use commandor_i18n::I18n;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, KeyboardShortcut, Margin, Order,
    RichText, Sense, Shadow, Stroke, pos2, vec2,
};

/// Panel oynasidagi sichqoncha amali.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteAction {
    Run(PaletteItem),
    Highlight(usize),
    Close,
}

const WIDTH: f32 = 520.0;
const ROW: f32 = 34.0;
const MAX_ROWS: usize = 10;

/// Ilova klaviatura yorliqlari bilan bir xil jadvaldan olinadigan yorliq matni.
pub(crate) type ShortcutOf = fn(PaletteCommand) -> Option<KeyboardShortcut>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn palette_window(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    state: &mut PaletteState,
    items: &[PaletteItem],
    bookmarks: &[String],
    shortcut_of: ShortcutOf,
    action: &mut Option<PaletteAction>,
) {
    // Orqa fonni xiralashtirish; unga bosish panelni yopadi.
    let screen = ctx.content_rect();
    let backdrop = egui::Area::new(egui::Id::new("palette-backdrop"))
        .order(Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(90));
            response
        });
    if backdrop.inner.clicked() {
        *action = Some(PaletteAction::Close);
    }

    let top = pos2(screen.center().x - WIDTH / 2.0, screen.min.y + 72.0);
    egui::Area::new(egui::Id::new("palette"))
        .order(Order::Foreground)
        .fixed_pos(top)
        .constrain(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.surface)
                .stroke(Stroke::new(1.0, p.border_strong))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(10))
                .shadow(Shadow {
                    offset: [0, 14],
                    blur: 36,
                    spread: 0,
                    color: p.shadow,
                })
                .show(ui, |ui| {
                    ui.set_width(WIDTH);
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(i18n.t("palette-title"))
                                .size(13.0)
                                .color(p.text),
                        );
                    });
                    ui.add_space(4.0);
                    let input = ui.add(
                        egui::TextEdit::singleline(&mut state.query)
                            .hint_text(i18n.t("palette-hint"))
                            .char_limit(128)
                            .margin(vec2(10.0, 7.0))
                            .desired_width(ui.available_width()),
                    );
                    if !input.has_focus() {
                        input.request_focus();
                    }
                    if input.changed() {
                        state.highlighted = 0;
                    }
                    ui.add_space(6.0);
                    if items.is_empty() {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(i18n.t("palette-empty")).color(p.text_faint));
                        });
                        ui.add_space(8.0);
                    }
                    let highlighted = state.highlighted.min(items.len().saturating_sub(1));
                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        egui::ScrollArea::vertical()
                            .id_salt("palette-items")
                            .max_height(ROW * MAX_ROWS as f32)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for (index, item) in items.iter().enumerate() {
                                    let (rect, row) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), ROW),
                                        Sense::click(),
                                    );
                                    let row = row.on_hover_cursor(CursorIcon::PointingHand);
                                    let is_high = index == highlighted;
                                    if is_high {
                                        row.scroll_to_me(None);
                                    }
                                    if ui.is_rect_visible(rect) {
                                        paint_item(
                                            ui,
                                            p,
                                            i18n,
                                            rect,
                                            *item,
                                            bookmarks,
                                            shortcut_of,
                                            is_high,
                                        );
                                    }
                                    if row.clicked() {
                                        *action = Some(PaletteAction::Run(*item));
                                    } else if row.hovered()
                                        && !is_high
                                        && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                                    {
                                        *action = Some(PaletteAction::Highlight(index));
                                    }
                                }
                            });
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.spacing_mut().item_spacing.x = 4.0;
                        hint(ui, p, &["↑", "↓"], &i18n.t("volumes-hint-select"));
                        hint(ui, p, &["Enter"], &i18n.t("palette-hint-run"));
                        hint(ui, p, &["Esc"], &i18n.t("volumes-hint-close"));
                    });
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn paint_item(
    ui: &egui::Ui,
    p: &Palette,
    i18n: &I18n,
    rect: egui::Rect,
    item: PaletteItem,
    bookmarks: &[String],
    shortcut_of: ShortcutOf,
    highlighted: bool,
) {
    let painter = ui.painter();
    if highlighted {
        painter.rect_filled(rect, CornerRadius::same(7), p.accent_soft);
        let bar = egui::Rect::from_min_size(
            pos2(rect.min.x + 1.0, rect.min.y + 8.0),
            vec2(3.0, rect.height() - 16.0),
        );
        painter.rect_filled(bar, CornerRadius::same(2), p.accent);
    }
    let cy = rect.center().y;
    let (label, detail, shortcut) = match item {
        PaletteItem::Command(command) => (
            i18n.t(command.label_key()),
            None,
            shortcut_of(command).map(|sc| ui.ctx().format_shortcut(&sc)),
        ),
        PaletteItem::Bookmark(index) => {
            let path = bookmarks.get(index).cloned().unwrap_or_default();
            let name = crate::state::path_label(&path).to_owned();
            (name, Some(path), None)
        }
    };
    let icon_x = rect.min.x + 18.0;
    match item {
        PaletteItem::Command(_) => {
            painter.circle_filled(
                pos2(icon_x, cy),
                3.0,
                if highlighted { p.accent } else { p.text_faint },
            );
        }
        PaletteItem::Bookmark(_) => paint_star(painter, pos2(icon_x, cy), 5.5, true, p.warning),
    }
    let mut right = rect.max.x - 10.0;
    if let Some(shortcut) = shortcut {
        let text = painter.text(
            pos2(right, cy),
            Align2::RIGHT_CENTER,
            shortcut,
            FontId::monospace(11.5),
            p.text_weak,
        );
        right = text.min.x - 10.0;
    }
    let text_x = rect.min.x + 32.0;
    let color = if highlighted { p.text } else { p.text_weak };
    let galley = truncated(
        ui,
        label,
        FontId::proportional(13.5),
        color,
        (right - text_x).max(40.0),
    );
    let label_w = galley.size().x;
    painter.galley(pos2(text_x, cy - galley.size().y / 2.0), galley, color);
    if let Some(detail) = detail {
        let x = text_x + label_w + 10.0;
        let galley = truncated(
            ui,
            detail,
            FontId::proportional(11.5),
            p.text_faint,
            (right - x).max(20.0),
        );
        painter.galley(pos2(x, cy - galley.size().y / 2.0), galley, p.text_faint);
    }
}
