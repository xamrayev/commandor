# P3-01b — UI metadata ustunlari, native OS papka tanlash va klaviatura navigatsiyasi testlari

- Sana: 2026-11-07
- Holat: P3-01b bajarildi; umumiy P3-01 yakunlandi
- TODO: [3-faza](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-01, UX-03, UX-04, UX-06, UX-07, FS-01

## Qilingan ish

- **Jadval sarlavhasi va metadata ustunlari**: [commandor-ui](../../crates/commandor-ui/src/lib.rs) da fayl ro‘yxati endi ustunli jadval ko‘rinishida chiziladi:
  - `list-col-name` (Nom)
  - `list-col-size` (Hajm, `format_bytes`)
  - `list-col-modified` (O‘zgartirilgan sana va vaqt, `format_timestamp` orqali `YYYY-MM-DD HH:MM`)
- **Native OS papka tanlash (Folder Picker)**:
  - Arxitektura chegarasini saqlash uchun `commandor-ui`da [`FolderPicker`](../../crates/commandor-ui/src/lib.rs) traiti joriy qilindi (UI bevosita disk yoki tizim kutubxonalariga kirmaydi).
  - [`commandor-app`](../../crates/commandor-app/src/main.rs) da `rfd::FileDialog` orqali native macOS Cocoa dialogi ochiladi. Dialog alohida threadda chaqiriladi, shuning uchun egui render sikli bloklanmaydi.
  - [`xtask/src/main.rs`](../../xtask/src/main.rs) ruxsat etilgan bog‘liqlik qoidalariga `commandor-app -> rfd` qo‘shildi.
- **Ko‘p tillilik (i18n)**:
  - [uz/ru/en](../../resources/i18n/) resurslariga `nav-pick-folder`, `list-col-name`, `list-col-size`, `list-col-modified` kalitlari qo‘shildi. Barcha tillarda to‘liq sinxronlashtirildi.
- **Klaviatura navigatsiyasi va formatlash testlari**:
  - `format_bytes_correctly` va `format_timestamp_known_dates` sana/hajm formatlash sinovlari.
  - [`state.rs`](../../crates/commandor-ui/src/state.rs) da `step_selection_navigates_within_bounds` (yuqoriga/pastga o‘tish va chegaradan chiqib ketmaslik) hamda `open_selected_only_opens_directories` (fayl bosilganda ochilmaslik, faqat papka ochilishi) unit testlari yozildi.

## Tekshiruv

Muhit: macOS arm64; rustc/cargo 1.94.1.

| Tekshiruv | Natija |
|---|---|
| `cargo test --workspace` | **42/42 o‘tdi**, 0 failed/ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | O‘tdi (ogohlantirishsiz) |
| `cargo fmt --all -- --check` | O‘tdi |
| `cargo xtask check-boundaries` | OK — 8 crate (chegaralar buzilmagan) |
| `cargo run -p commandor-app -- --smoke` | **SMOKE OK**, exit 0 |

## Cheklov va qolgan ish

- Sinov faqat macOS arm64 da o‘tkazildi; Windows va Linux platformalarida native dialog va HiDPI ko‘rinishi hali tekshirilmagan.
- Windows file identity hozircha `None`ligicha qolmoqda (Unix inode asosidagi tekshiruv hozircha faqat Unixda).
- P3-02 (tablar, tarix/bookmarks, volume ro‘yxati, multi-select, buyruq paneli) navbatdagi vazifa hisoblanadi.
