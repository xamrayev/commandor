# Commandor — ishlar va hisobotlar

Yangilangan: 2026-11-09. Ilova **Rust**da yoziladi; Electron prototipi o‘chirildi ([P0-03](docs/reports/P0-03-rust-qayta-shakllantirish.md)). Platforma asosi, haqiqiy ikki panelli listing, saralash, tarix va multi-select lokal macOS arm64da ishlaydi ([P3-02a](docs/reports/P3-02a-saralash-tarix-multiselect.md), [P3-02b](docs/reports/P3-02b-gui-tahlil-va-tuzatishlar.md)); interfeys zamonaviy dizaynda qayta yaratildi ([P3-02c](docs/reports/P3-02c-zamonaviy-dizayn.md)); tomlar (F1/F2), panel tablari, xatcho‘plar va buyruqlar paneli qo‘shildi ([P3-08](docs/reports/P3-08-tomlar-f1-f2.md), [P3-02d](docs/reports/P3-02d-tablar-xatchoplar-buyruq-paneli.md)); fayllar tizim dasturida va "Open with" orqali ochiladi ([P3-09](docs/reports/P3-09-tizim-dasturida-ochish.md)); yangi papka, rename va savat reja → tasdiq → bajarish orqali ishlaydi ([P3-04a](docs/reports/P3-04a-papka-rename-savat.md)); nusxalash/ko‘chirish fon vazifasi sifatida progress va bekor qilish bilan ishlaydi ([P3-05a](docs/reports/P3-05a-nusxalash-kochirish.md)); uch OS CI GitHub’da ishga tushdi: build/clippy/test Ubuntu, Windows, macOS arm64 va Intelda yashil ([P1-05a](docs/reports/P1-05a-ci-birinchi-ishga-tushirish.md)); GUI smoke faqat lokal macOS arm64da. Repo: <https://github.com/xamrayev/commandor> (Apache-2.0).
Asosiy talablar: [texnik topshiriq](docs/TECHNICAL-SPEC.uz.md), [arxitektura](docs/ARCHITECTURE.uz.md), [kontraktlar](docs/CONTRACTS.uz.md), [qabul rejasi](docs/DELIVERY.uz.md).

Belgilar: `[x]` — dalili bor tugallangan vazifa; `[ ]` — ochiq. **Jarayonda** vazifa ham tekshirilmaguncha ochiq qoladi. Fazadagi bitta vazifa tugashi butun faza tayyorligini anglatmaydi.

## 0-faza — reja va ish tartibi

- [x] P0-01: R0–R3 talablarini fazalar va kuzatiladigan vazifalarga ajratish. [Hisobot](docs/reports/P0-01-rejalashtirish.md)
- [x] P0-02: Har ishga TODO ID, Markdown hisobot va o‘zaro havola tartibini belgilash. [Hisobot](docs/reports/P0-01-rejalashtirish.md)
- [x] P0-03: Imkoniyat va arxitekturani Rust uchun qayta shakllantirish; Electron kodini o‘chirish. [Hisobot](docs/reports/P0-03-rust-qayta-shakllantirish.md)

## 1-faza — Rust platforma asosi (R0.1)

- [x] P1-01: Cargo workspace, 1-faza crate’lari (core/broker/ui/app/ipc/i18n + xtask), `Cargo.lock`, `rustfmt`/workspace lintlari. MSRV 1.92 e’lon qilingan, faqat 1.94.1da sinalgan. [Hisobot](docs/reports/P1-rust-platforma.md)
- [x] P1-02: `commandor-ui` `clippy.toml` + `#![forbid]`, `cargo xtask check-boundaries`, `AppHandle` Command/Event kanali va eskirgan javob filtri. [Hisobot](docs/reports/P1-rust-platforma.md)
- [x] P1-03: `commandor-core` turlari (ID, Revision, AppError/ErrorCode, JobState, Command/Event) va unit testlar. Hali IO yo‘q, `tempfile` fixturelari P3da. [Hisobot](docs/reports/P1-rust-platforma.md)
- [x] P1-04: `commandor --worker` protokoli (freym, handshake, timeout, crash restart, restart limiti) va haqiqiy jarayon bilan supervisor testlari. [Hisobot](docs/reports/P1-rust-platforma.md)
- [x] P1-04a: Kutilmagan worker xabarlari javob timeoutini uzaytirishini tuzatish; regression testi. [Hisobot](docs/reports/P1-04a-worker-deadline.md)
- [ ] P1-04b: Worker stdin pipe yozuviga deadline/cancel; katta payload va o‘qimaydigan child bilan end-to-end timeout sinovi. [Aniqlangan cheklov](docs/reports/P1-04a-worker-deadline.md#chegara)
- [ ] P1-05: Windows/macOS/Linux CI: build, `cargo test`, egui oyna smoke; rusqlite FTS5 va portable-pty prototipi. **Jarayonda:** CI to‘rt runnerda yashil ([P1-05a](docs/reports/P1-05a-ci-birinchi-ishga-tushirish.md)); runnerlarda GUI smoke, FTS5 va portable-pty prototiplari qolgan. [Hisobot](docs/reports/P1-rust-platforma.md)
- [x] P1-05a: Public GitHub repo, uch tilli README, Apache-2.0, CI macOS runnerlarini yangilash; fmt/clippy/test/check-boundaries ubuntu-24.04, windows-2022, macos-15, macos-15-intel’da o‘tdi. [Hisobot](docs/reports/P1-05a-ci-birinchi-ishga-tushirish.md)
- [ ] P1-06: fluent i18n (uz asosiy, ru/en tuzilma) va `tracing` redaction qatlami. **Jarayonda:** i18n tayyor va testlangan; `tracing` redaction yo‘q. [Hisobot](docs/reports/P1-rust-platforma.md)

Chiqish mezoni: uch OSda oyna ochiladi, UI chegarasi lint bilan majburlanadi, worker crash UIni yiqitmaydi.

## 2-faza — asosiy texnik prototiplar (R0.2–R0.4)

- [ ] P2-01: LibreOffice runtime probe; private profil, macro/external-link bloklash va snapshot → PDF.
- [ ] P2-02: DOCX/XLSX/PPTX working-copy edit/save; original revision va recovery prototipi.
- [ ] P2-03: LibreOfficeKit FFI → egui tekstura spike; uch OS feasibility va yozma go/no-go qarori (OFF-07).
- [ ] P2-04: ropey+syntect editor spike: IME, 10 MB fayl, BOM/EOL round-trip; egui yetarliligi (ADR-001 qayta ko‘rish sharti).
- [ ] P2-05: PDFium vs hayro/MuPDF render solishtiruvi; lopdf sahifa amallari fixturelari; litsenziya qarori.
- [ ] P2-06: Pinlangan Qwen GGUF/llama.cpp; RAM/latency va o‘zbekcha sifat benchmarki.

Chiqish mezoni: runtime/format/OS bo‘yicha real dalillar; tekshirilmagan imkoniyatlar ochiq qoladi.

## 3-faza — ikki panelli fayl menejeri (R1.1)

- [x] P3-01: Mustaqil panellar, yo‘l/picker, yuqoriga, reload, filtr, yashirin fayllar va xato holatlari (UX-01/03/04/06/07, FS-01/07). [Hisobot P3-01a](docs/reports/P3-01-rust-listing.md), [Hisobot P3-01b](docs/reports/P3-01b-ui-metadata-va-picker.md)
- [x] P3-01a: `commandor-fs`, file thread, typed listing, ikki mustaqil panel, yo‘l/home/up/reload, filtr/hidden, cheklangan virtual ro‘yxat, error/retry; temp fixturelar va GUI smoke. [Hisobot](docs/reports/P3-01-rust-listing.md)
- [x] P3-01b: Native OS papka picker, sana/metadatani UI’da to‘liq ko‘rsatish, klaviatura/focus/HiDPI vizual qabul sinovi; Windows file identity tekshiruvi. [Hisobot](docs/reports/P3-01b-ui-metadata-va-picker.md)
- [ ] P3-02: Tablar, tarix/bookmarks, volume ro‘yxati, multi/range select, saralash, buyruq paneli (UX-08). **Jarayonda:** saralash, tarix, multi-select ([P3-02a](docs/reports/P3-02a-saralash-tarix-multiselect.md), [P3-02b](docs/reports/P3-02b-gui-tahlil-va-tuzatishlar.md)), tomlar ([P3-08](docs/reports/P3-08-tomlar-f1-f2.md)), tablar, sessiya xatcho‘plari va buyruq paneli ([P3-02d](docs/reports/P3-02d-tablar-xatchoplar-buyruq-paneli.md)), Shift bilan oraliq tanlash ([P3-02e](docs/reports/P3-02e-oraliq-tanlash.md)), xatcho‘p va tablarni saqlash ([P3-07a](docs/reports/P3-07a-sozlamalar-sessiya.md)) tayyor; GUI’da to‘liq qabul sinovi qolgan.
- [x] P3-02a: Ustunlar bo‘yicha saralash (o‘sish/kamayish), ko‘p tanlash (Space/Insert, Ctrl+Click, hajm/son statistikasi), navigatsiya tarixi (orqaga/oldinga va Alt+Left/Right). [Hisobot](docs/reports/P3-02a-saralash-tarix-multiselect.md)
- [x] P3-02b: GUI tahlili va tuzatishlari (saralashda tanlovni saqlash, tarixda mutlaq yo‘l, GiB/TiB format, ustunlarni tekislash, enter fokusini bo‘shatish). [Hisobot](docs/reports/P3-02b-gui-tahlil-va-tuzatishlar.md)
- [x] P3-02c: Interfeysni zamonaviy dizaynda qayta yaratish: dark/light palitra va `theme.rs`, vektor ikonkalar, panel kartalari, maxsus chizilgan jadval, holat bloklari, F-tugma chiplari. Light mavzu va boshqa OSlarda vizual sinalmagan. [Hisobot](docs/reports/P3-02c-zamonaviy-dizayn.md)
- [x] P3-02d: Panel tablari (Cmd/Ctrl+T/W, Ctrl+Tab), sessiya xatcho‘plari (★, Cmd/Ctrl+D), buyruqlar paneli (Cmd/Ctrl+Shift+P) va yagona yorliqlar jadvali, PageUp/Down/Home/End va kursorga avto-scroll. Saqlash P3-07da; ★ popup va dark mavzu skrinshotda ko‘rilmagan. [Hisobot](docs/reports/P3-02d-tablar-xatchoplar-buyruq-paneli.md)
- [x] P3-02e: Oraliq tanlash: Shift+strelka/PageUp/PageDown/Home/End va Shift+bosish, tayanch va avvalgi tanlovni saqlash. 132 test; GUI’da sinalmagan. [Hisobot](docs/reports/P3-02e-oraliq-tanlash.md)
- [x] P3-08: Tomlar ro‘yxati (`sysinfo`, OS mount filtrlari), tom chiplari va F1/F2 popup; `check-boundaries` qoidasi tuzatildi. Linux/Windows faqat sintetik testda. [Hisobot](docs/reports/P3-08-tomlar-f1-f2.md)
- [x] P3-09: Faylni tizimdagi standart dasturda ochish (Enter/ikki marta bosish) va "Open with" (Shift+Enter, o‘ng tugma, palette): yangi `commandor-launch` crate, launcher thread, bajariladigan fayl tasdig‘i. 101 test; GUI vizual sinovi va real ishga tushirish qilinmagan (ekran qulflangan), Linux/Windows faqat CI. [Hisobot](docs/reports/P3-09-tizim-dasturida-ochish.md)
- [ ] P3-03: Oqimli listing va virtual jadval; 10 000 yozuv benchmarki, `notify` watcher/reconcile.
- [ ] P3-04: Brokerga tegishli copy/move/rename/mkdir/trash rejalari, konflikt UI va tasdiq. **Jarayonda:** reja → tasdiq → bajarish protokoli, mkdir/rename/trash ([P3-04a](docs/reports/P3-04a-papka-rename-savat.md)), copy/move rejasi va batch konflikt qarori ([P3-05a](docs/reports/P3-05a-nusxalash-kochirish.md)) tayyor; per-item konflikt tanlovi, doimiy o‘chirish va bo‘sh fayl yaratish qolgan.
- [x] P3-04a: Yangi papka (F7), rename (Shift+F6, case-only), savat (F8/Delete, tasdiq bilan; savat bo‘lmasa o‘chirmaydi); OS nom qoidalari, reja xeshi/muddati/inode tekshiruvi, `trash` crate. 115 test; oynalar va haqiqiy savat GUI’da sinalmagan, rename TOCTOU oynasi hujjatlashtirilgan. [Hisobot](docs/reports/P3-04a-papka-rename-savat.md)
- [ ] P3-05: Copy worker navbati, progress/cancel, idempotency, per-item jurnal va tiklash. **Jarayonda:** navbat, progress, bekor qilish, `.part` orqali yozish va disklararo ko‘chirish tayyor ([P3-05a](docs/reports/P3-05a-nusxalash-kochirish.md)); idempotency kaliti, jurnal/tiklash (FS-08), xattr/ACL, parallel navbat qolgan.
- [x] P3-05a: F5/F6 nusxalash/ko‘chirish: fon vazifasi (`commandor-jobs`), progress ≤10/s, bekor qilish, skip/keep both/replace, `.part`+`sync_all`+rename, symlink kuzatilmaydi, disklararo ko‘chirish hajm tekshiruvidan keyin manbani o‘chiradi. 129 test (disklararo test haqiqiy ikki qurilmada); oynalar GUI’da sinalmagan. [Hisobot](docs/reports/P3-05a-nusxalash-kochirish.md)
- [ ] P3-06: Unicode, symlink/hardlink, case-only rename, disk full va cross-volume testlari.
- [ ] P3-07: TOML sozlamalar va klaviatura xaritasi (UX-09); xatcho‘plar va ochiq tablarni saqlash (P3-02d davomi). **Jarayonda:** `settings.toml`, xatcho‘p/sessiya saqlash va tiklash tayyor ([P3-07a](docs/reports/P3-07a-sozlamalar-sessiya.md)); klaviatura xaritasi va til tanlash qolgan.
- [x] P3-07a: `commandor-settings` (OS konfiguratsiya papkasi, TOML, atomik saqlash, buzilgan fayl → `.broken` + sukut qiymatlar), sessiya/xatcho‘plarni tiklash va avto-saqlash; CI tuzatishi (ext4 inode qayta ishlatilishi → fayl fingerprint; Windows dead code); lokal Windows/Linux clippy. 145 test; qayta ochish GUI’da sinalmagan. [Hisobot](docs/reports/P3-07a-sozlamalar-sessiya.md)

Chiqish mezoni: UX/FS talablari va A01–A03; data-loss bo‘yicha ochiq kritik xato yo‘q.

## 4-faza — hujjatlar va tashqi terminal (R1.2–R1.3)

- [ ] P4-01: `commandor-editor`: UTF-8/BOM/EOL, highlight, find/replace, undo/redo, dirty state va Save As.
- [ ] P4-02: Revision conflict, staging/fsync/backup/commit, crash recovery.
- [ ] P4-03: Native Markdown preview (HTMLsiz); PDFium sahifalari/zoom/qidiruv va xato holatlari.
- [ ] P4-04: Office preview cache, lazy native edit, working-copy apply va invalidate.
- [ ] P4-05: Har OS uchun tanlangan papkada tizim terminalini ochish (argument massivi); `commandor-launch` crate’ida ([P3-09](docs/reports/P3-09-tizim-dasturida-ochish.md)).
- [ ] P4-06: TXT/PDF/OFF/TERM qabul testlari; R1 offline demo.

## 5-faza — lokal indeks va qidiruv (R2.1)

- [ ] P5-01: rusqlite FTS5 sxema/migratsiya, yagona writer thread va crash recovery.
- [ ] P5-02: Tanlangan roots/excludes, workerdagi extractorlar, chunklar va provenance.
- [ ] P5-03: Watcher/reconciliation, offline/stale holatlar va purge/GC.
- [ ] P5-04: Filtrli qidiruv UI, o‘zbekcha normalizatsiya, 100 000 hujjat benchmarki.

## 6-faza — AI, modellar va amal rejalari (R2.2–R2.3)

- [ ] P6-01: Ollama/compatible `Provider` adapterlari, probe/models, streaming/cancel va `keyring`.
- [ ] P6-02: HF manifest pin/hash/license, download/resume/checksum va model katalogi.
- [ ] P6-03: Owned llama-server start/health/stop, loopback va resurs holatlari.
- [ ] P6-04: Local-only summary, chunk coverage, provenance va stale invalidation.
- [ ] P6-05: AI search/tool schema, broker validation, tasdiqlanadigan reja va audit.
- [ ] P6-06: Redirect/prompt injection/plan drift/idempotency va model sifat sinovlari.

## 7-faza — kengaytirilgan tahrir va ichki terminal (R2.4)

- [ ] P7-01: PDF annotation/page operations va Save As fixturelari.
- [ ] P7-02: Ichki PTY (`portable-pty` + `alacritty_terminal`), tab/resize/cancel va owned process cleanup.
- [ ] P7-03: YAML/JSON diagnostics, katta fayl/encoding imkoniyatlari.

## 8-faza — embedded Office va reliz (R3)

- [ ] P8-01: P2-03 qarori asosida embedded Office; uch OS focus/IME/clipboard/recovery dalillari.
- [ ] P8-02: AccessKit accessibility, klaviatura boshqaruvi va ru/en tarjimalari.
- [ ] P8-03: Performance (NFR-01/04/05), resurs kvotalari, offline/security va to‘liq regressiya.
- [ ] P8-04: Installer (`cargo-packager`/`cargo-dist`), signing/notarization, update/migration, SBOM va litsenziya inventari.
- [ ] P8-05: Foydalanish/recovery qo‘llanmasi, release notes va ma’lum cheklovlar.

## Keyingi imkoniyatlar

- [ ] P9-01: OCR va semantik/vektor qidiruv.
- [ ] P9-02: Arxivlar va masofaviy filesystem adapterlari.

## Arxiv

Electron prototipi bo‘yicha oldingi ishlar (eski P1-01–03, P3-01) kod bilan birga bekor qilindi: [P1 hisoboti](docs/reports/P1-platforma.md), [P3-01 hisoboti](docs/reports/P3-01-ikki-panel.md). Ular tarixiy qayd; yangi Rust vazifalari yuqorida noldan ochiq.

## Hisobot yuritish tartibi

1. Ish boshlanganda IDni **Jarayonda** deb belgilang.
2. Har tugallangan vazifa yoki mazmunli oraliq ish uchun `docs/reports/ID-qisqa-nom.md` yarating; bir nechta bog‘liq ID bitta hisobotga bog‘lanishi mumkin.
3. [Shablon](docs/reports/TEMPLATE.md) bo‘yicha o‘zgarishlar, tekshiruv natijasi, cheklovlar va keyingi qadamni yozing.
4. TODO bandidan hisobotga, hisobotdan TODOga va tegishli kodga nisbiy havola qo‘ying.
5. Faqat bajarilgan va tekshirilgan kichik vazifani `[x]` qiling. Test bajarilmagan bo‘lsa sababini yozing; taxminni natija sifatida ko‘rsatmang.
6. Har ish sessiyasi oxirida TODO sanasi va hisobotni yangilang; foydalanuvchiga TODO va yangi hisobot havolalarini bering.
