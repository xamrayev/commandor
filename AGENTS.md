# Commandor ish tartibi

- Talablar: `docs/TECHNICAL-SPEC.uz.md`; yechim: `docs/ARCHITECTURE.uz.md` va `docs/CONTRACTS.uz.md`.
- Ish holatining asosiy manbasi: `TODO.md`. 2026-11-07dan ilova Rustda (Cargo workspace, egui) yoziladi; Electron/TypeScript kodi qaytarilmaydi.
- Har tugallangan vazifa yoki mazmunli oraliq ish uchun `docs/reports/`da kichik Markdown hisobot yarating/yangilang. `TEMPLATE.md`dan foydalaning.
- TODO bandidan hisobotga, hisobotdan TODO va kodga havola qo‘ying. Faqat dalil bilan tugagan vazifalarni belgilang.
- Har sessiya oxirida TODO sanasi va holatini yangilang, javobda TODO hamda yangi hisobot havolalarini bering. Hujjat va foydalanuvchi bilan muloqot tili — o‘zbekcha.
- `commandor-ui` crate’i `std::fs`, `std::process` va tarmoqqa bevosita kirmaydi; faqat `commandor-core` turlari va broker `AppHandle` orqali ishlaydi. Broker har command argumentini va grantini tekshiradi.
- Ishonchsiz kontent parserlari (PDF, OOXML, LibreOffice) `commandor --worker` jarayonida; tashqi dasturlar faqat argument massivi bilan, shell orqali emas.
- Tekshiruvlarda `tempfile` bilan sintetik vaqtinchalik fayllardan foydalaning. Shaxsiy diskni test fixture sifatida ishlatmang.
- `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` va `cargo test --workspace` natijasini hisobotga yozing.
- Sinovdan o‘tmagan OS yoki imkoniyatni tayyor deb ko‘rsatmang; native Office embedded deb nomlanmaydi.
