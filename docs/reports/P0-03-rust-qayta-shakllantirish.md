# P0-03 — Rust uchun qayta shakllantirish

- Sana: 2026-11-07
- Holat: bajarildi
- TODO: [P0-03](../../TODO.md#0-faza--reja-va-ish-tartibi)
- Talablar: barcha IDlar saqlandi; UX-08/UX-09 qo‘shildi; NFR-01/04/05 maqsadlari yangilandi

## Qilingan ish

Foydalanuvchi topshirig‘i: imkoniyat va arxitekturani qayta shakllantirish, barcha kodni o‘chirish, ilovani Rustda yozish.

- **Kod o‘chirildi:** `src/`, `tests/`, `scripts/`, `dist/`, `node_modules/`, `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`. Loyiha git repozitoriy emasligi sababli o‘chirishdan oldin vaqtinchalik arxiv sessiya scratchpad’iga olindi (node_modules’siz); u doimiy saqlanmaydi.
- [Arxitektura](../ARCHITECTURE.uz.md) v2.0: Rust + egui/eframe (ADR-001), crate-darajali vakolat chegarasi va `--worker` jarayoni (ADR-005), PDFium (ADR-010), o‘z editori (ADR-011), workspace/crate tuzilmasi, UI↔broker Command/Event oqimi, LibreOfficeKit FFI gipotezasi.
- [Kontraktlar](../CONTRACTS.uz.md) v2.0: TypeScript turlari o‘rniga `Command`/`Reply`/`AppError`/`Revision`, `Provider`, `OfficeRenderer`/`OfficeEditor`, `PdfBackend` traitlari; worker protokoli qoidalari.
- [Texnik topshiriq](../TECHNICAL-SPEC.uz.md) v2.0: Rustga o‘tish bilan imkoniyatlar o‘zgarishi jadvali (1.1.1), UX-08 buyruq paneli, UX-09 TOML sozlama/klaviatura xaritasi, NFR-01 ≤ 1.5 s, NFR-04 UI thread kadr byudjeti, NFR-05 ≤ 150 MB.
- [Qabul rejasi](../DELIVERY.uz.md) v2.0: R0.1/R0.4 Rust spike’lari, egui riski, mehnat taxmini 43–75+ kishi-hafta.
- [TODO](../../TODO.md): fazalar Rust vazifalari bilan qayta tuzildi; eski Electron vazifalari arxivga o‘tkazildi. [README](../../README.md), [AGENTS.md](../../AGENTS.md), `.gitignore` yangilandi.

## Tekshiruv

- Loyiha ildizida faqat `AGENTS.md`, `README.md`, `TODO.md`, `.gitignore` va `docs/` qolgani `ls` bilan tekshirildi.
- Muhit: macOS, `cargo 1.94.1`, `rustc 1.94.1` mavjud. Rust kodi hali yozilmagani uchun build/test bajarilmadi.
- Crate tanlovlari faqat nomzod: versiya, litsenziya va uch OS ishlashi hali tekshirilmagan.

## Cheklov va qolgan ish

GUI toolkit (egui) va PDF backend R0 spike’lari (P2-04, P2-05) bilan tasdiqlanishi kerak. Keyingi qadam: [P1-01](../../TODO.md#1-faza--rust-platforma-asosi-r01) — Cargo workspace skeleti va `Cargo.lock`.
