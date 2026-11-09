# P3-01a — Rust’da haqiqiy ikki panelli listing va navigatsiya

- Sana: 2026-11-07
- Holat: P3-01a bajarildi; umumiy P3-01 jarayonda
- TODO: [3-faza](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-01/03/04/06/07, FS-01/07ning boshlang‘ich qismi; bu IDlar to‘liq yopilmadi

## Ko‘rib chiqilgan o‘zgarishlar

Loyiha Rust + egui Cargo workspace’iga o‘tgan. [Platforma hisoboti](P1-rust-platforma.md), Command/Event brokeri, UI state, i18n, crate chegaralari, worker supervisor va uning haqiqiy jarayon testlari ko‘rib chiqildi. Eski Electron prototipi qaytarilmadi. Git repozitoriy yo‘q; baholash joriy kod va hisobotlar asosida qilindi, commit diff asosida emas.

## Qilingan ish

- [commandor-fs](../../crates/commandor-fs/src/lib.rs): read-only listing; real fayl nomi/turi/hajmi/mtime; directories-first saralash, case-insensitive nom filtri va hidden siyosati. Unixda dotfiles, Windows kodida hidden atributi ham hisobga olinadi.
- [Core shartnoma](../../crates/commandor-core/src/filesystem.rs): `DirectoryTarget`, `ListOptions`, `DirectoryListing`, `Location`, `FileEntry`. UI haqiqiy nom/manzilni ko‘radi; navigatsiya servis bergan ID orqali. Raw `OsString` va `PathBuf` servisda qoladi.
- [Broker](../../crates/commandor-broker/src/lib.rs): listing uchun alohida `commandor-files` thread va 32 ta so‘rovli navbat. UI va broker file IO kutmaydi; to‘lgan navbat `RateLimited` qaytaradi.
- Grantlar canonical path va Unix file identity bilan qayta tekshiriladi. Symlink entry kuzatilmaydi; user kiritgan aniq path yangi grant yaratadi. Bir panel listingi boshqasining IDlarini darhol bekor qilmaydi. 128 listing granti saqlanadi; eskirgan ID `StaleReference` qaytaradi.
- [egui interfeysi](../../crates/commandor-ui/src/lib.rs): ikki panel, path/home/up/reload, filter/hidden, tanlov, double-click/Enter, ↑/↓/Backspace/Tab, loading/empty/error/retry. `ScrollArea::show_rows` faqat ko‘rinadigan qatorlarni chizadi.
- [UI state](../../crates/commandor-ui/src/state.rs): har panelning alohida pending RequestId’i bor; kechikkan eski javob yangi listingni bosmaydi. Xatoda oxirgi muvaffaqiyatli location qayta urinish uchun saqlanadi.
- [uz/ru/en resurslari](../../resources/i18n/): barcha yangi UI matnlari uch tilda.
- [Smoke](../../crates/commandor-app/src/main.rs): shaxsiy kataloglar o‘rniga `tempfile` fixture; broker va ikkala listing tayyor bo‘lgach yopiladi. Tayyor bo‘lmasa 10 soniyadan keyin muvaffaqiyatsiz tugaydi.

## Tekshiruv

Muhit: macOS arm64; cargo/rustc 1.94.1. Yangi tashqi crate yuklanmadi; test/build lokal cache bilan `--offline` ishladi.

| Tekshiruv | Natija |
|---|---|
| `cargo test --workspace --offline` | **40/40 o‘tdi**, 0 failed/ignored; supervisor deadline tuzatishi ham shu hisobda |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | O‘tdi |
| `cargo fmt --all -- --check` | O‘tdi |
| `cargo xtask check-boundaries` | OK — 8 crate |
| `cargo run -p commandor-app --offline -- --smoke` | **SMOKE OK**, exit 0; haqiqiy egui oyna + ikkala listing |

Yangi qamrov: [5 filesystem testi](../../crates/commandor-fs/tests/listing.rs), [broker round-trip testi](../../crates/commandor-broker/tests/files.rs), 2 UI state testi; qo‘shimcha 1 [supervisor regression testi](P1-04a-worker-deadline.md). 5 001 faylli fixture listing limiti hamda filtr bilan chiqarilmagan yozuvni topishni tekshiradi.

Noto‘g‘ri UTF-8 nom fixture’i dastlab APFSda `EPERM` berdi: APFS bunday nomni yaratishga ruxsat bermaydi. Shu disk testi Linuxga cheklab belgilandi; **Linuxda hali ishga tushirilmagan**. Bu test lokal o‘tdi degan da’vo yo‘q.

## Cheklov va keyingi ish

- P3-01b: native OS picker, sana/metadatani UI’da to‘liq ko‘rsatish va real klaviatura/focus/HiDPI qabul sinovi. Hozir mtime servisdan keladi, UI nom/tur/hajmni ko‘rsatadi.
- P3-03: listing ≤ 5 000 mos yozuv, skan ≤ 100 000 yozuv; limitga yetsa UI ochiq bildiradi. Oqimli cursor, cancellation, watcher, katta katalog performance sinovi hali yo‘q. Sekin disk bitta file thread’ini band qilishi va shutdownni kechiktirishi mumkin.
- UI smoke startup/render va holatga dalil; pixel/screenshot, native klaviatura click-through, screen reader va barcha UX ssenariylariga dalil emas.
- Windows/Linux/macOS x64 sinalmagan. Windows file identity hozir `None`; Unixdagidek almashtirilgan inode tekshiruvi u platformada hali yo‘q. Canonical path qayta tekshiruvi barcha platformalar kodida bor.
- Grantlar mutatsiya uchun tayyor ruxsat modeli emas: tashqi jarayon bilan TOCTOU poygalariga qarshi mutlaq kafolat yo‘q. Copy/move/save qo‘shishdan oldin handle/revision siyosati alohida tekshiriladi.
- P3-02/P3-04: multi-select/tab/tarix/bookmark/volume va fayl o‘zgartirish amallari ochiq. P1-05/06 va P2 spike’lari ham ochiq qoladi.
