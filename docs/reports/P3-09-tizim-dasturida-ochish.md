# P3-09 — faylni tizim dasturida ochish va "Open with"

- Sana: 2026-11-09
- Holat: bajarildi (kod va testlar). Vizual GUI sinovi va real ishga tushirish qilinmadi — [cheklovga](#cheklov-va-qolgan-ish) qarang
- TODO: [P3-09](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: foydalanuvchi talabi (2026-11-09). Texnik topshiriqda alohida ID yo‘q; UX-03/UX-05 (klaviatura) va [Kontent xavfsizligi](../ARCHITECTURE.uz.md#kontent-xavfsizligi) (tashqi dastur faqat argument massivi bilan)

## Qilingan ish

1. **Kontrakt** ([`launch.rs`](../../crates/commandor-core/src/launch.rs), [`protocol.rs`](../../crates/commandor-core/src/protocol.rs)).
   - Yangi buyruqlar: `Command::OpenEntry { location, entry, with, allow_executable }` va `Command::ListOpenWith { location, entry }`.
   - Javoblar: `Response::Opened(OpenOutcome)` va `Response::OpenWithApps(OpenWithList)`.
   - Yangi xato kodi: `ErrorCode::NoApplication`.
   - UI fayl yo‘lini yubormaydi: faqat listingdagi opaque ID’lar va broker bergan `AppId` ishlatiladi.
2. **Yo‘lni aniqlash** ([`commandor-fs`](../../crates/commandor-fs/src/lib.rs)). `FileService::resolve_file` faqat amaldagi grantga tegishli oddiy faylni qabul qiladi. Papka (`InvalidArgument`), symlink (`PolicyBlocked`) va eskirgan grant (`StaleReference`) rad etiladi.
3. **Yangi `commandor-launch` crate’i** ([lib.rs](../../crates/commandor-launch/src/lib.rs)). Har OSda ishlash tartibi:

   | OS | Standart dastur | "Open with" ro‘yxati | Tanlangan dastur |
   |---|---|---|---|
   | macOS | `/usr/bin/open <fayl>` | `/Applications`, `/System/Applications`, `~/Applications` ichidagi `.app` paketlarining `Info.plist` fayllari | `open -a <app> <fayl>` |
   | Linux/BSD | `xdg-open <fayl>` | XDG `applications` papkalaridagi `.desktop` yozuvlari | `Exec` maydonidan tuzilgan argv |
   | Windows | `explorer.exe <fayl>` | o‘z ro‘yxat yo‘q | `rundll32 shell32.dll,OpenAs_RunDLL` (tizim dialogi) |

   - **macOS katalogi** ([macos.rs](../../crates/commandor-launch/src/macos.rs)). LaunchServices FFI ishlatilmaydi, chunki `unsafe` taqiqlangan. O‘rniga `CFBundleDocumentTypes` va `CoreTypes.bundle` hamda dasturlarning UTI e’lonlari o‘qiladi (UTI — macOS’dagi fayl turi identifikatori). Fayl turining "ota" turlari ham hisobga olinadi: masalan, `public.png` → `public.image`. Faqat `public.data` kabi umumiy tur orqali mos keladigan dastur tavsiya etilganlar qatoriga kirmaydi. Katalog 60 soniya keshlanadi.
   - **Linux** ([desktop.rs](../../crates/commandor-launch/src/desktop.rs)). Fayl turi `xdg-mime query filetype` bilan, standart dastur `xdg-mime query default` bilan aniqlanadi. Bir xil nomli yozuvlar orasida XDG ustuvorligi saqlanadi. `Exec` maydoni spetsifikatsiya bo‘yicha qo‘shtirnoq va escape qoidalari bilan argumentlarga ajratiladi; `%f/%F/%u/%U` fayl yo‘li bilan almashtiriladi. `Terminal=true` va `Hidden=true` yozuvlar o‘tkazib yuboriladi.
   - **Jarayon ishga tushirish** ([spawn.rs](../../crates/commandor-launch/src/spawn.rs)):
     - shell ishlatilmaydi, stdin/stdout/stderr ulanmaydi;
     - `open`/`xdg-open` exit kodi 3 soniyagacha kutiladi; noldan farqli kod `NoApplication` deb qaytariladi;
     - uzoq ishlaydigan jarayon fonda yig‘iladi (zombie qolmaydi).
   - **Bajariladigan fayllar** ([risk.rs](../../crates/commandor-launch/src/risk.rs)). Standart ochishda `.exe`, `.sh`, `.command`, `.ps1`, `.AppImage` va shunga o‘xshash fayllar, shuningdek kengaytmasiz `x` bitli fayllar tasdiqsiz ishga tushmaydi: `OpenOutcome::NeedsConfirmation` qaytadi. Kengaytmali, lekin `x` bitli hujjatlar (masalan, exFAT diskdagi) bu tekshiruvga tushmaydi.
4. **Broker** ([lib.rs](../../crates/commandor-broker/src/lib.rs)). File thread yo‘lni grant orqali aniqlaydi va ishni alohida `commandor-launch` threadiga uzatadi (navbat 16 ta). Katalogni o‘qish va launcher’ni kutish listingni bloklamaydi. Navbat to‘lsa `RateLimited` qaytadi.
5. **UI**:
   - Holat [opening.rs](../../crates/commandor-ui/src/opening.rs)da, oynalar [open_with_view.rs](../../crates/commandor-ui/src/open_with_view.rs)da.
   - Enter va ikki marta bosish: papka bo‘lsa ichiga kiradi, fayl bo‘lsa standart dasturda ochadi.
   - Shift+Enter, o‘ng tugma menyusi va palette’dagi "Dastur bilan ochish…" "Open with" oynasini ochadi. Oynada:
     - tavsiya etilgan dasturlar birinchi chiqadi;
     - "Barcha dasturlar (N)" tugmasi qolganlarini ko‘rsatadi;
     - nom bo‘yicha qidirish mumkin; qidiruv barcha dasturlar orasida ishlaydi;
     - Linux’da standart dastur "standart" yorlig‘i bilan belgilanadi;
     - Windows’da "Tizim dialogi…" tugmasi chiqadi.
   - Bajariladigan fayl uchun tasdiq oynasi chiqadi. Enter u yerda o‘chirilgan: ishga tushirish faqat tugmani bosish bilan, Esc esa bekor qiladi.
   - Ochish xatosi pastki panelda × bilan yopiladigan xabar sifatida ko‘rsatiladi.
   - Palette’ga 2 ta yangi buyruq qo‘shildi: `Open` (Enter) va `OpenWith` (Shift+Enter).
6. **Hujjatlar va qoidalar**:
   - uz/ru/en uchun 22 ta i18n kaliti qo‘shildi;
   - [CONTRACTS](../CONTRACTS.uz.md)dagi `Command`/`Response` ro‘yxati amaldagi holatga keltirildi va `NO_APPLICATION` qo‘shildi;
   - [ARCHITECTURE](../ARCHITECTURE.uz.md)ga crate va oqim qo‘shildi;
   - `xtask` qoidasi: `commandor-launch → commandor-core, plist`;
   - yangi bog‘liqlik `plist` 1.7 (MIT, sof Rust);
   - README’lar yangilandi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: ogohlantirishsiz.
- `cargo test --workspace`: 101 test o‘tdi, 0 yiqildi, 1 tasi `ignore` qilingan (avval 78 ta edi). Yangi testlar:
  - `commandor-launch` (16 ta): bajariladigan fayl qoidalari; exit kodlari va yo‘q dastur; argumentlar shell orqali talqin qilinmasligi (`$(id)`, `;` matnicha qoladi); `capture` timeoutda jarayonni o‘ldirishi; sintetik `Info.plist`/UTI katalogi (ota turlar, umumiy turlar, `Role=None`, ichki papka, buzilgan plist); `.desktop` parse, `Exec` argv (qo‘shtirnoq, `%%`, field kodlari, yopilmagan qo‘shtirnoq), MIME naqshlari, XDG ustuvorligi; eskirgan `AppId`; tasdiq so‘rash.
  - `commandor-fs` (1 ta): `resolve_file` — fayl, papka, symlink, noma’lum grant, o‘chirilgan fayl.
  - `commandor-broker` (1 ta): grant orqali tekshirish — papka, eskirgan grant, skript tasdig‘i, ro‘yxat. Bu testda hech qanday dastur ishga tushirilmaydi.
  - `commandor-ui` (5 ta): Enter (papka/fayl), xato xabari, tasdiq oqimi, "Open with" filtri va tanlovi, eskirgan javoblar.
- `cargo xtask check-boundaries`: OK (9 crate).
- `cargo run -p commandor-app -- --smoke`: SMOKE OK.
- Haqiqiy macOS katalogi, qo‘lda ishlatiladigan `ignore` testi bilan tekshirildi (`cargo test -p commandor-launch -- --ignored --nocapture`). Unda faqat `Info.plist`lar o‘qiladi, hech narsa ishga tushirilmaydi. Natija: 140 ta dastur, birinchi skan va 4 ta keshli so‘rov jami 157 ms.

  | Fayl | Tavsiya etilganlar | Ichida |
  |---|---|---|
  | `.txt` | 20 | TextEdit, Xcode |
  | `.png` | 14 | Preview |
  | `.pdf` | 19 | Preview, Books |
  | `.md` | 19 | TextEdit |
  | noma’lum kengaytma | 0 | — |

- **Foydalanuvchi tekshiruvi (2026-11-09):** foydalanuvchi ilovani qo‘lda sinab, "hammasi ishladi" deb xabar berdi. Bu foydalanuvchining so‘zi; qaysi qadamlar va qaysi OSda sinalgani batafsil qayd etilmagan. Yuqoridagi cheklovlar (Linux/Windows, haqiqiy savat va boshqalar) alohida tasdiqlanmagan.

## Cheklov va qolgan ish

- **GUI vizual sinovi va real ishga tushirish qilinmadi.** Ilova ishga tushdi, lekin ekran qulflangan edi: ekrandagi oynalar ro‘yxatida faqat `loginwindow` bor edi, ilova oynasi esa ekranda ko‘rinmadi. Shuning uchun "Open with" oynasi, tasdiq dialogi va xabar ko‘rinishi skrinshot bilan tekshirilmadi. `open <fayl>` bilan haqiqiy ochish ham sinalmadi. Keyingi sessiyada sintetik papkada (`tempfile`) qo‘lda tekshirish kerak.
- Linux va Windows yo‘llari faqat CI’da kompilyatsiya va unit testlar darajasida tekshiriladi. `xdg-open`, `.desktop` orqali ishga tushirish va `OpenAs_RunDLL` real OSda sinalmagan. Windows’da o‘z dasturlar ro‘yxati yo‘q, faqat tizim dialogi ishlatiladi.
- macOS tavsiyasi LaunchServices bilan aynan bir xil emas. Standart dastur (`is_default`) aniqlanmaydi; tizim katalogidan tashqaridagi ilovalar va `CFBundleDocumentTypes`siz ilovalar ro‘yxatga kirmaydi.
- Symlink faylni ochish `PolicyBlocked` bilan rad etiladi: papka symlinklari bilan bir xil siyosat. Kerak bo‘lsa, alohida qaror bilan o‘zgartiriladi.
- `.app` paketlari papka sifatida ko‘rinadi, Enter ularning ichiga kiradi. Ilovani ishga tushirish alohida buyruq bo‘lishi kerak.
- Bir nechta tanlangan faylni bir vaqtda ochish yo‘q; faqat kursordagi fayl ochiladi.
- Shu crate’da TERM-01 (tizim terminalini ochish, [P4-05](../../TODO.md#4-faza--hujjatlar-va-tashqi-terminal-r12r13)) amalga oshirilishi mumkin.
