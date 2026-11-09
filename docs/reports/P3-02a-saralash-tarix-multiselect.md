# P3-02a — Saralash, tarix va multi-selection

- Sana: 2026-11-07
- Holat: P3-02a bajarildi; umumiy P3-02 jarayonda
- TODO: [3-faza](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-03 (orqaga/oldinga), UX-04 (multi-select, saralash), FS-01

## Qilingan ish

- **Ustunlar bo‘yicha saralash (Sorting)**:
  - [`commandor-core/src/filesystem.rs`](../../crates/commandor-core/src/filesystem.rs) da `SortField` (`Name`, `Size`, `Modified`), `SortDirection` (`Ascending`, `Descending`) va `SortCriteria` turlari qo‘shildi; `ListOptions` tarkibiga kiritildi.
  - [`commandor-fs/src/lib.rs`](../../crates/commandor-fs/src/lib.rs) da fayl tizimi listingida saralash joriy etildi: papkalar doim yuqorida (`directories-first`), qolgan yozuvlar belgilangan ustun va yo‘nalish bo‘yicha saralanadi.
  - [`commandor-ui/src/lib.rs`](../../crates/commandor-ui/src/lib.rs) da jadval sarlavhalari (`Nom`, `Hajm`, `O‘zgartirilgan`) bosiladigan tugmalarga aylantirildi; faol saralash ustuni va yo‘nalishi `▲` / `▼` ko‘rsatkichlari bilan aks ettiriladi.
  - Fayl va papkalar nomlari hamda "Nom" sarlavhasi tugma ichida markazdan chapga tekislandi (`.left_text(...)`), natijada barcha nomlar Total Commander kabi chap tomondan chiroyli tiziladi.
- **Ko‘p tanlash (Multi-select) va statistika**:
  - [`commandor-ui/src/state.rs`](../../crates/commandor-ui/src/state.rs) da `cursor: Option<EntryId>` (fokuslangan element) va `selected: BTreeSet<EntryId>` (belgilangan elementlar to‘plami) ajratildi.
  - `Space` yoki `Insert` tugmasi kursordagi elementni belgilaydi/bekor qiladi va avtomatik pastga qadam tashlaydi (`select_current_and_step`).
  - `Ctrl` / `Cmd` + sichqoncha bilan bosganda alohida elementlar tanlovga qo‘shiladi/olib tashlanadi.
  - Belgilangan elementlar `☑` belgisi va sariq/ogohlantiruvchi rang bilan ajratiladi.
  - Tanlangan fayllar soni va umumiy hajmi sarlavhada dinamik ko‘rsatiladi (`list-selected` — masalan: `· Tanlangan: 3 ta (12.4 KiB)`).
- **Navigatsiya tarixi (History: Orqaga / Oldinga)**:
  - `PanelState` da `history: Vec<DirectoryTarget>` va `history_index: usize` orqali to‘liq orqaga/oldinga navbat saqlandi.
  - Panellar panelida "Orqaga" va "Oldinga" tugmalari joriy etildi; mavjud bo‘lmaganda tugmalar nofaol bo‘ladi.
  - Klaviatura tezkor tugmalari: `Alt + Chapga` (orqaga) va `Alt + O‘ngga` (oldinga).
- **Ko‘p tillilik (i18n)**:
  - [uz/ru/en](../../resources/i18n/) resurslariga `nav-back`, `nav-forward`, `list-selected` kalitlari qo‘shildi.
- **Unit testlar**:
  - `commandor-fs` da `sorting_by_name_size_and_modified` (kamayish va o‘sish tartibida saralash).
  - `commandor-ui` da `multi_selection_and_stats`, `history_navigation_back_and_forward`, `toggle_sort_changes_field_and_direction`.

## Tekshiruv

Muhit: macOS arm64; rustc/cargo 1.94.1.

| Tekshiruv | Natija |
|---|---|
| `cargo test --workspace` | **46/46 o‘tdi**, 0 failed/ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | O‘tdi (ogohlantirishsiz) |
| `cargo fmt --all -- --check` | O‘tdi |
| `cargo xtask check-boundaries` | OK — 8 crate (chegaralar buzilmagan) |
| `cargo run -p commandor-app -- --smoke` | **SMOKE OK**, exit 0 |

## Cheklov va qolgan ish

- P3-02 ning qolgan qismlari: tablar (har panelda mustaqil tablar), bookmarks (xatcho‘plar), volume ro‘yxati (disklarni tanlash) va buyruq paneli (`Ctrl/Cmd+Shift+P` command palette — UX-08).
- Range-select (`Shift + Click` orqali oraliqni birdaniga belgilash) keyingi qadamda qo‘shiladi.
