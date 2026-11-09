# P1-01–06 — Rust platforma asosi

- Sana: 2026-11-07
- Holat: P1-01–04 bajarildi; P1-05 va P1-06 jarayonda
- TODO: [1-faza](../../TODO.md#1-faza--rust-platforma-asosi-r01)
- Talablar: R0.1 infratuzilmasi; NFR-06 (worker crash izolyatsiyasi) va UX-06 (i18n tuzilmasi) boshlang‘ich qismi. UX/FS talablari hali yopilmagan.

## Qilingan ish

- **Workspace** ([Cargo.toml](../../Cargo.toml), `Cargo.lock`, [rustfmt.toml](../../rustfmt.toml)): edition 2024, umumiy lintlar (`unsafe_code = forbid`, clippy `all`, `unwrap_used`, `print_stdout`). GUI: `eframe 0.35.0` (sukut bo‘yicha wgpu + AccessKit). 0.36/0.37 Rust 1.95 talab qiladi, lokal toolchain 1.94.1 — shuning uchun 0.35 tanlandi.
- [commandor-core](../../crates/commandor-core/src/lib.rs): opaque IDlar, `Revision` (hash faqat ikkala tomonda bo‘lsa konflikt isboti), 24 ta `ErrorCode` va fluent kaliti, `JobState` holat mashinasi, `Command`/`Response`/`Event`/`Envelope`.
- [commandor-broker](../../crates/commandor-broker/src/lib.rs): alohida thread, `AppHandle::dispatch` (bloklamaydi), `EventReceiver::drain`, API versiya tekshiruvi, GUIga bog‘lanmagan `Notifier`.
- [commandor-ui](../../crates/commandor-ui/src/lib.rs): ikki panel joyi, Tab bilan faol panel, o‘chiq F3–F8, broker holati. [Holat mantiqi](../../crates/commandor-ui/src/state.rs) GUIsiz testlanadi; boshqa `RequestId` bilan kelgan javob qabul qilinmaydi.
- **UI chegarasi:** [clippy.toml](../../crates/commandor-ui/clippy.toml) std fs/process/net turlari va metodlarini taqiqlaydi; `#![forbid]` ularni `allow` bilan yoqishga yo‘l qo‘ymaydi. [xtask](../../xtask/src/main.rs) `check-boundaries` har crate’ning bevosita bog‘liqliklarini ruxsat ro‘yxati bilan solishtiradi.
- [commandor-ipc](../../crates/commandor-ipc/src/lib.rs): 4 bayt LE + JSON freym (≤ 16 MiB, katta uzunlik xotira ajratilishidan oldin rad etiladi), `Hello` handshake, supervisor (timeoutda kill, crashdan keyin restart, restart limiti). Diagnostika so‘rovlari (`Sleep`, `Abort`) faqat `COMMANDOR_WORKER_DIAG=1` bilan bajariladi.
- [commandor-app](../../crates/commandor-app/src/main.rs): bitta `commandor` binary — GUI, `--smoke`, `--worker diag`; noto‘g‘ri argumentda chiqish kodi 2.
- [commandor-i18n](../../crates/commandor-i18n/src/lib.rs) va [resources/i18n](../../resources/i18n/): uz (asosiy), ru, en; kalit topilmasa uz, keyin kalitning o‘zi qaytadi.
- [CI](../../.github/workflows/ci.yml): Ubuntu 24.04, macOS 14 (arm64), macOS 13 (x64), Windows 2022 uchun fmt/clippy/test/xtask — **yozilgan, ishga tushirilmagan** (loyiha git repozitoriy emas, remote yo‘q). *Yangilanish 2026-11-09:* CI GitHub’da ishga tushdi, macOS runnerlari `macos-15`/`macos-15-intel`ga almashtirildi, to‘rtala OS yashil — [P1-05a](P1-05a-ci-birinchi-ishga-tushirish.md).

## Tekshiruv

Muhit: macOS arm64, rustc/cargo 1.94.1.

| Tekshiruv | Natija |
|---|---|
| `cargo test --workspace` | 31/31 o‘tdi: core 11, broker 3, i18n 4, ipc 4, ui 3, haqiqiy worker jarayoni bilan 6 ta integratsiya testi; 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | O‘tdi (bitta `single_match` tuzatildi) |
| `cargo fmt --all -- --check` | O‘tdi (birinchi `cargo fmt`dan keyin) |
| `cargo xtask check-boundaries` | `OK (7 crate)`. Salbiy sinov: UIga vaqtincha `commandor-ipc` qo‘shilganda exit 1 bilan rad etdi, keyin qaytarildi |
| UI lint salbiy sinovi | Vaqtincha qo‘shilgan `std::fs::read`, `Path::exists`, `process::Command` 3 ta xato berdi; `#[allow]` qo‘shilganda E0453 (forbid). Sinov kodi o‘chirildi |
| `commandor --smoke` (debug va release) | `SMOKE OK`, exit 0: oyna ochildi, broker `AppInfo` qaytardi, oyna o‘zi yopildi |
| Release o‘lchovi (bitta mashina, NFR baseline emas) | Binary 11.3 MB; 6 s idle RSS ≈ 111 MB; `--smoke` jarayoni boshidan oxirigacha 0.17–0.19 s (5 marta) |

Worker testlari: ping/Unicode echo, diagnostika flagisiz rad etish, `abort()` → `Crashed` → keyingi so‘rovda restart, 10 s osilish → 300 ms timeout va almashtirish (< 3 s), restart limiti, noma’lum worker turi.

Tekshirilmagan: oynaning vizual ko‘rinishi (ekran yozishga ruxsat yo‘qligi sabab skrinshot olinmadi), o‘zbekcha ‘ belgisining chizilishi, Windows/Linux/macOS x64, MSRV 1.92, AccessKit/screen reader, HiDPI.

## Cheklov va qolgan ish

- P1-05: ~~CI’ni real runnerlarda ishga tushirish~~ — bajarildi ([P1-05a](P1-05a-ci-birinchi-ishga-tushirish.md)); runnerlarda GUI smoke, rusqlite FTS5 va portable-pty prototiplari qolgan.
- P1-06: `tracing` va redaction qatlami (kalit/prompt/fayl mazmuni logga tushmasligi).
- NFR-05 maqsadi (≤ 150 MB) hozircha mos, lekin bo‘sh oyna uchun o‘lchandi; ishlaydigan funksiyalar qo‘shilgach qayta o‘lchanadi.
- Keyingi faza: [P2](../../TODO.md#2-faza--asosiy-texnik-prototiplar-r02r04) R0 spike’lari yoki [P3-01](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11) haqiqiy papka listingi.
