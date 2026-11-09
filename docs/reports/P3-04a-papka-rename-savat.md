# P3-04a — yangi papka, nomini o‘zgartirish va savat (reja → tasdiq → bajarish)

- Sana: 2026-11-09
- Holat: bajarildi (kod va testlar). Oynalarning GUI sinovi va haqiqiy tizim savati sinalmagan — [cheklovga](#cheklov-va-qolgan-ish) qarang
- TODO: [P3-04a](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-04](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: FS-03 (rename, yangi papka, OS nom qoidalari, case-only rename), FS-05 (savat; savat bo‘lmasa avtomatik doimiy o‘chirish yo‘q), UX-08 (tasdiqni chetlab o‘tmaslik), [CONTRACTS 2.1](../CONTRACTS.uz.md#21-amal-rejasi)

## Qilingan ish

1. **Kontrakt** ([ops.rs](../../crates/commandor-core/src/ops.rs), [protocol.rs](../../crates/commandor-core/src/protocol.rs)):
   - so‘rovlar: `PlanRequest::{CreateDirectory, Rename, Trash}`;
   - buyruqlar: `Command::CreatePlan`, `ExecutePlan { plan, hash }`, `DiscardPlan`;
   - javoblar: `Response::Plan(OperationPlan)`, `Executed(OperationReport)` (har yozuv bo‘yicha bajarildi/bajarilmadi), `Discarded`;
   - yangi ID: `PlanId`;
   - yangi xato kodlari: `InvalidName`, `TrashUnavailable`.

   Bu ishda `AppError` va `DetailValue` turlari ham `Eq` oldi.
2. **Reja va bajarish** ([commandor-fs/src/ops.rs](../../crates/commandor-fs/src/ops.rs)).
   - *Reja tuzish* diskni o‘zgartirmaydi:
     - manba grant orqali aniqlanadi;
     - nom OS qoidalari bilan tekshiriladi (`validate_name`, `NameRules`);
     - reja 10 daqiqa amal qiladi, bir vaqtda 32 tagacha saqlanadi;
     - xesh barqaror FNV-1a 64: ID, amal, papka, nomlar, turlar, hajmlar, maqsad va manba identifikatori (dev+inode) bo‘yicha hisoblanadi.
   - *Nom qoidalari*:

     | Qamrov | Taqiqlangan |
     |---|---|
     | Hamma OS | bo‘sh nom, `.`/`..`, `/`, NUL, boshqaruv belgilari, 255 baytdan uzun nom |
     | macOS | `:` |
     | Windows | `<>:"\|?*\\`, oxiridagi probel yoki nuqta, `CON/PRN/AUX/NUL/COM1–9/LPT1–9` (kengaytma bilan ham) |

   - *Bajarish*. Reja bir marta ishlatiladi. Xesh mos kelmasa yoki muddat o‘tgan bo‘lsa `PlanExpired` qaytadi. Grant qayta tekshiriladi, manba identifikatori (inode, symlinkmi) solishtiriladi; o‘zgargan bo‘lsa `StaleReference`.
     - **Yangi papka:** `create_dir` mavjud maqsadni bosib yozmaydi.
     - **Rename:** maqsad bo‘sh bo‘lishi shart. Istisno — case-only rename: o‘sha inode, Windows’da esa faqat registr farqi.
     - **Savat:** yozuvlar birma-bir tashlanadi, natija har biri uchun alohida yoziladi.
   - *Savat* `trash` 5.3 (MIT) orqali:
     - macOS’da `NsFileManager` usuli ishlatiladi; Finder/AppleScript usuli Automation ruxsatini so‘ragani uchun tanlanmadi;
     - Windows’da Recycle Bin, Linux’da freedesktop savati;
     - savat xatosi `TrashUnavailable` qaytaradi va fayl joyida qoladi; doimiy o‘chirish yo‘q;
     - testlarda `Trasher` trait orqali soxta savat ishlatiladi.
   - `io_error` xaritasi kengaytirildi:

     | `io::ErrorKind` | `ErrorCode` |
     |---|---|
     | `AlreadyExists` | `DestinationExists` |
     | `StorageFull` | `DiskFull` |
     | `ReadOnlyFilesystem` | `PermissionDenied` |
     | `InvalidFilename` | `InvalidName` |

3. **Broker.** Uchala buyruq file threadda bajariladi ([lib.rs](../../crates/commandor-broker/src/lib.rs)).
4. **UI** — holat [fileops.rs](../../crates/commandor-ui/src/fileops.rs)da, oynalar [fileops_view.rs](../../crates/commandor-ui/src/fileops_view.rs)da:
   - **F7 / Shift+F6.** Nom oynasi ochiladi; rename’da kengaytmasiz qism belgilangan bo‘ladi. Enter reja so‘raydi. Tasdiq talab qilmaydigan reja keyingi kadrda `AppState::pump` orqali bajariladi. Xato bo‘lsa (`DestinationExists`, `InvalidName`) oyna ochiq qoladi va xato ko‘rsatiladi.
   - **F8 / Delete / Cmd+Backspace.** Tanlov bo‘lsa tanlangan yozuvlar, aks holda kursordagi yozuv olinadi. Reja kelgach tasdiq oynasi chiqadi: yozuvlar soni, hajmi, papka va dastlabki 8 ta nom. "Savatga tashlash" yoki Enter tasdiqlaydi; Esc yoki bekor qilish rejani brokerda ham unutadi (`DiscardPlan`). Bajarilayotganda Esc oynani yopmaydi.
   - **Natija.** Shu papkani ko‘rsatayotgan barcha panel va tablar yangilanadi. Yangi yoki qayta nomlangan yozuvga kursor qo‘yiladi (`PanelState::focus_name`). Bajarilmagan yozuvlar pastki panelda "Amal bajarilmadi (1/3) «nom»: sabab" ko‘rinishida xabar qilinadi.
   - Pastki paneldagi `F7 Papka` va `F8 Savat` tugmalari faollashdi. Palette’ga 3 ta buyruq qo‘shildi.
5. **Hujjatlar va qoidalar:**
   - uz/ru/en uchun 19 ta i18n kaliti qo‘shildi;
   - [CONTRACTS](../CONTRACTS.uz.md)dagi `Command`/`Response` ro‘yxati va xatolar jadvali yangilandi;
   - `xtask` qoidasi: `commandor-fs → trash`;
   - README’lar yangilandi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: ogohlantirishsiz.
- `cargo test --workspace`: 115 test o‘tdi, 0 yiqildi, 1 tasi `ignore` qilingan (avval 101 ta edi). Yangi testlar:
  - `commandor-fs` (8 ta): nom qoidalari uchala OS uchun; FNV barqarorligi; yangi papka (reja diskni o‘zgartirmaydi, reja bir marta ishlatiladi, mavjud nom, `../`); noto‘g‘ri xesh; reja va bajarish orasida maqsad paydo bo‘lishi (mavjud fayl tegilmaydi); rename (band nom, o‘zgarmagan nom, muvaffaqiyat, almashtirilgan manba → `StaleReference`); APFS’da case-only rename; savat (tasdiq kerak, takror ID, har yozuv hisoboti, savat ishlamasa fayl joyida qoladi, bo‘sh ro‘yxat); `DiscardPlan`, eskirgan grant.
  - `commandor-broker` (1 ta): file thread orqali reja → bajarish → bekor qilish.
  - `commandor-ui` (5 ta): F7 oqimi (avtomatik bajarish, ikkala panel yangilanishi, kursor yangi papkaga); rename xatosida oyna ochiq qolishi; savat (tanlov ustunligi, tasdiqsiz bajarilmasligi, bajarilayotganda Esc, xato xabari); bekor qilishda `DiscardPlan`; `stem_chars`.
- `cargo xtask check-boundaries`: OK (9 crate).
- `cargo run -p commandor-app -- --smoke`: SMOKE OK.
- Vizual: ilova oynasi skrinshotida `F7 Papka` va `F8 Savat` tugmalari faol ko‘rinishda.
- Avtomatik GUI sinovida har bir tugma yuborishdan oldin himoya skripti ilova faol va ekranda ekanini tekshirdi. Bir marta ilova faol bo‘lsa ham oyna ekrandan chiqib ketdi, keyin esa faol ilova Finder bo‘lib qoldi. Himoya hech qanday tugma yubormadi va sinov to‘xtatildi.

- **Foydalanuvchi tekshiruvi (2026-11-09):** foydalanuvchi ilovani qo‘lda sinab, "hammasi ishladi" deb xabar berdi. Bu foydalanuvchining so‘zi; qaysi qadamlar va qaysi OSda sinalgani batafsil qayd etilmagan. Yuqoridagi cheklovlar (Linux/Windows, haqiqiy savat va boshqalar) alohida tasdiqlanmagan.

## Cheklov va qolgan ish

- **Oynalar GUI’da sinalmagan.** Nom oynasi, rename’da nom belgilanishi, savat tasdig‘i va xabar faqat kod va testlar darajasida tekshirilgan. Sababi yuqorida: oyna fokusi barqaror bo‘lmadi. `🗑` belgisi egui shriftida chiqishi ham tekshirilmagan.
- **Haqiqiy tizim savati sinalmagan.** Testlar foydalanuvchi savatini to‘ldirmasligi uchun soxta savat ishlatadi. macOS, Windows va Linux savatini qo‘lda tekshirish kerak; macOS’da `NsFileManager` usulida "Put Back" ishlamasligi mumkin.
- **Rename’dagi TOCTOU oynasi.** Maqsad tekshiruvi va `rename` orasida boshqa jarayon xuddi shu nomli fayl yaratsa, Unix’da u bosib yozilishi mumkin. Windows’da `std::fs::rename` ham mavjud faylni almashtiradi. Buni yopish uchun `renameat2(RENAME_NOREPLACE)` yoki `renamex_np(RENAME_EXCL)` kerak, ular esa FFI (`unsafe`) talab qiladi.
- **Windows.** Fayl identifikatori yo‘q, shuning uchun almashtirilgan manba aniqlanmaydi va bu holat faqat Unix’da testlanadi. Case-only rename Windows’da faqat registr farqi bo‘yicha qabul qilinadi.
- **Qolgan amallar.** Doimiy o‘chirish, bo‘sh matn fayli yaratish (FS-03 qismi), copy/move, konflikt UI va navbat P3-04/P3-05da. Undo yo‘q. Amallar jurnali (FS-08) yo‘q.
- Savat ko‘p yozuvda file threadni band qiladi: o‘sha paytda listing kutadi. Uzoq amallar P3-05 navbatiga o‘tkaziladi.
