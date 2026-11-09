//! Yangi papka / rename oynasi va savat tasdig‘i.

use crate::open_with_view::{backdrop, card};
use crate::state::{NameDialog, NameKind, PlanDialog};
use crate::theme::Palette;
use crate::volumes_view::hint;
use crate::widgets::{paint_file, paint_folder, primary_button, truncated};
use crate::{error_text, format_bytes};
use commandor_core::{ConflictPolicy, EntryKind, PlanAction};
use commandor_i18n::I18n;
use eframe::egui::{
    self, CornerRadius, FontId, Margin, RichText, Sense, pos2, text::CCursor, text::CCursorRange,
    vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpAction {
    Submit,
    Cancel,
    Policy(ConflictPolicy),
}

/// Fayl nomi kengaytmasiz qismining belgilar soni (`hisobot.txt` → 7; `.config` → hammasi).
fn stem_chars(name: &str) -> usize {
    match name.rfind('.') {
        Some(index) if index > 0 => name[..index].chars().count(),
        _ => name.chars().count(),
    }
}

fn error_box(ui: &mut egui::Ui, p: &Palette, text: String) {
    egui::Frame::new()
        .fill(p.danger_soft)
        .corner_radius(CornerRadius::same(7))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).size(12.5).color(p.danger));
        });
}

fn buttons(
    ui: &mut egui::Ui,
    i18n: &I18n,
    primary: &str,
    busy: bool,
    primary_enabled: bool,
    action: &mut Option<OpAction>,
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !busy,
                egui::Button::new(i18n.t("op-cancel")).min_size(vec2(0.0, 28.0)),
            )
            .clicked()
        {
            *action = Some(OpAction::Cancel);
        }
        ui.add_enabled_ui(!busy && primary_enabled, |ui| {
            if primary_button(ui, primary).clicked() {
                *action = Some(OpAction::Submit);
            }
        });
        if busy {
            ui.add(egui::Spinner::new().size(14.0));
            ui.label(RichText::new(i18n.t("op-working")).size(12.0));
        }
    });
}

pub(crate) fn name_window(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    dialog: &mut NameDialog,
    directory: &str,
    action: &mut Option<OpAction>,
) {
    if backdrop(ctx, "op-backdrop") && dialog.pending.is_none() {
        *action = Some(OpAction::Cancel);
    }
    card(ctx, p, "op-name", |ui| {
        let (title, hint_text, primary) = match &dialog.kind {
            NameKind::NewFolder => (
                i18n.t("op-new-folder-title"),
                i18n.t("op-new-folder-hint"),
                i18n.t("op-create"),
            ),
            NameKind::Rename { original, .. } => (
                i18n.t_args("op-rename-title", &[("name", original)]),
                i18n.t("op-rename-hint"),
                i18n.t("op-rename"),
            ),
        };
        ui.horizontal(|ui| {
            let (icon, _) = ui.allocate_exact_size(vec2(20.0, 18.0), Sense::hover());
            paint_folder(ui.painter(), icon.center(), 0.9, p.folder);
            let galley = truncated(ui, title, FontId::proportional(14.5), p.text, 440.0);
            ui.label(galley);
        });
        let dir = truncated(
            ui,
            i18n.t_args("op-in", &[("dir", directory)]),
            FontId::proportional(11.5),
            p.text_faint,
            480.0,
        );
        ui.label(dir);
        ui.add_space(6.0);
        let busy = dialog.pending.is_some();
        let output = ui
            .add_enabled_ui(!busy, |ui| {
                egui::TextEdit::singleline(&mut dialog.text)
                    .hint_text(hint_text)
                    .char_limit(255)
                    .margin(vec2(10.0, 7.0))
                    .desired_width(ui.available_width())
                    .show(ui)
            })
            .inner;
        if dialog.fresh {
            dialog.fresh = false;
            output.response.request_focus();
            let mut state = output.state;
            let end = stem_chars(&dialog.text);
            state
                .cursor
                .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(end))));
            state.store(ui.ctx(), output.response.id);
        } else if !busy && !output.response.has_focus() {
            output.response.request_focus();
        }
        if output.response.changed() {
            dialog.error = None;
        }
        if let Some(error) = &dialog.error {
            ui.add_space(4.0);
            error_box(ui, p, error_text(i18n, error));
        }
        ui.add_space(8.0);
        buttons(ui, i18n, &primary, busy, true, action);
        ui.add_space(2.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            hint(ui, p, &["Enter"], &primary.to_lowercase());
            hint(ui, p, &["Esc"], &i18n.t("volumes-hint-close"));
        });
    });
}

pub(crate) fn plan_window(
    ctx: &egui::Context,
    p: &Palette,
    i18n: &I18n,
    dialog: &PlanDialog,
    action: &mut Option<OpAction>,
) {
    let busy = dialog.pending.is_some() || dialog.replanning.is_some();
    if backdrop(ctx, "op-backdrop") && !busy {
        *action = Some(OpAction::Cancel);
    }
    let (icon, icon_color, title, body, confirm) = match dialog.plan.action {
        PlanAction::Copy => (
            "⎘",
            p.accent,
            "op-copy-title",
            "op-transfer-body",
            "op-copy-confirm",
        ),
        PlanAction::Move => (
            "➜",
            p.accent,
            "op-move-title",
            "op-transfer-body",
            "op-move-confirm",
        ),
        _ => (
            "🗑",
            p.danger,
            "op-trash-title",
            "op-trash-body",
            "op-trash-confirm",
        ),
    };
    card(ctx, p, "op-plan", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(18.0).color(icon_color));
            ui.label(RichText::new(i18n.t(title)).size(15.0).color(p.text));
        });
        let items = &dialog.plan.items;
        let total: u64 = dialog.plan.totals.map_or_else(
            || items.iter().filter_map(|item| item.size_bytes).sum(),
            |totals| totals.bytes,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(i18n.t_args(
                body,
                &[
                    ("count", &items.len().to_string()),
                    ("size", &format_bytes(total)),
                ],
            ))
            .color(p.text_weak),
        );
        let dir = truncated(
            ui,
            i18n.t_args("op-in", &[("dir", &dialog.plan.directory)]),
            FontId::proportional(11.5),
            p.text_faint,
            480.0,
        );
        ui.label(dir);
        if let Some(target) = &dialog.plan.target {
            let to = truncated(
                ui,
                i18n.t_args("op-to", &[("dir", target)]),
                FontId::proportional(11.5),
                p.accent,
                480.0,
            );
            ui.label(to);
        }
        ui.add_space(6.0);
        const SHOWN: usize = 8;
        egui::Frame::new()
            .fill(p.surface_alt)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for item in items.iter().take(SHOWN) {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
                    let icon = pos2(rect.min.x + 8.0, rect.center().y);
                    if item.kind == EntryKind::Directory {
                        paint_folder(ui.painter(), icon, 0.8, p.folder);
                    } else {
                        paint_file(ui.painter(), icon, p.file, item.kind == EntryKind::Symlink);
                    }
                    // Maqsadda shu nom bor — ogohlantirish rangi.
                    let color = if item.conflict { p.warning } else { p.text };
                    let name = truncated(
                        ui,
                        item.name.clone(),
                        FontId::proportional(13.0),
                        color,
                        rect.width() - 110.0,
                    );
                    let y = rect.center().y - name.size().y / 2.0;
                    ui.painter().galley(pos2(rect.min.x + 24.0, y), name, color);
                    if let Some(size) = item.size_bytes {
                        ui.painter().text(
                            pos2(rect.max.x, rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            format_bytes(size),
                            FontId::proportional(11.5),
                            p.text_faint,
                        );
                    }
                }
                if items.len() > SHOWN {
                    ui.label(
                        RichText::new(i18n.t_args(
                            "op-trash-more",
                            &[("count", &(items.len() - SHOWN).to_string())],
                        ))
                        .size(12.0)
                        .color(p.text_faint),
                    );
                }
            });
        let conflicts = dialog.conflicts();
        if dialog.transfer.is_some() && conflicts > 0 {
            ui.add_space(8.0);
            ui.label(
                RichText::new(i18n.t_args("op-conflicts", &[("count", &conflicts.to_string())]))
                    .color(p.warning),
            );
            ui.add_enabled_ui(!busy, |ui| {
                for (policy, key) in [
                    (ConflictPolicy::Skip, "policy-skip"),
                    (ConflictPolicy::KeepBoth, "policy-keep-both"),
                    (ConflictPolicy::Replace, "policy-replace"),
                ] {
                    if ui
                        .radio(dialog.plan.policy == policy, i18n.t(key))
                        .clicked()
                    {
                        *action = Some(OpAction::Policy(policy));
                    }
                }
            });
            if dialog.plan.policy == ConflictPolicy::Ask {
                ui.label(
                    RichText::new(i18n.t("op-choose-policy"))
                        .size(12.0)
                        .color(p.text_faint),
                );
            }
        }
        ui.add_space(8.0);
        // Konflikt siyosati tanlanmaguncha asosiy tugma o‘chiq; bekor qilish ishlaydi.
        let enabled = busy || dialog.can_confirm();
        buttons(ui, i18n, &i18n.t(confirm), busy, enabled, action);
        ui.add_space(2.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            hint(ui, p, &["Enter"], &i18n.t(confirm).to_lowercase());
            hint(ui, p, &["Esc"], &i18n.t("volumes-hint-close"));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::stem_chars;

    #[test]
    fn stem_excludes_only_the_last_extension() {
        assert_eq!(stem_chars("hisobot.txt"), 7);
        assert_eq!(stem_chars("arxiv.tar.gz"), 9);
        assert_eq!(stem_chars(".config"), 7);
        assert_eq!(stem_chars("README"), 6);
        assert_eq!(stem_chars("o‘zbek.md"), 6, "belgilar, baytlar emas");
    }
}
