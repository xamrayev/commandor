# P3-07a — sozlamalar fayli: xatcho‘plar va sessiyani saqlash; CI tuzatishi

- Sana: 2026-11-09
- Holat: bajarildi (kod va testlar). Real ishga tushirish/qayta ochish GUI’da sinalmagan. Klaviatura xaritasi (UX-09 ikkinchi qismi) qilinmagan
- TODO: [P3-07a](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-07](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-09 (TOML sozlamalar; noto‘g‘ri fayl ilovani yiqitmaydi, xato ko‘rsatiladi va sukut qiymatlar ishlatiladi), UX-03 (bookmarks)

## Qilingan ish

1. **Kontrakt** ([settings.rs](../../crates/commandor-core/src/settings.rs)):
   - turlar: `Settings { version, ui.restore_session, bookmarks, session { active, left, right } }`; har tab uchun `TabSession { path, hidden, sort }`;
   - barcha maydonlar `serde(default)`: qisman yoki eski fayl ham o‘qiladi, noma’lum maydon xato emas;
   - buyruqlar `Command::LoadSettings` va `SaveSettings(Box<Settings>)`; javoblar `Response::Settings(Box<SettingsLoad>)` va `SettingsSaved`.
2. **Yangi `commandor-settings` crate’i** ([lib.rs](../../crates/commandor-settings/src/lib.rs)). Bog‘liqlik — `toml` 1.2. Fayl joylashuvi:

   | OS | Yo‘l |
   |---|---|
   | macOS | `~/Library/Application Support/Commandor/settings.toml` |
   | Windows | `%APPDATA%\Commandor\settings.toml` |
   | Linux | `$XDG_CONFIG_HOME/commandor/settings.toml` (yoki `~/.config/commandor/settings.toml`) |

   O‘qish va yozish qoidalari:
   - fayl yo‘q bo‘lsa — sukut qiymatlar, xato emas;
   - TOML xatosi, UTF-8 bo‘lmagan matn yoki 1 MiB’dan katta fayl — sukut qiymatlar va `problem`; asl fayl `settings.toml.broken` nomi bilan saqlanadi, shuning uchun keyingi saqlash foydalanuvchi tahririni yo‘qotmaydi;
   - saqlash atomik: `.tmp` → `sync_all` → `rename`.
3. **Broker.** Saqlash file threadda bajariladi:
   - `start()` — standart yo‘l;
   - `start_at` (testlar va smoke) — saqlash o‘chiq: foydalanuvchi fayliga tegilmaydi;
   - `start_with(…, Some(path))` — yo‘lni aniq berish.
4. **UI** ([settings_state.rs](../../crates/commandor-ui/src/settings_state.rs)):
   - **Yuklash.** Ishga tushganda `LoadSettings` so‘raladi. Javob kelmaguncha **hech narsa saqlanmaydi** — mavjud fayl sukut qiymatlar bilan bosib yozilmasligi uchun.
   - **Tiklash** (`pump` orqali):
     - xatcho‘plar — takror, nisbiy va bo‘sh yo‘llar tashlanadi, 50 tagacha;
     - har panel tablari — bo‘sh yoki nisbiy yo‘l tashlanadi, 12 tagacha, joriy indeks chegaralanadi;
     - yashirin fayllar va saralash tab bo‘yicha tiklanadi; faol panel ham tiklanadi;
     - sessiyasi bo‘sh panel o‘zgarmaydi;
     - `ui.restore_session = false` bo‘lsa, faqat xatcho‘plar tiklanadi.
   - **Saqlash.** Holat surati — filtr kabi vaqtinchalik qiymatlarsiz; yuklanmoqda yoki xato holatidagi tab uchun tarixdagi yo‘l olinadi. Surat oxirgi saqlangandan farq qilsa, sekundiga ko‘pi bilan bir marta saqlanadi. Chiqishda (`on_exit`) oldingi javobni kutmasdan oxirgi holat yuboriladi (`flush_settings`), broker esa uni yopilishdan oldin bajaradi.
   - **Xatolar:**
     - o‘qishda IO xatosi (masalan, ruxsat yo‘q) bo‘lsa, saqlash shu sessiyada o‘chiriladi — o‘qilmagan faylni bosib yozmaslik uchun;
     - buzilgan fayl zaxiralangan bo‘lsa, saqlash ruxsat etiladi;
     - saqlash xatosi bir marta xabar qilinadi, keyin saqlash to‘xtatiladi.
   - Xabar `NoticeKind::Settings` sifatida chiqadi (`Sozlamalar: …`). Xatcho‘plar popup’idagi izoh yangilandi.
5. **CI tuzatishi** (push qilingan `b9a59f6` Ubuntu va Windows’da yiqilgan edi, [run 42595329752](https://github.com/xamrayev/commandor/actions/runs/42595329752)):
   - **Ubuntu — haqiqiy bug.** `replaced_source_is_stale…` testi yiqildi: ext4 o‘chirilgan fayl inode raqamini darhol qayta beradi, shuning uchun (dev, inode) almashtirilgan faylni sezmadi va ko‘chirish davom etdi. Tuzatish: oddiy fayllar uchun "hamon o‘sha" tekshiruviga hajm va o‘zgartirish vaqti qo‘shildi ([ops.rs](../../crates/commandor-fs/src/ops.rs), `fingerprint`). Bu rename, savat, copy va move — hammasiga taalluqli. Inode’ga bog‘liq bo‘lmagan barqaror test qo‘shildi: fayl o‘z joyida o‘zgartiriladi → `StaleReference`.
   - **Windows — lint.** `spawn::Wait::ExitCode` Windows’da ishlatilmaydi (dead code), shu sababli clippy yiqildi. Variantga `cfg_attr(windows, allow(dead_code))` qo‘shildi.
   - **Oldini olish.** Lokal `x86_64-pc-windows-msvc` va `x86_64-unknown-linux-gnu` target’lari o‘rnatildi. `cargo clippy --workspace --all-targets --target <t> -- -D warnings` ikkalasida ham exit 0 bilan tugadi. Bundan keyin push’dan oldin shu tekshiruv ishlatiladi.
6. **Hujjatlar.** uz/ru/en uchun 3 ta i18n kaliti qo‘shildi. [CONTRACTS](../CONTRACTS.uz.md), [ARCHITECTURE](../ARCHITECTURE.uz.md) va README’lar yangilandi. `xtask` qoidasi: `commandor-settings → commandor-core, toml`.

## Tekshiruv

macOS arm64, Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: toza — macOS’da, shuningdek `--target x86_64-pc-windows-msvc` va `--target x86_64-unknown-linux-gnu` bilan.
- `cargo test --workspace`: 145 test o‘tdi, 0 yiqildi, 1 tasi `ignore` qilingan (P3-02e’dan keyin 132 ta edi). Yangi testlar:
  - `commandor-settings` (5 ta): roundtrip (papka yaratiladi, `.tmp` qolmaydi); qisman va noma’lum maydonlar; buzilgan, UTF-8 bo‘lmagan va juda katta fayl → sukut qiymatlar, `problem` va `.broken` zaxira (keyingi saqlashdan keyin ham qoladi); o‘chiq store diskka tegmaydi; standart yo‘l.
  - `commandor-broker` (1 ta): `start_with` orqali saqlash va qayta o‘qish.
  - `commandor-ui` (6 ta): yuklanmaguncha saqlanmaydi; tiklashda validatsiya; faqat o‘zgarish saqlanadi va oldingi javob kutiladi; chiqishda javob kutilmaydi; surat (listing yo‘li, filtrsiz); IO xatosida saqlash o‘chiq, buzilgan faylda yoqiq.
  - `commandor-fs` (1 ta): o‘z joyida o‘zgartirilgan manba → `StaleReference`.
- `cargo xtask check-boundaries`: OK (10 crate).
- `--smoke`: SMOKE OK. Smoke `start_at` bilan ishlaydi, shuning uchun foydalanuvchi sozlamalariga tegmaydi.

## Cheklov va qolgan ish

- **Real sinov yo‘q.** Ilovani yopib qayta ochganda tablar va xatcho‘plar tiklanishi qo‘lda sinalmagan. `on_exit`dagi oxirgi saqlash ham real yopilishda tekshirilmagan.
- **Klaviatura xaritasi yo‘q** (UX-09 ikkinchi qismi). Til tanlash ham sozlamaga kiritilmagan: til hozircha `commandor-app`da `uz` qilib berilgan.
- **Ishlayotgan ilova va tashqi tahrir.** Ilova ishlayotgan paytda faylga qo‘lda kiritilgan o‘zgartirish keyingi avto-saqlashda bosib yoziladi (fayl boshidagi izohda aytilgan). Faylni kuzatish (watcher) yo‘q.
- **Tablar orasidagi farqlar.** Tablarning navigatsiya tarixi va filtri saqlanmaydi. Tiklangan papka endi mavjud bo‘lmasa, tab xato holatida ochiladi ("Qayta urinish" yoki uy papkasi tugmasi bilan chiqish mumkin).
