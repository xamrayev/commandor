//! Commandor vizual mavzusi: dark/light palitra, egui `Style` sozlamalari.
//!
//! Ilova tizim mavzusiga ergashadi (`ThemePreference::System`); ikkala mavzu uchun
//! uslub [`install`] orqali bir marta o‘rnatiladi. Maxsus chiziladigan elementlar
//! ranglarni [`Palette::of`] orqali joriy mavzudan oladi.

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Theme,
};

/// Premultiplied yarim shaffof rang (const kontekstda hisoblanadi).
const fn soft(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    const fn mul(c: u8, a: u8) -> u8 {
        ((c as u16 * a as u16) / 255) as u8
    }
    Color32::from_rgba_premultiplied(mul(r, a), mul(g, a), mul(b, a), a)
}

/// Semantik ranglar to‘plami.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Palette {
    /// Oyna foni.
    pub(crate) bg: Color32,
    /// Panel kartalari.
    pub(crate) surface: Color32,
    /// Jadval sarlavhasi, zebra qatorlar.
    pub(crate) surface_alt: Color32,
    /// Kiritish maydonlari va tugmalar.
    pub(crate) raised: Color32,
    pub(crate) raised_hover: Color32,
    pub(crate) border: Color32,
    pub(crate) border_strong: Color32,
    pub(crate) text: Color32,
    pub(crate) text_weak: Color32,
    pub(crate) text_faint: Color32,
    pub(crate) accent: Color32,
    pub(crate) accent_hover: Color32,
    pub(crate) accent_soft: Color32,
    pub(crate) on_accent: Color32,
    pub(crate) success: Color32,
    pub(crate) warning: Color32,
    pub(crate) warning_soft: Color32,
    pub(crate) danger: Color32,
    pub(crate) danger_soft: Color32,
    pub(crate) row_stripe: Color32,
    pub(crate) row_hover: Color32,
    pub(crate) row_cursor_idle: Color32,
    pub(crate) folder: Color32,
    pub(crate) file: Color32,
    pub(crate) link: Color32,
    pub(crate) shadow: Color32,
    pub(crate) glow: Color32,
}

impl Palette {
    pub(crate) const DARK: Self = Self {
        bg: Color32::from_rgb(0x0E, 0x10, 0x16),
        surface: Color32::from_rgb(0x15, 0x18, 0x21),
        surface_alt: Color32::from_rgb(0x1B, 0x1F, 0x2A),
        raised: Color32::from_rgb(0x20, 0x25, 0x33),
        raised_hover: Color32::from_rgb(0x2A, 0x30, 0x40),
        border: Color32::from_rgb(0x25, 0x2A, 0x37),
        border_strong: Color32::from_rgb(0x38, 0x3F, 0x52),
        text: Color32::from_rgb(0xE7, 0xEA, 0xF0),
        text_weak: Color32::from_rgb(0x9A, 0xA3, 0xB5),
        text_faint: Color32::from_rgb(0x63, 0x6B, 0x7E),
        accent: Color32::from_rgb(0x7C, 0x8C, 0xFF),
        accent_hover: Color32::from_rgb(0x95, 0xA2, 0xFF),
        accent_soft: soft(0x7C, 0x8C, 0xFF, 46),
        on_accent: Color32::from_rgb(0x0B, 0x0D, 0x14),
        success: Color32::from_rgb(0x3D, 0xD6, 0x8C),
        warning: Color32::from_rgb(0xF5, 0xB9, 0x4A),
        warning_soft: soft(0xF5, 0xB9, 0x4A, 30),
        danger: Color32::from_rgb(0xFF, 0x6B, 0x6B),
        danger_soft: soft(0xFF, 0x6B, 0x6B, 30),
        row_stripe: soft(0xFF, 0xFF, 0xFF, 6),
        row_hover: soft(0xFF, 0xFF, 0xFF, 14),
        row_cursor_idle: soft(0xFF, 0xFF, 0xFF, 22),
        folder: Color32::from_rgb(0xF2, 0xC5, 0x5C),
        file: Color32::from_rgb(0x8F, 0xA0, 0xBF),
        link: Color32::from_rgb(0x5C, 0xC8, 0xE0),
        shadow: soft(0x00, 0x00, 0x00, 90),
        glow: soft(0x7C, 0x8C, 0xFF, 40),
    };

    pub(crate) const LIGHT: Self = Self {
        bg: Color32::from_rgb(0xF2, 0xF3, 0xF7),
        surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        surface_alt: Color32::from_rgb(0xF6, 0xF7, 0xFA),
        raised: Color32::from_rgb(0xEE, 0xF0, 0xF5),
        raised_hover: Color32::from_rgb(0xE3, 0xE6, 0xEE),
        border: Color32::from_rgb(0xE2, 0xE5, 0xED),
        border_strong: Color32::from_rgb(0xC9, 0xCE, 0xDB),
        text: Color32::from_rgb(0x1B, 0x1F, 0x2A),
        text_weak: Color32::from_rgb(0x5B, 0x63, 0x75),
        text_faint: Color32::from_rgb(0x98, 0x9F, 0xB0),
        accent: Color32::from_rgb(0x4F, 0x5B, 0xD5),
        accent_hover: Color32::from_rgb(0x43, 0x4E, 0xC0),
        accent_soft: soft(0x4F, 0x5B, 0xD5, 34),
        on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        success: Color32::from_rgb(0x1F, 0x9D, 0x61),
        warning: Color32::from_rgb(0xB0, 0x6F, 0x12),
        warning_soft: soft(0xE0, 0x9B, 0x2D, 36),
        danger: Color32::from_rgb(0xD6, 0x45, 0x45),
        danger_soft: soft(0xD6, 0x45, 0x45, 26),
        row_stripe: soft(0x1B, 0x1F, 0x2A, 6),
        row_hover: soft(0x1B, 0x1F, 0x2A, 12),
        row_cursor_idle: soft(0x1B, 0x1F, 0x2A, 20),
        folder: Color32::from_rgb(0xE0, 0xA4, 0x1A),
        file: Color32::from_rgb(0x7A, 0x86, 0x9E),
        link: Color32::from_rgb(0x1C, 0x8F, 0xB0),
        shadow: soft(0x1B, 0x1F, 0x2A, 22),
        glow: soft(0x4F, 0x5B, 0xD5, 34),
    };

    /// Joriy `Ui` mavzusiga mos palitra.
    pub(crate) fn of(ui: &egui::Ui) -> Self {
        if ui.visuals().dark_mode {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }
}

/// Umumiy o‘lchamlar.
pub(crate) const RADIUS_CARD: u8 = 12;
pub(crate) const RADIUS_CONTROL: u8 = 7;
pub(crate) const ROW_HEIGHT: f32 = 28.0;

/// Ikkala mavzu uchun uslubni o‘rnatadi.
pub(crate) fn install(ctx: &egui::Context) {
    ctx.style_mut_of(Theme::Dark, |style| apply(style, &Palette::DARK));
    ctx.style_mut_of(Theme::Light, |style| apply(style, &Palette::LIGHT));
}

fn apply(style: &mut egui::Style, p: &Palette) {
    use FontFamily::{Monospace, Proportional};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, Proportional)),
        (TextStyle::Body, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(14.0, Proportional)),
        (TextStyle::Small, FontId::new(11.5, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Monospace)),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(8.0, 8.0);
    spacing.button_padding = egui::vec2(10.0, 5.0);
    spacing.interact_size.y = 28.0;
    spacing.window_margin = Margin::same(14);
    spacing.menu_margin = Margin::same(8);

    let v = &mut style.visuals;
    let control = CornerRadius::same(RADIUS_CONTROL);
    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: p.shadow,
    };
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 14,
        spread: 0,
        color: p.shadow,
    };
    v.faint_bg_color = p.surface_alt;
    v.extreme_bg_color = p.raised;
    v.text_edit_bg_color = Some(p.raised);
    v.code_bg_color = p.raised;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.warning;
    v.error_fg_color = p.danger;
    v.selection.bg_fill = p.accent_soft;
    v.selection.stroke = Stroke::new(1.0, p.accent);
    v.text_cursor.stroke = Stroke::new(2.0, p.accent);
    v.striped = false;

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    w.noninteractive.corner_radius = control;

    w.inactive.bg_fill = p.raised;
    w.inactive.weak_bg_fill = p.raised;
    w.inactive.bg_stroke = Stroke::new(1.0, p.border);
    w.inactive.fg_stroke = Stroke::new(1.0, p.text);
    w.inactive.corner_radius = control;
    w.inactive.expansion = 0.0;

    w.hovered.bg_fill = p.raised_hover;
    w.hovered.weak_bg_fill = p.raised_hover;
    w.hovered.bg_stroke = Stroke::new(1.0, p.border_strong);
    w.hovered.fg_stroke = Stroke::new(1.0, p.text);
    w.hovered.corner_radius = control;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = p.accent_soft;
    w.active.weak_bg_fill = p.accent_soft;
    w.active.bg_stroke = Stroke::new(1.0, p.accent);
    w.active.fg_stroke = Stroke::new(1.0, p.text);
    w.active.corner_radius = control;
    w.active.expansion = 0.0;

    w.open = w.hovered;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_colors_are_premultiplied() {
        let c = soft(0xFF, 0x80, 0x00, 51);
        assert_eq!(c.a(), 51);
        assert!(c.r() <= c.a() && c.g() <= c.a() && c.b() <= c.a());
        assert_eq!(c.r(), 51);
    }

    #[test]
    fn install_sets_both_themes() {
        let ctx = egui::Context::default();
        install(&ctx);
        assert_eq!(
            ctx.style_of(Theme::Dark).visuals.panel_fill,
            Palette::DARK.bg
        );
        assert_eq!(
            ctx.style_of(Theme::Light).visuals.panel_fill,
            Palette::LIGHT.bg
        );
    }
}
