//! Panel tablari qatori va xatcho‘plar menyusi.

use crate::theme::Palette;
use crate::widgets::{paint_folder, truncated};
use commandor_i18n::I18n;
use eframe::egui::{
    self, Align, CornerRadius, CursorIcon, FontId, Layout, Pos2, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

/// Tab qatori foydalanuvchi amali; holatga `lib.rs` qo‘llaydi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StripAction {
    Select(usize),
    Close(usize),
    New,
    AddBookmark,
    OpenBookmark(usize),
    RemoveBookmark(usize),
}

/// Bitta tabni chizish uchun kerakli ma’lumot.
pub(crate) struct TabInfo {
    pub title: String,
    pub tooltip: String,
    pub loading: bool,
}

/// Tab qatori uchun ma’lumotlar.
pub(crate) struct StripModel<'a> {
    pub tabs: Vec<TabInfo>,
    pub current: usize,
    pub can_open: bool,
    pub bookmarks: &'a [String],
    pub bookmarked: bool,
    pub can_bookmark: bool,
}

const TAB_HEIGHT: f32 = 26.0;
const TAB_MAX_TEXT: f32 = 130.0;
const CLOSE_SIZE: f32 = 16.0;

fn paint_cross(painter: &egui::Painter, center: Pos2, color: egui::Color32) {
    let d = 3.5;
    let stroke = Stroke::new(1.4, color);
    painter.line_segment([center + vec2(-d, -d), center + vec2(d, d)], stroke);
    painter.line_segment([center + vec2(-d, d), center + vec2(d, -d)], stroke);
}

fn paint_plus(painter: &egui::Painter, center: Pos2, color: egui::Color32) {
    let d = 5.0;
    let stroke = Stroke::new(1.6, color);
    painter.line_segment([center + vec2(-d, 0.0), center + vec2(d, 0.0)], stroke);
    painter.line_segment([center + vec2(0.0, -d), center + vec2(0.0, d)], stroke);
}

/// Besh qirrali yulduz (xatcho‘p belgisi); `filled` bo‘lsa to‘ldiriladi.
pub(crate) fn paint_star(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    filled: bool,
    color: egui::Color32,
) {
    let points: Vec<Pos2> = (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { radius } else { radius * 0.45 };
            let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            center + vec2(angle.cos(), angle.sin()) * r
        })
        .collect();
    if filled {
        // Qavariq bo‘lmagan shakl: markazdan uchburchaklar bilan to‘ldiriladi.
        for i in 0..points.len() {
            let next = points[(i + 1) % points.len()];
            painter.add(egui::Shape::convex_polygon(
                vec![center, points[i], next],
                color,
                Stroke::NONE,
            ));
        }
    }
    painter.add(egui::Shape::closed_line(points, Stroke::new(1.3, color)));
}

/// Kichik kvadrat tugma (yangi tab, xatcho‘plar).
fn square_button(ui: &mut Ui, p: &Palette, tooltip: &str, enabled: bool) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(TAB_HEIGHT, TAB_HEIGHT), sense);
    let response = response.on_hover_text(tooltip);
    let response = if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    };
    if ui.is_rect_visible(rect) && enabled && response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(7), p.raised_hover);
    }
    response
}

fn tab_widget(
    ui: &mut Ui,
    p: &Palette,
    info: &TabInfo,
    current: bool,
    closable: bool,
    close_tip: &str,
) -> (Response, bool) {
    let text_color = if current { p.text } else { p.text_weak };
    let galley = truncated(
        ui,
        info.title.clone(),
        FontId::proportional(12.5),
        text_color,
        TAB_MAX_TEXT,
    );
    let close_w = if closable { CLOSE_SIZE + 4.0 } else { 0.0 };
    let width = 10.0 + 16.0 + 6.0 + galley.size().x + 10.0 + close_w;
    let (rect, response) = ui.allocate_exact_size(vec2(width, TAB_HEIGHT), Sense::click());
    let close_rect = Rect::from_center_size(
        pos2(rect.max.x - 6.0 - CLOSE_SIZE / 2.0, rect.center().y),
        vec2(CLOSE_SIZE, CLOSE_SIZE),
    );
    let close = closable.then(|| {
        ui.interact(close_rect, response.id.with("close"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(close_tip)
    });
    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(&info.tooltip);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = response.hovered() || close.as_ref().is_some_and(Response::hovered);
        let (fill, stroke) = if current {
            (p.surface_alt, p.border_strong)
        } else if hovered {
            (p.raised_hover, p.border)
        } else {
            (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT)
        };
        painter.rect(
            rect,
            CornerRadius::same(7),
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
        if current {
            let bar = Rect::from_min_size(
                pos2(rect.min.x + 8.0, rect.max.y - 2.5),
                vec2(rect.width() - 16.0, 2.0),
            );
            painter.rect_filled(bar, CornerRadius::same(1), p.accent);
        }
        let icon = pos2(rect.min.x + 10.0 + 8.0, rect.center().y);
        if info.loading {
            // Yuklanayotgan tab: kichik yarim halqa (animatsiyasiz, qayta chizishni talab qilmaydi).
            painter.circle_stroke(icon, 5.0, Stroke::new(1.5, p.text_faint));
            painter.circle_filled(icon + vec2(0.0, -5.0), 1.8, p.accent);
        } else {
            let color = if current { p.folder } else { p.text_faint };
            paint_folder(painter, icon, 0.8, color);
        }
        painter.galley(
            pos2(
                rect.min.x + 10.0 + 16.0 + 6.0,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            text_color,
        );
        if let Some(close) = &close
            && (current || hovered)
        {
            if close.hovered() {
                painter.rect_filled(close_rect, CornerRadius::same(4), p.raised);
            }
            let color = if close.hovered() {
                p.text
            } else {
                p.text_faint
            };
            paint_cross(painter, close_rect.center(), color);
        }
    }
    let close_clicked =
        close.is_some_and(|close| close.clicked()) || (closable && response.middle_clicked());
    (response, close_clicked)
}

fn bookmarks_popup(
    ui: &mut Ui,
    p: &Palette,
    i18n: &I18n,
    model: &StripModel<'_>,
    action: &mut Option<StripAction>,
) {
    ui.set_width(320.0);
    ui.spacing_mut().item_spacing.y = 2.0;
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(i18n.t("nav-bookmarks"))
                .size(13.0)
                .color(p.text),
        );
    });
    ui.add_space(2.0);
    if model.bookmarks.is_empty() {
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n.t("bookmarks-empty"))
                .size(12.0)
                .color(p.text_faint),
        );
        ui.add_space(6.0);
    }
    let remove_tip = i18n.t("bookmark-remove");
    for (index, path) in model.bookmarks.iter().enumerate() {
        let (rect, row) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
        let remove_rect = Rect::from_center_size(
            pos2(rect.max.x - 14.0, rect.center().y),
            vec2(CLOSE_SIZE, CLOSE_SIZE),
        );
        let remove = ui
            .interact(remove_rect, row.id.with("remove"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(&remove_tip);
        let row = row
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(path);
        let painter = ui.painter();
        if row.hovered() || remove.hovered() {
            painter.rect_filled(rect, CornerRadius::same(6), p.row_hover);
        }
        paint_star(
            painter,
            pos2(rect.min.x + 14.0, rect.center().y),
            5.5,
            true,
            p.warning,
        );
        let name = crate::state::path_label(path).to_owned();
        let name = truncated(ui, name, FontId::proportional(13.0), p.text, 110.0);
        let name_w = name.size().x;
        let name_h = name.size().y;
        painter.galley(
            pos2(rect.min.x + 28.0, rect.center().y - name_h / 2.0),
            name,
            p.text,
        );
        let detail_x = rect.min.x + 28.0 + name_w + 8.0;
        let detail = truncated(
            ui,
            path.clone(),
            FontId::proportional(11.0),
            p.text_faint,
            (remove_rect.min.x - detail_x - 6.0).max(20.0),
        );
        let detail_h = detail.size().y;
        painter.galley(
            pos2(detail_x, rect.center().y - detail_h / 2.0),
            detail,
            p.text_faint,
        );
        if row.hovered() || remove.hovered() {
            let color = if remove.hovered() {
                p.danger
            } else {
                p.text_faint
            };
            paint_cross(painter, remove_rect.center(), color);
        }
        if remove.clicked() {
            *action = Some(StripAction::RemoveBookmark(index));
        } else if row.clicked() {
            *action = Some(StripAction::OpenBookmark(index));
        }
    }
    ui.add_space(2.0);
    ui.separator();
    let label = if model.bookmarked {
        i18n.t("bookmark-already")
    } else {
        i18n.t("bookmark-add-current")
    };
    let enabled = model.can_bookmark && !model.bookmarked;
    if ui
        .add_enabled(enabled, egui::Button::new(label).min_size(vec2(0.0, 26.0)))
        .clicked()
    {
        *action = Some(StripAction::AddBookmark);
    }
    ui.label(
        RichText::new(i18n.t("bookmarks-session-note"))
            .size(11.0)
            .color(p.text_faint),
    );
}

/// Panel ichidagi tablar qatori: chapda tablar (gorizontal aylanadi), o‘ngda `+` va xatcho‘plar.
pub(crate) fn tab_strip(
    ui: &mut Ui,
    p: &Palette,
    i18n: &I18n,
    model: &StripModel<'_>,
    action: &mut Option<StripAction>,
) {
    let close_tip = i18n.t("tab-close");
    let closable = model.tabs.len() > 1;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), TAB_HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let marks = square_button(ui, p, &i18n.t("nav-bookmarks"), true);
            let star_color = if model.bookmarked {
                p.warning
            } else {
                p.text_weak
            };
            paint_star(
                ui.painter(),
                marks.rect.center(),
                6.5,
                model.bookmarked,
                star_color,
            );
            egui::Popup::menu(&marks)
                .gap(6.0)
                .show(|ui| bookmarks_popup(ui, p, i18n, model, action));

            let new_tab = square_button(ui, p, &i18n.t("tab-new"), model.can_open);
            let plus_color = if model.can_open {
                p.text_weak
            } else {
                p.text_faint.gamma_multiply(0.5)
            };
            paint_plus(ui.painter(), new_tab.rect.center(), plus_color);
            if new_tab.clicked() {
                *action = Some(StripAction::New);
            }

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                egui::ScrollArea::horizontal()
                    .id_salt("tab-strip")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for (index, info) in model.tabs.iter().enumerate() {
                                let current = index == model.current;
                                let (tab, close) =
                                    tab_widget(ui, p, info, current, closable, &close_tip);
                                if current && model.tabs.len() > 1 {
                                    tab.scroll_to_me(None);
                                }
                                if close {
                                    *action = Some(StripAction::Close(index));
                                } else if tab.clicked() && !current {
                                    *action = Some(StripAction::Select(index));
                                }
                            }
                        });
                    });
            });
        },
    );
}
