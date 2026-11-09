# P3-02b — GUI interfeys tahlili va aniqlangan xatoliklarni tuzatish

- Sana: 2026-11-07
- Holat: bajarildi
- TODO: [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-01, UX-03, UX-04, UX-07, FS-01

## Qilingan ish

GUI interfeysi tahlil qilindi va aniqlangan quyidagi muhim xatoliklar hamda nomuvofiqliklar bartaraf etildi:

1. **Saralash o‘zgarganda tanlov (Multi-select) va kursor saqlanishi:**
   - [`crates/commandor-ui/src/state.rs`](../../crates/commandor-ui/src/state.rs): `PanelState::apply()` ichida yangi ro‘yxat qabul qilinganda avval barcha tanlov va kursor o‘chib ketardi. Endi `self.selected.retain(|id| new_ids.contains(id))` orqali mavjud tanlov va kursor yangi ro‘yxatda bo‘lsa saqlab qolinadi; faqat yangi katalogga o‘tishda (`navigate()`) tozalanadi.
2. **Navigatsiya tarixida mutlaq yo‘l saqlanishi (`StaleReference` himoyasi):**
   - [`crates/commandor-ui/src/state.rs`](../../crates/commandor-ui/src/state.rs): `history: Vec<DirectoryTarget>` da vaqtinchalik `LocationId` o‘rniga haqiqiy tasdiqlangan yo‘l (`DirectoryTarget::Path(display_path)`) yangilanadi; tarix o‘lchami 100 ta qadam bilan chegaralandi.
3. **Katta hajmdagi fayllar formati (GiB va TiB qo‘shildi):**
   - [`crates/commandor-ui/src/lib.rs`](../../crates/commandor-ui/src/lib.rs): `format_bytes` funksiyasi `GiB` va `TiB` birliklarini hisoblaydigan qilindi.
4. **Jadval ustunlari va sarlavhalari tekislanishi (Layout & Alignment):**
   - [`crates/commandor-ui/src/lib.rs`](../../crates/commandor-ui/src/lib.rs): Jadval sarlavhalarida Hajm o‘ngga (`right_text`), Sana chapga (`left_text`) tekislandi. Ro‘yxat qatorlarida `allocate_ui_with_layout` orqali Hajm o‘ng tomonga, Sana esa chap tomonga moslashtirildi.
5. **Filtr va Path maydonlarida Enter bosilganda fokusni bo‘shatish:**
   - [`crates/commandor-ui/src/lib.rs`](../../crates/commandor-ui/src/lib.rs): Foydalanuvchi yo‘l yoki filtr kiritib `Enter` bosganda `surrender_focus()` chaqirilib, klaviatura kursor boshqaruvi (Up/Down/Space) darhol ishlashi ta’minlandi.
6. **Tab tugmasining yuqori tugmalarga fokus berish effekti bartaraf etildi:**
   - [`crates/commandor-ui/src/lib.rs`](../../crates/commandor-ui/src/lib.rs): egui'ning sukut bo‘yicha widgetlar bo‘ylab fokus aylantirish mexanizmi (`FocusDirection::Next`) to‘xtatildi (`move_focus(FocusDirection::None)` va `surrender_focus`). Endi `Tab` tugmasi yuqoridagi tugmalarni tanlamaydi va faqat chap hamda o‘ng panellar o‘rtasida sakrash uchun xizmat qiladi.

## Tekshiruv

- `cargo test --workspace`: 47/47 ta test muvaffaqiyatli o‘tdi (yangi `refresh_preserves_valid_selection_and_cursor` va `GiB/TiB` testlari bilan).
- `cargo clippy --workspace --all-targets -- -D warnings`: xatosiz va ogohlantirishlarsiz o‘tdi.
- `cargo fmt --all -- --check`: kod uslubi to‘liq mos.
- `cargo run -p commandor-app -- --smoke`: SMOKE OK (macOS arm64).
- `cargo xtask check-boundaries`: 8 ta crate chegaralari to‘liq qoniqtirildi.

## Cheklov va qolgan ish

- Tablar tizimi (har bir panelda bir nechta tab).
- Kataloglar uchun xatcho‘plar (bookmarks).
- Buyruqlar palitrasi (Command Palette: `Ctrl/Cmd+Shift+P`).
- Keyingi TODO: [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11) doirasida tablar va xatcho‘plarni yakunlash.
