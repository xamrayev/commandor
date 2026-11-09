# Commandor

[English](README.md) · [Русский](README.ru.md) · **O‘zbekcha**

**Rustda yozilgan kross-platform, ikki panelli fayl menejeri.**

> **Holat: dastlabki ishlab chiqish (v0.1.0, 2026-yil noyabr).** Hozircha papkalarni ko‘rish, fayllarni boshqa dasturlarda ochish, yangi papka, nomini o‘zgartirish, savat, nusxalash va ko‘chirish ishlaydi. Viewer, editor, Office, terminal, qidiruv indeksi va lokal AI ham hali qilinmagan. Ilova faqat **macOS arm64**da ishga tushirib ko‘rilgan. Windows va Linux uchun CI yozilgan, lekin hali ishga tushirilmagan.

## Hozir nima ishlaydi

- Haqiqiy papkalarni ko‘rsatadigan ikki mustaqil panel. Har panelda yo‘l kiritish, uy/yuqoriga/yangilash, orqaga/oldinga tarix, nom filtri va yashirin fayllar bor.
- **Har panelda tablar.** Har tabning o‘z papkasi, tarixi, tanlovi va scroll holati bor.
- **Tomlar ro‘yxati.** Disklar yo‘l ustida chip ko‘rinishida chiqadi. F1 chap panel, F2 o‘ng panel uchun ro‘yxatni ochadi.
- **Buyruqlar paneli.** Cmd/Ctrl+Shift+P barcha panel buyruqlari va xatcho‘plarni qidiruv bilan ko‘rsatadi.
- **Xatcho‘plar** ★ tugmasi yoki Cmd/Ctrl+D bilan. Hozircha faqat shu sessiyada saqlanadi.
- **Fayllarni ochish** tizimdagi standart dasturda (Enter yoki ikki marta bosish). **Dastur bilan ochish** (Shift+Enter yoki o‘ng tugma) shu fayl turini e’lon qilgan dasturlarni ko‘rsatadi. macOS’da ro‘yxat dasturlarning `Info.plist` fayllaridan, Linux’da `.desktop` yozuvlaridan olinadi; Windows’da tizim dialogi ishlatiladi. Dastur yoki skript ochilishidan oldin tasdiq so‘raladi.
- **Yangi papka (F7), nomini o‘zgartirish (Shift+F6) va savat (F8 / Delete).** Har amal rejadan o‘tadi: diskda biror narsa o‘zgarishidan oldin broker uni tekshiradi. Savatga tashlashdan oldin tasdiq so‘raladi. Tizim savati ishlamasa, hech narsa o‘chirilmaydi.
- **Nusxalash (F5) va ko‘chirish (F6)** boshqa panelga fon vazifasi sifatida bajariladi: progress va bekor qilish tugmasi bor. Har fayl avval vaqtinchalik `.part` faylga yoziladi, shuning uchun to‘xtatilgan nusxa hech qachon tugagandek ko‘rinmaydi. Nom band bo‘lsa, siz tanlaysiz: o‘tkazib yuborish, ikkalasini saqlash yoki almashtirish (papkalar birlashtiriladi). Disklararo ko‘chirishda manba faqat nusxa to‘liq bo‘lib, hajmi mos kelgandan keyin o‘chiriladi.
- Nom, hajm va sana bo‘yicha saralash; ko‘p tanlash, tanlanganlar soni va umumiy hajmi bilan.
- Tizim papka tanlash dialogi; tizimga ergashadigan dark va light mavzu.
- Interfeys tillari: o‘zbek (asosiy), rus, ingliz (Fluent).

### Klaviatura

| Tugma | Amal |
|---|---|
| Tab | Faol panelni almashtirish |
| ↑ ↓ · PageUp PageDown · Home End | Kursorni siljitish |
| Enter · Backspace | Papka yoki faylni ochish · yuqoriga |
| Shift+Enter | Dastur bilan ochish… |
| Alt+← / Alt+→ | Orqaga / oldinga |
| Space / Insert | Tanlash va pastga o‘tish |
| Cmd/Ctrl+T · Cmd/Ctrl+W · Ctrl+Tab | Yangi tab · tabni yopish · keyingi tab |
| Cmd/Ctrl+Shift+P | Buyruqlar paneli |
| Cmd/Ctrl+D | Xatcho‘pga qo‘shish |
| Cmd/Ctrl+A · Cmd/Ctrl+Shift+A | Hammasini tanlash · tanlovni bekor qilish |
| Cmd/Ctrl+1 / 2 / 3 | Nom / hajm / sana bo‘yicha saralash |
| F1 / F2 | Chap / o‘ng panel tomlari |
| F5 · F6 | Boshqa panelga nusxalash · ko‘chirish |
| F7 · Shift+F6 · F8 / Delete | Yangi papka · nomini o‘zgartirish · savat |

## Yig‘ish va ishga tushirish

Barqaror Rust kerak (`rustup`). Loyiha MSRV 1.92 deb e’lon qilingan, lekin faqat 1.94.1 da sinalgan.

```sh
cargo run -p commandor-app                     # GUI
cargo run -p commandor-app -- --smoke          # ochiladi, ikkala panel yuklanishini kutadi va yopiladi
cargo test --workspace                         # unit va worker jarayoni testlari
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask check-boundaries                   # crate bog‘liqliklari yo‘nalishini tekshiradi
```

## Arxitektura qisqacha

- **Rust + `egui`/`eframe`.** Ilova bitta native binary.
- **Xavfsizlik chegarasi.** `commandor-ui` crate’i fayl tizimi, jarayonlar va tarmoqqa kira olmaydi. Buni Clippy qoidalari va `#![forbid]` majburlaydi. UI faqat `AppHandle` orqali broker bilan ishlaydi, broker esa har bir buyruqni tekshiradi.
- **Izolyatsiyalangan worker.** Ishonchsiz kontent parserlari, masalan PDF va OOXML, alohida `commandor --worker` jarayonida ishlaydi. Worker yiqilsa ham UI ishlashda davom etadi.
- **Crate’lar:** [`crates/`](crates/)

  | Crate | Vazifasi |
  |---|---|
  | `commandor-core` | Turlar va protokol |
  | `commandor-broker` | UI uchun fasad |
  | `commandor-fs` | Listing va tomlar |
  | `commandor-launch` | Fayllarni tizim dasturlarida ochish |
  | `commandor-ui` | egui interfeysi |
  | `commandor-app` | Binary |
  | `commandor-ipc` | Worker protokoli |
  | `commandor-i18n` | Tarjimalar |

  Tarjimalar [`resources/i18n/`](resources/i18n/)da, repo tekshiruvlari esa [`xtask/`](xtask/)da.

### Rejada

- Vazifalar navbati, konflikt oynasi va savatli fayl amallari.
- Matn editori (`ropey` + `syntect`).
- Native Markdown preview va PDFium orqali PDF.
- Headless LibreOffice orqali Office preview va tahrir. Ilova ichida Office tahriri hali ochiq savol, shuning uchun va’da qilinmaydi.
- SQLite FTS5 asosidagi to‘liq matnli qidiruv.
- Ixtiyoriy lokal AI (Ollama yoki `llama-server`). U faqat reja *taklif qiladi*; foydalanuvchi tasdiqlamaguncha hech narsa bajarilmaydi.

## Hujjatlar

| Hujjat | Mazmuni |
|---|---|
| [Texnik topshiriq](docs/TECHNICAL-SPEC.uz.md) | Talablar va qabul mezonlari |
| [Arxitektura](docs/ARCHITECTURE.uz.md) | Crate’lar, jarayonlar modeli, ADRlar |
| [Kontraktlar](docs/CONTRACTS.uz.md) | Command/Event turlari, ma’lumotlar modeli, xatolar |
| [Amalga oshirish rejasi](docs/DELIVERY.uz.md) | Bosqichlar, test matritsasi, risklar |
| [TODO](TODO.md) · [hisobotlar](docs/reports/) | Joriy holat va dalillar |

Ish tartibi [AGENTS.md](AGENTS.md)da.

## Litsenziya

[Apache License 2.0](LICENSE) litsenziyasi asosida tarqatiladi.
