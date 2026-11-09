//! "Open with" oynasi va bajariladigan fayl tasdig‘i.

use crate::state::{FileRef, OpenWithMenu};
use crate::theme::Palette;
use crate::volumes_view::hint;
use crate::widgets::{paint_file, primary_button, truncated};
use commandor_core::AppId;
use commandor_i18n::I18n;
use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontId, Margin, Order, RichText, Sense, Shadow,
    Stroke, pos2, vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenWithAction {
    Choose(AppId),
    Highlight(usize),
    ToggleAll,
    SystemChooser,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmAction {
    Run,
    Cancel,
}

const WIDTH: f32 = 500.0;
const ROW: f32 = 40.0;
const MAX_ROWS: usize = 9;

/// Xiralashtirilgan fon; bosilsa `true`.
pub(crate) fn backdrop(ctx: &egui::Context, id: &str) -> bool {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new(id))
        .order(Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(90));
            response.clicked()
        })
        .inner
}

pub(crate) fn card<R>(
    ctx: &egui::Context,
    p: &Palette,
    id: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let screen = ctx.content_rect();
    let top = pos2(screen.center().x - WIDTH / 2.0, screen.min.y + 72.0);
    egui::Area::new(egui::Id::new(id))
        .order(Order::Foreground)
        .fixed_pos(top)
        .constrain(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.surface)
                .stroke(Stroke::new(1.0, p.border_strong))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(12))
                .shadow(Shadow {
                    offset: [0, 14],
                    blur: 36,
                    spread: 0,
                    color: p.shadow,
                })
                .show(ui, |ui| {
                    ui.set_width(WIDTH);
                    add(ui)
                })
                .inner
        })
        .inner
}

fn section(ui: &mut egui::Ui, p: &Palette, text: String) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label(
            RichText::new(text.to_uppercase())
                .size(10.5)
                .color(p.text_faint),
        );
    });
    ui.add_space(2.0);
}

pub(crate) fn open_with_window(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    menu: &mut OpenWithMenu,
    action: &mut Option<OpenWithAction>,
) {
    if backdrop(ctx, "open-with-backdrop") {
        *action = Some(OpenWithAction::Close);
    }
    card(ctx, p, "open-with", |ui| {
        ui.horizontal(|ui| {
            let (icon, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
            paint_file(ui.painter(), icon.center(), p.file, false);
            let title = i18n.t_args("open-with-title", &[("name", &menu.file.name)]);
            let galley = truncated(ui, title, FontId::proportional(14.0), p.text, WIDTH - 40.0);
            ui.label(galley);
        });
        ui.add_space(6.0);
        let input = ui.add(
            egui::TextEdit::singleline(&mut menu.query)
                .hint_text(i18n.t("open-with-search"))
                .char_limit(128)
                .margin(vec2(10.0, 7.0))
                .desired_width(ui.available_width()),
        );
        if !input.has_focus() {
            input.request_focus();
        }
        if input.changed() {
            menu.highlighted = 0;
        }
        ui.add_space(6.0);

        let visible = menu.visible();
        let status = if let Some(error) = &menu.error {
            Some((i18n.t(error.message_key()), p.danger))
        } else if menu.pending.is_some() {
            Some((i18n.t("open-with-loading"), p.text_weak))
        } else if visible.is_empty() {
            Some((i18n.t("open-with-empty"), p.text_faint))
        } else {
            None
        };
        if let Some((text, color)) = status {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                if menu.pending.is_some() {
                    ui.add(egui::Spinner::new().size(16.0).color(p.accent));
                }
                ui.label(RichText::new(text).color(color));
            });
            ui.add_space(10.0);
        }

        let highlighted = menu.highlighted.min(visible.len().saturating_sub(1));
        let any_recommended = visible.iter().any(|app| app.recommended);
        let any_other = visible.iter().any(|app| !app.recommended);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical()
                .id_salt("open-with-apps")
                .max_height(ROW * MAX_ROWS as f32)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (index, app) in visible.iter().enumerate() {
                        if index == 0 && any_recommended && any_other {
                            section(ui, p, i18n.t("open-with-recommended"));
                        }
                        if any_recommended
                            && !app.recommended
                            && visible
                                .get(index.wrapping_sub(1))
                                .is_some_and(|prev| prev.recommended)
                        {
                            section(ui, p, i18n.t("open-with-others"));
                        }
                        let (rect, row) =
                            ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
                        let row = row
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .on_hover_text(&app.detail);
                        let is_high = index == highlighted;
                        if is_high {
                            row.scroll_to_me(None);
                        }
                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter();
                            if is_high {
                                painter.rect_filled(rect, CornerRadius::same(7), p.accent_soft);
                                let bar = egui::Rect::from_min_size(
                                    pos2(rect.min.x + 1.0, rect.min.y + 9.0),
                                    vec2(3.0, rect.height() - 18.0),
                                );
                                painter.rect_filled(bar, CornerRadius::same(2), p.accent);
                            } else if row.hovered() {
                                painter.rect_filled(rect, CornerRadius::same(7), p.row_hover);
                            }
                            let color = if is_high { p.text } else { p.text_weak };
                            let mut right = rect.max.x - 10.0;
                            if app.is_default {
                                let text = painter.text(
                                    pos2(right, rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    i18n.t("open-with-default"),
                                    FontId::proportional(11.0),
                                    p.success,
                                );
                                right = text.min.x - 10.0;
                            }
                            let name = truncated(
                                ui,
                                app.name.clone(),
                                FontId::proportional(13.5),
                                color,
                                (right - rect.min.x - 18.0).max(40.0),
                            );
                            painter.galley(pos2(rect.min.x + 14.0, rect.min.y + 5.0), name, color);
                            let detail = truncated(
                                ui,
                                app.detail.clone(),
                                FontId::proportional(11.0),
                                p.text_faint,
                                (right - rect.min.x - 18.0).max(40.0),
                            );
                            painter.galley(
                                pos2(rect.min.x + 14.0, rect.max.y - 5.0 - detail.size().y),
                                detail,
                                p.text_faint,
                            );
                        }
                        if row.clicked() {
                            *action = Some(OpenWithAction::Choose(app.id));
                        } else if row.hovered()
                            && !is_high
                            && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                        {
                            *action = Some(OpenWithAction::Highlight(index));
                        }
                    }
                });
        });

        let total = menu.list.as_ref().map_or(0, |list| list.apps.len());
        let chooser = menu.list.as_ref().is_some_and(|list| list.system_chooser);
        if menu.has_hidden_apps() || menu.show_all || chooser {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if menu.has_hidden_apps() || menu.show_all {
                    let label = if menu.show_all {
                        i18n.t("open-with-show-less")
                    } else {
                        i18n.t_args("open-with-show-all", &[("count", &total.to_string())])
                    };
                    if ui.button(label).clicked() {
                        *action = Some(OpenWithAction::ToggleAll);
                    }
                }
                if chooser && primary_button(ui, &i18n.t("open-with-system")).clicked() {
                    *action = Some(OpenWithAction::SystemChooser);
                }
            });
        }
        ui.add_space(4.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.spacing_mut().item_spacing.x = 4.0;
            hint(ui, p, &["↑", "↓"], &i18n.t("volumes-hint-select"));
            hint(ui, p, &["Enter"], &i18n.t("open-hint-open"));
            hint(ui, p, &["Esc"], &i18n.t("volumes-hint-close"));
        });
    });
}

pub(crate) fn confirm_window(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    file: &FileRef,
    action: &mut Option<ConfirmAction>,
) {
    if backdrop(ctx, "confirm-backdrop") {
        *action = Some(ConfirmAction::Cancel);
    }
    card(ctx, p, "confirm-exec", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚠").size(20.0).color(p.warning));
            ui.label(
                RichText::new(i18n.t("confirm-exec-title"))
                    .size(15.0)
                    .color(p.text),
            );
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n.t_args("confirm-exec-body", &[("name", &file.name)]))
                .color(p.text_weak),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            // Xavfsiz tanlov birinchi va Esc bilan ham ishlaydi; ishga tushirish faqat bosish bilan.
            if ui
                .add(egui::Button::new(i18n.t("confirm-exec-cancel")).min_size(vec2(0.0, 28.0)))
                .clicked()
            {
                *action = Some(ConfirmAction::Cancel);
            }
            let run =
                egui::Button::new(RichText::new(i18n.t("confirm-exec-run")).color(p.on_accent))
                    .fill(p.danger)
                    .min_size(vec2(0.0, 28.0));
            if ui.add(run).clicked() {
                *action = Some(ConfirmAction::Run);
            }
        });
    });
}
