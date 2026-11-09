# P3-02d — panel tablari, xatcho‘plar, buyruqlar paneli va kursorga aylantirish

- Sana: 2026-11-09
- Holat: bajarildi (macOS arm64); xatcho‘plarni saqlash P3-07ga qoldi
- TODO: [P3-02d](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-08 (tablar, bookmarks, buyruq paneli), UX-03/UX-04 (klaviatura)

## Qilingan ish

1. **Tablar.** [`state.rs`](../../crates/commandor-ui/src/state.rs): `AppState.panels` endi `[PanelTabs; 2]`. Har tab alohida `PanelState`: listing, tarix, kursor, tanlov va filtr. Yangi tab joriy papka va ko‘rinish sozlamalarini oladi, lekin kursor/tanlovni olmaydi. Bir panelda 12 tabgacha ochiladi, oxirgi tab yopilmaydi. Fondagi tab o‘z javobini oladi; yopilgan tabga kelgan javob holatni o‘zgartirmaydi. Har tabda barqaror `tab_id` bor, shuning uchun scroll holati tablar orasida aralashmaydi. Tab qatori [`tabs_view.rs`](../../crates/commandor-ui/src/tabs_view.rs)da: yuklanish belgisi, × va o‘rta tugma bilan yopish, `+`.
2. **Xatcho‘plar.** Ular ★ popup’ida va buyruqlar panelida chiqadi; qo‘shish, ochish va o‘chirish mumkin. Bir xil yo‘l ikki marta qo‘shilmaydi, chegara 50 ta. Hozircha faqat sessiya ichida saqlanadi, buni UIda ham yozib qo‘yganmiz.
3. **Buyruqlar paneli.** Model [`palette.rs`](../../crates/commandor-ui/src/palette.rs)da, oyna [`palette_view.rs`](../../crates/commandor-ui/src/palette_view.rs)da. Cmd/Ctrl+Shift+P yoki yuqori paneldagi tugma bilan ochiladi. Ro‘yxatda 16 ta buyruq va xatcho‘plar bor. Qidiruvda registr hisobga olinmaydi va so‘rovning har bir so‘zi mos kelishi kerak. Boshqaruv: ↑/↓, Enter, Esc yoki fonga bosish. Buyruq `AppState::run_command` orqali faol panelga qo‘llanadi; OS dialogi `UiEffect` sifatida UIga qaytadi.
4. **Yagona yorliqlar jadvali.** [`lib.rs`](../../crates/commandor-ui/src/lib.rs)dagi `command_shortcut` ham klaviaturani ishlaydi, ham panelda yorliqni ko‘rsatadi:

   | Yorliq | Buyruq |
   |---|---|
   | Cmd+T / Cmd+W | yangi tab / tabni yopish |
   | Ctrl+Tab / Ctrl+Shift+Tab | keyingi / oldingi tab |
   | Cmd+D | xatcho‘p qo‘shish |
   | Cmd+A / Cmd+Shift+A | hammasini tanlash / tanlovni bekor qilish |
   | Cmd+R | yangilash |
   | Cmd+Shift+H | uy papkasi |
   | Cmd+Shift+. | yashirin fayllar |
   | Cmd+1/2/3 | nom / hajm / sana bo‘yicha saralash |
   | Cmd+O | papka tanlash dialogi |

   Windows va Linuxda Cmd o‘rniga Ctrl ishlaydi. egui moslashda ortiqcha Shift’ni e’tiborsiz qoldiradi, shuning uchun Shift’li yorliqlar birinchi tekshiriladi.
5. **Kursor.** PageUp/PageDown 20 qatorga, Home/End boshiga va oxiriga o‘tkazadi. Klaviatura bilan siljigan kursor ko‘rinmay qolsa, jadval o‘zi aylanadi (P3-02c’dagi cheklov yopildi). Endi Backspace, Alt+←/→ va boshqa navigatsiya buyruqlari xato holatida ham ishlaydi: muvaffaqiyatsiz o‘tishdan keyin orqaga qaytish mumkin.
6. i18n: uz/ru/en uchun 5 ta yangi kalit qo‘shildi (`bookmarks-empty`, `bookmark-remove`, `bookmark-already`, `bookmarks-session-note`, `palette-hint-run`). Qolgan kalitlar oldindan tayyor edi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: ogohlantirishsiz.
- `cargo test --workspace`: 78 o‘tdi, 0 yiqildi (avval 65 edi). 13 ta yangi test: tab nusxalash va mustaqilligi, fondagi va yopilgan tab javoblari, yopishdan keyingi indeks, tab limiti va aylanish, `path_label` (`/`, `C:\`), xatcho‘plar, palette holati va buyruqlar, PageUp/End, qidiruv mosligi, kalitlar takrorlanmasligi, uchala tilda tarjima, yorliqlar takrorlanmasligi.
- `cargo xtask check-boundaries`: OK (8 crate), `sysinfo` qoidasi tuzatilgandan keyin ([P3-08](P3-08-tomlar-f1-f2.md)).
- `cargo run -p commandor-app -- --smoke`: SMOKE OK.
- Vizual va qo‘lda sinov: ilova ishga tushirildi, tugmalar `osascript` bilan bosildi va faqat ilova oynasi suratga olindi. Tizim **light** mavzuda edi, shu sababli light mavzu ham birinchi marta ko‘rildi. Quyidagilar ishladi:
  - Cmd+T yangi tab ochdi;
  - ↓↓Enter faqat yangi tabda papkaga kirdi;
  - Cmd+Shift+P panelni ochdi, `tab` va `yangi tab` so‘rovlari to‘g‘ri filtrlandi, yorliqlar `Cmd+T` ko‘rinishida chiqdi;
  - paneldagi Enter buyruqni bajardi;
  - Cmd+D ★ni to‘ldirdi;
  - Cmd+W tabni yopdi, oyna yopilmadi.

  Birinchi avtomatik urinishda tugma bosishlar fokus yo‘qligi sababli boshqa oynaga ketgan. Bundan tashqari egui faqat hodisa bo‘lganda qayta chiziladi, fondagi oyna skrinshoti esa eski kadrni ko‘rsatgan bo‘lishi mumkin. Har qadamdan oldin fokus beriladigan qilib qayta sinalganda natijalar izchil chiqdi.

## Cheklov va qolgan ish

- Xatcho‘plar va ochiq tablar ilova qayta ishga tushganda saqlanmaydi. Bu TOML sozlamalar bilan qilinadi ([P3-07](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)); yorliqlarni qayta belgilash ham o‘sha vazifada.
- Quyidagilar skrinshotda ko‘rilmadi: ★ popup’i, tab qatori to‘lib aylanishi, avto-scroll (faqat kod va test darajasida tekshirildi) va dark mavzudagi yangi elementlar.
- Windows va Linuxda Ctrl yorliqlari sinalmagan. Ctrl+W/Ctrl+T tizim yorliqlari bilan to‘qnashmasligi tekshirilmagan.
- Tabni sudrab tartiblash va boshqa panelga ko‘chirish qilinmagan.
- Buyruqlar paneli faqat panel buyruqlarini qamraydi. F3–F8 amallari tayyor bo‘lganda qo‘shiladi (P3-04, P4).
