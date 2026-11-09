# P3-02e — oraliq tanlash (Shift)

- Sana: 2026-11-09
- Holat: bajarildi (kod va testlar). GUI’da Shift bilan tanlash sinalmagan
- TODO: [P3-02e](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-04 (range select)

## Qilingan ish

[`state.rs`](../../crates/commandor-ui/src/state.rs) — `PanelState`ga oraliq tayanchi (`anchor`) va o‘sha paytdagi tanlov (`range_base`) qo‘shildi. Ishlash tartibi Explorer/Finder’dagidek:

- **Shift+↑/↓/PageUp/PageDown/Home/End** — kursor siljiydi. Tanlov = tayanch paytidagi tanlov ∪ [tayanch … kursor]. Orqaga qaytilsa oraliq qisqaradi, tayanchdan yuqoriga ham ishlaydi.
- **Shift+bosish** — tayanchdan bosilgan qatorgacha tanlanadi. Oddiy bosish, Ctrl/Cmd+bosish va Space/Insert tayanchni yangilaydi.
- **Tayanch yangilanishi.** Shift’siz harakat, navigatsiya, "hammasini tanlash" va "tanlovni bekor qilish" tayanchni unutadi. Keyingi Shift harakati kursordan yangi oraliq boshlaydi, avvalgi tanlov esa saqlanadi.
- **Kursor bo‘lmasa** oraliq birinchi qatordan boshlanadi.

[`lib.rs`](../../crates/commandor-ui/src/lib.rs): Shift varianti birinchi tekshiriladi, chunki `consume_key(NONE, …)` Shift’li bosishga ham mos keladi. Shift+Enter ("Dastur bilan ochish") va Shift+Tab o‘zgarmadi.

## Tekshiruv

macOS arm64, Rust 1.94.1:

- `cargo fmt --check` va `cargo clippy --workspace --all-targets -D warnings`: toza.
- `cargo test --workspace`: 132 test o‘tdi, 0 yiqildi, 1 tasi `ignore` qilingan. 3 ta yangi test:
  - kengayish/qisqarish, tayanchdan yuqoriga va oxirigacha;
  - avvalgi tanlovni saqlash, tayanchni tiklash;
  - kursorsiz va listingsiz holat.
- `--smoke`: SMOKE OK.

## Cheklov va qolgan ish

- GUI’da Shift bilan tanlash qo‘lda sinalmagan.
- Total Commander uslubidagi "Shift+strelka har qatorni almashtiradi" rejimi yo‘q. Klaviatura xaritasi sozlanadigan bo‘lganda ([P3-07](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)) tanlov sifatida qo‘shilishi mumkin.
