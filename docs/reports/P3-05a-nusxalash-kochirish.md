# P3-05a — nusxalash va ko‘chirish: fon vazifasi, progress, bekor qilish, konfliktlar

- Sana: 2026-11-09
- Holat: bajarildi (kod va testlar). GUI oynalari ekranda sinalmagan — [cheklovga](#cheklov-va-qolgan-ish) qarang
- TODO: [P3-05a](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-04](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-05](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: FS-02 (navbat, progress, bekor qilish; yarim fayl to‘liq deb ko‘rsatilmaydi), FS-04 (skip/keep both/replace; batch qarori ko‘rinadi), FS-07 (symlink kuzatilmaydi, siklga kirmaydi), UX-02 (F5/F6 manbasi faol panel, maqsadi boshqa panel)

## Qilingan ish

1. **Kontrakt** ([ops.rs](../../crates/commandor-core/src/ops.rs), [protocol.rs](../../crates/commandor-core/src/protocol.rs)):
   - so‘rovlar: `PlanRequest::Copy`/`Move { location, entries, target, policy }`;
   - konflikt siyosati: `ConflictPolicy { Ask, Skip, KeepBoth, Replace }`;
   - rejaga qo‘shilgan maydonlar: har yozuvda `conflict`, rejada `target`, `policy`, `totals` (`Totals { items, bytes }`);
   - hisobotga qo‘shilgan maydonlar: `skipped`, `target`, `cancelled`;
   - yangi buyruq: `Command::CancelJob`; yangi javoblar: `Response::JobStarted`, `CancelRequested`;
   - yangi hodisalar: `Event::JobProgress`, `Event::JobFinished`.
2. **Rejalash** ([commandor-fs/src/ops.rs](../../crates/commandor-fs/src/ops.rs)):
   - Manba va maqsad grantlar orqali aniqlanadi. Papkani o‘z ichiga nusxalash yoki ko‘chirish va o‘sha papkaga ko‘chirish rad etiladi: `InvalidArgument` va sabab.
   - Har yozuv uchun maqsadda shu nom borligi aniqlanadi.
   - Daraxt oldindan sanaladi (symlink ichiga kirilmaydi). Limit — 200 000 yozuv yoki 3 soniya; undan oshsa `totals` bo‘lmaydi.
   - Xeshga siyosat, maqsad va uning identifikatori ham kiradi.
   - `start_transfer` rejani bir marta ishlatadi. U grantlar va maqsad identifikatorini qayta tekshiradi. Siyosat `Ask` bo‘lib konflikt bor bo‘lsa, `Conflict` xatosini qaytaradi.
3. **Bajarish** ([transfer.rs](../../crates/commandor-fs/src/transfer.rs)):
   - **Fayl.** Avval maqsad papkadagi `.<nom>.commandor-part` faylga 1 MiB bo‘laklarda yoziladi, keyin:
     - har bo‘lakda bekor qilish tekshiriladi;
     - `sync_all`, ruxsatlar va o‘zgartirish vaqti nusxalanadi;
     - nusxa hajmi manba hajmiga solishtiriladi; mos kelmasa (manba nusxalash davomida o‘zgargan) `Conflict`;
     - shundan keyingina `rename` qilinadi.

     Xato yoki bekor qilishda `.part` o‘chiriladi.
   - **Papka va symlink.** Papka rekursiv nusxalanadi. `Replace` siyosatida mavjud papkaga birlashtiriladi: eski fayllar o‘chirilmaydi. Symlink Unix’da link sifatida qayta yaratiladi (ichiga kirilmaydi); Windows’da `PolicyBlocked`.
   - **Konflikt** yuqori darajadagi yozuvlar uchun hal qilinadi:

     | Siyosat | Natija |
     |---|---|
     | `Skip` | `skipped` ro‘yxatiga qo‘shiladi |
     | `KeepBoth` | `nom (2).ext`, `nom (3).ext`, … |
     | `Replace` | fayl almashtiriladi, papka birlashtiriladi |

   - **Ko‘chirish.** Bir diskda `rename` ishlatiladi. Disklararo (`CrossesDevices`) quyidagi tartibda: nusxalash → baytlar sonini solishtirish → manba hamon o‘sha ekanini tekshirish → manbani o‘chirish (`remove_dir_all` symlink ichiga kirmaydi). Hajm mos kelmasa `ChecksumMismatch` qaytadi va manba o‘chirilmaydi.
   - **Bekor qilish.** Bekor qilingan yozuv va undan keyingilarga tegilmaydi, `cancelled: true` qaytadi.
4. **Broker** ([lib.rs](../../crates/commandor-broker/src/lib.rs)):
   - Yangi `commandor-jobs` threadi: vazifalar navbati (64 ta) va bekor qilish belgilari jadvali.
   - Copy/Move uchun `ExecutePlan` `JobStarted` qaytaradi. Progress ko‘pi bilan har 100 ms’da yuboriladi; oxirida `JobFinished`.
   - `CancelJob` broker threadida darhol belgini qo‘yadi. Noma’lum vazifa uchun `NotFound` qaytadi.
   - UI yopilganda barcha vazifalar bekor qilinadi, keyin ular kutiladi.
5. **UI** ([fileops.rs](../../crates/commandor-ui/src/fileops.rs), [fileops_view.rs](../../crates/commandor-ui/src/fileops_view.rs), [lib.rs](../../crates/commandor-ui/src/lib.rs)):
   - **F5/F6.** Faol paneldagi tanlov (bo‘lmasa kursordagi yozuv) boshqa panelning papkasiga o‘tkaziladi. Tasdiq oynasida yozuvlar soni, jami hajm, "Qayerga" yo‘li va nomlar chiqadi; konfliktli nomlar ogohlantirish rangida.
   - **Konflikt.** Uchta variant beriladi. Tanlov o‘zgarsa, eski reja brokerda unutiladi va yangisi so‘raladi. Variant tanlanmaguncha asosiy tugma o‘chiq turadi.
   - **Vazifa.** Tasdiqdan keyin oyna yopiladi va vazifa pastki panelda ko‘rinadi: progress chizig‘i, foiz, baytlar, nom, navbatdagi vazifalar soni va bekor qilish tugmasi (✖). Tugagach manba va maqsad papkalari barcha panel va tablarda yangilanadi. Bajarilmagan yozuvlar yoki bekor qilish haqida xabar chiqadi.
   - **Xato matni.** `error_text` sabab tafsiloti bo‘lsa aniqroq matn ko‘rsatadi (`error-reason-*`). Masalan, "Bu nom tizim uchun band (CON, NUL, COM1 …)". Bu nom oynasi xatolariga ham taalluqli.
   - Pastki paneldagi `F5`/`F6` tugmalari faollashdi. Palette’ga 2 ta buyruq qo‘shildi.
6. **Hujjatlar.** uz/ru/en uchun 28 ta i18n kaliti qo‘shildi. [CONTRACTS](../CONTRACTS.uz.md) va README’lar yangilandi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: ogohlantirishsiz.
- `cargo test --workspace`: 129 test o‘tdi, 0 yiqildi, 1 tasi `ignore` qilingan (avval 115 ta edi). Yangi testlar:
  - `commandor-fs` (10 ta):
    - progress oxiri `totals`ga teng;
    - `.part` qolmaydi;
    - `Ask` bo‘lsa konfliktda bajarilmaydi;
    - `Skip`/`KeepBoth`/`Replace` (papka birlashadi, eski fayl qoladi);
    - bir diskda ko‘chirish;
    - o‘z ichiga va o‘sha papkaga yo‘l qo‘yilmaydi;
    - 3 MiB faylda bekor qilish — yarim fayl ham, `.part` ham, keyingi yozuv ham qolmaydi;
    - almashtirilgan manba → `StaleReference`;
    - reja bir marta ishlatiladi;
    - symlink link sifatida nusxalanadi;
    - noyob nom (`(2)`, `.config (2)`), daraxtni sanash (sikl yo‘q), "ichidami" tekshiruvi.
  - **Disklararo ko‘chirish** (fs testlari ichida). Haqiqiy ikki qurilmada sinaldi: tizim diskidagi temp (`st_dev` 16777234) → loyiha diskidagi `target/` (16777230). Nusxa to‘g‘ri chiqdi, manba o‘chirildi. Qurilmalar bir xil bo‘lsa (masalan, CI’da) test bir diskdagi ko‘chirishni tekshiradi.
  - `commandor-broker` (1 ta): 2 MiB nusxa fon vazifasi sifatida (`JobStarted` → `JobProgress` → `JobFinished`); noma’lum vazifani bekor qilish → `NotFound`.
  - `commandor-ui` (3 ta): F5 maqsadi boshqa panel; konfliktda siyosatsiz tasdiq yo‘q; siyosat almashganda eski reja unutiladi va qayta so‘raladi; qayta reja paytida Esc ishlamaydi; vazifa progressi ulushi; begona vazifa hisobga olinmaydi; bekor qilish bir marta yuboriladi; tugashda bekor qilish xabari va ikkala papka yangilanadi; noto‘g‘ri holatlarda hech narsa yuborilmaydi.
- `cargo xtask check-boundaries`: OK (9 crate).
- `cargo run -p commandor-app -- --smoke`: SMOKE OK.
- **GUI.** Himoya skripti bir marta ilovani faol deb topdi, keyin faol ilova Microsoft Excel bo‘lib qoldi: siz kompyuterdan foydalanayotgan edingiz. Shuning uchun hech qanday tugma yuborilmadi. Oyna skrinshoti ham olinmadi: ilova oynasi ekrandagi oynalar ro‘yxatida yo‘q edi.

- **Foydalanuvchi tekshiruvi (2026-11-09):** foydalanuvchi ilovani qo‘lda sinab, "hammasi ishladi" deb xabar berdi. Bu foydalanuvchining so‘zi; qaysi qadamlar va qaysi OSda sinalgani batafsil qayd etilmagan. Yuqoridagi cheklovlar (Linux/Windows, haqiqiy savat va boshqalar) alohida tasdiqlanmagan.

## Cheklov va qolgan ish

- **Oynalar ekranda sinalmagan.** Tasdiq oynasi, konflikt variantlari va pastki paneldagi progress faqat kod va unit testlar darajasida tekshirilgan. `⎘`/`➜` belgilari egui shriftida qanday chiqishi ham ko‘rilmagan.
- **Konflikt qarori butun reja uchun bitta.** Har fayl uchun alohida so‘rash yo‘q. Papka ichidagi konfliktlar `Replace` siyosatida almashtiriladi, boshqa siyosatlarda (`Skip`/`KeepBoth`) yuqori daraja bo‘yicha hal qilinadi. Bu FS-04’dagi "batch qarori ko‘rinadi" talabiga mos, lekin per-item tanlov keyinroq qilinadi.
- **Tezlik.** `copy_file_range`/`clonefile` (APFS klon) ishlatilmaydi, oddiy oqim bilan nusxalanadi. Xattr, ACL va egasi nusxalanmaydi; faqat ruxsatlar va o‘zgartirish vaqti (CONTRACTS’dagi metadata talabi ochiq qoladi).
- **Disklararo ko‘chirishdagi tekshiruv.** Faqat baytlar soni tekshiriladi, xesh yo‘q. Daraxt sanash limitdan oshsa, oldindan solishtirish o‘tkazib yuboriladi: har fayl hajmi baribir tekshiriladi.
- **Bekor qilishdan keyingi holat.** Bekor qilingan papka nusxasida tugallangan fayllar qoladi, ular to‘liq va to‘g‘ri. Jurnal yoki tiklash (FS-08) yo‘q.
- **Yozishdagi TOCTOU oynasi.** `Replace`siz holatda `.part` → `rename` oldidan maqsad yo‘qligi tekshiriladi, lekin oradagi oynada boshqa jarayon fayl yaratsa (Unix’da), u bosib yozilishi mumkin. Bu rename’dagi bilan bir xil cheklov.
- **Navbat ketma-ket ishlaydi.** Vazifalar parallel bajarilmaydi, ustuvorlik va pauza yo‘q.
- **Platformalar.** Windows/Linux faqat CI’da. Windows’da symlink nusxalanmaydi, manba almashtirilganini aniqlash ham yo‘q (inode yo‘q).
