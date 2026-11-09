//! Qayta ishlatiladigan kichik vizual elementlar (pill, ikonka-tugma, klaviatura chipi).

use crate::theme::{Palette, RADIUS_CONTROL};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Galley, Response, RichText, Sense, Stroke,
    StrokeKind, Ui, text::LayoutJob, text::TextWrapping, vec2,
};
use std::sync::Arc;

/// Bir qatorli, kerak bo‘lsa `…` bilan qisqartirilgan matn galley’si.
pub(crate) fn truncated(
    ui: &Ui,
    text: String,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text, font, color);
    job.wrap = TextWrapping::truncate_at_width(max_width.max(8.0));
    ui.painter().layout_job(job)
}

/// Yumaloq fonli kichik yorliq.
pub(crate) fn pill(ui: &mut Ui, text: impl Into<String>, fg: Color32, bg: Color32) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.into(), FontId::proportional(11.5), fg);
    let pad = vec2(8.0, 3.0);
    let (rect, response) = ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::hover());
    if ui.is_rect_visible(rect) {
        let radius = CornerRadius::same((rect.height() / 2.0) as u8);
        ui.painter().rect_filled(rect, radius, bg);
        ui.painter().galley(rect.min + pad, galley, fg);
    }
    response
}

/// Rangli nuqta va matnli holat yorlig‘i (broker holati uchun).
pub(crate) fn status_pill(ui: &mut Ui, text: String, dot: Color32) -> Response {
    let p = Palette::of(ui);
    let galley = ui
        .painter()
        .layout_no_wrap(text, FontId::proportional(12.5), p.text_weak);
    let pad = vec2(10.0, 5.0);
    let dot_space = 14.0;
    let size = vec2(galley.size().x + dot_space, galley.size().y) + pad * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = CornerRadius::same((rect.height() / 2.0) as u8);
        painter.rect(
            rect,
            radius,
            p.surface,
            Stroke::new(1.0, p.border),
            StrokeKind::Inside,
        );
        let center = egui::pos2(rect.min.x + pad.x + 4.0, rect.center().y);
        painter.circle_filled(center, 6.0, dot.gamma_multiply(0.25));
        painter.circle_filled(center, 3.5, dot);
        painter.galley(
            egui::pos2(rect.min.x + pad.x + dot_space, rect.min.y + pad.y),
            galley,
            p.text_weak,
        );
    }
    response
}

/// Kvadrat ikonka-tugma; nomi tooltip sifatida ko‘rsatiladi.
pub(crate) fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str, enabled: bool) -> bool {
    let p = Palette::of(ui);
    let button = egui::Button::new(RichText::new(icon).size(14.0).color(p.text))
        .min_size(vec2(30.0, 28.0))
        .fill(p.raised)
        .stroke(Stroke::new(1.0, p.border));
    ui.add_enabled(enabled, button)
        .on_hover_text(tooltip)
        .on_disabled_hover_text(tooltip)
        .clicked()
}

/// To‘ldirilgan asosiy (accent) tugma.
pub(crate) fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let p = Palette::of(ui);
    let button = egui::Button::new(RichText::new(label).color(p.on_accent))
        .fill(p.accent)
        .stroke(Stroke::NONE)
        .min_size(vec2(0.0, 28.0));
    let response = ui.add(button);
    if response.hovered() {
        // Hover: accent rangdagi nozik halqa.
        ui.painter().rect_stroke(
            response.rect.expand(1.5),
            CornerRadius::same(RADIUS_CONTROL + 2),
            Stroke::new(1.5, p.accent_hover.gamma_multiply(0.6)),
            StrokeKind::Outside,
        );
    }
    response
}

/// Holatni almashtiruvchi chip (masalan, yashirin fayllar).
pub(crate) fn toggle_chip(ui: &mut Ui, on: bool, icon: &str, label: &str) -> Response {
    let p = Palette::of(ui);
    let (fg, fill, stroke) = if on {
        (p.accent, p.accent_soft, p.accent)
    } else {
        (p.text_weak, p.raised, p.border)
    };
    let button = egui::Button::new(RichText::new(format!("{icon}  {label}")).color(fg))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .min_size(vec2(0.0, 28.0));
    ui.add(button)
}

/// `F5` kabi klaviatura belgisi.
pub(crate) fn key_badge(ui: &mut Ui, key: &str, enabled: bool) {
    let p = Palette::of(ui);
    let fg = if enabled { p.text } else { p.text_faint };
    let galley = ui
        .painter()
        .layout_no_wrap(key.to_owned(), FontId::monospace(11.5), fg);
    let pad = vec2(6.0, 2.0);
    let size = vec2(
        (galley.size().x + pad.x * 2.0).max(22.0),
        galley.size().y + pad.y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = CornerRadius::same(5);
        painter.rect(
            rect,
            radius,
            p.raised,
            Stroke::new(1.0, p.border_strong),
            StrokeKind::Inside,
        );
        // Pastki "qalinlik" chizig‘i — fizik klaviatura tugmasi taassuroti.
        painter.hline(
            (rect.min.x + 2.0)..=(rect.max.x - 2.0),
            rect.max.y - 0.5,
            Stroke::new(1.0, p.border_strong),
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            key,
            FontId::monospace(11.5),
            fg,
        );
    }
}

/// `F3 Ko‘rish` ko‘rinishidagi funksiya tugmasi chipi.
pub(crate) fn fkey_chip(ui: &mut Ui, text: &str, enabled: bool, tooltip: &str) -> Response {
    let p = Palette::of(ui);
    let (key, label) = text.split_once(' ').unwrap_or(("", text));
    let response = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if !key.is_empty() {
                key_badge(ui, key, enabled);
            }
            let color = if enabled { p.text_weak } else { p.text_faint };
            ui.label(RichText::new(label).size(12.5).color(color));
        })
        .response;
    let response = if enabled {
        response
            .interact(Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    };
    response.on_hover_text(tooltip)
}

/// Vektor papka ikonkasi (`scale` = 1.0 → taxminan 16×13 px).
pub(crate) fn paint_folder(
    painter: &egui::Painter,
    center: egui::Pos2,
    scale: f32,
    color: Color32,
) {
    let w = 16.0 * scale;
    let h = 12.0 * scale;
    let body = egui::Rect::from_center_size(center + vec2(0.0, 1.0 * scale), vec2(w, h));
    let tab = egui::Rect::from_min_size(
        body.min - vec2(0.0, 2.5 * scale),
        vec2(w * 0.42, 4.0 * scale),
    );
    let r = CornerRadius::same((2.0 * scale) as u8);
    painter.rect_filled(tab, r, color.gamma_multiply(0.75));
    painter.rect_filled(body, r, color);
}

/// Vektor hujjat ikonkasi; `link` bo‘lsa ramziy havola strelkasi qo‘shiladi.
pub(crate) fn paint_file(painter: &egui::Painter, center: egui::Pos2, color: Color32, link: bool) {
    let rect = egui::Rect::from_center_size(center, vec2(11.0, 14.0));
    painter.rect(
        rect,
        CornerRadius::same(2),
        color.gamma_multiply(0.18),
        Stroke::new(1.2, color),
        StrokeKind::Inside,
    );
    let stroke = Stroke::new(1.0, color.gamma_multiply(0.8));
    if link {
        // ↗ ko‘rinishidagi kichik strelka.
        let a = egui::pos2(rect.min.x + 3.0, rect.max.y - 3.5);
        let b = egui::pos2(rect.max.x - 3.0, rect.min.y + 4.0);
        painter.line_segment([a, b], stroke);
        painter.line_segment([b, b + vec2(-3.0, 0.0)], stroke);
        painter.line_segment([b, b + vec2(0.0, 3.0)], stroke);
    } else {
        for dy in [-2.0, 1.0, 4.0] {
            let y = center.y + dy;
            painter.hline((rect.min.x + 3.0)..=(rect.max.x - 3.0), y, stroke);
        }
    }
}

/// Tanlangan yozuv uchun doira ichidagi belgi.
pub(crate) fn paint_check(
    painter: &egui::Painter,
    center: egui::Pos2,
    color: Color32,
    on: Color32,
) {
    painter.circle_filled(center, 7.0, color);
    let stroke = Stroke::new(1.8, on);
    let a = center + vec2(-3.5, 0.2);
    let b = center + vec2(-1.0, 2.8);
    let c = center + vec2(3.6, -2.6);
    painter.line_segment([a, b], stroke);
    painter.line_segment([b, c], stroke);
}

/// Saralash yo‘nalishi uchburchagi.
pub(crate) fn paint_sort_arrow(
    painter: &egui::Painter,
    center: egui::Pos2,
    ascending: bool,
    color: Color32,
) {
    let (tip, base) = if ascending { (-3.0, 2.5) } else { (3.0, -2.5) };
    let points = vec![
        center + vec2(0.0, tip),
        center + vec2(4.0, base),
        center + vec2(-4.0, base),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}
