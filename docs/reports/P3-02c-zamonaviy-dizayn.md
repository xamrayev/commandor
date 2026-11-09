# P3-02c — interfeysni zamonaviy dizaynda qayta yaratish

- Sana: 2026-11-07
- Holat: bajarildi
- TODO: [P3-02c](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-01, UX-03, UX-04, UX-07 (vizual qatlam; xatti-harakat o‘zgarmadi)

## Qilingan ish

Holat ([`state.rs`](../../crates/commandor-ui/src/state.rs)), broker bilan aloqa, klaviatura boshqaruvi va smoke mantiqi o‘zgarmadi. Faqat ko‘rinish qatlami qayta yozildi.

1. **Mavzu tizimi.** [`theme.rs`](../../crates/commandor-ui/src/theme.rs): dark va light `Palette` (fon, karta, accent, success/warning/danger, qator holatlari, ikonka ranglari). `install()` ikkala `Theme` uchun `Style`ni o‘rnatadi: tipografika shkalasi (18/14/11.5), bo‘shliqlar, 7–12 px radiuslar va soyalar. Ilova tizim mavzusiga ergashadi.
2. **Vizual elementlar.** [`widgets.rs`](../../crates/commandor-ui/src/widgets.rs): pill, broker holati yorlig‘i, ikonka-tugma, accent tugma, toggle chip, `F5` ko‘rinishidagi klaviatura belgisi. Papka, fayl, havola, tanlov va saralash strelkasi vektor sifatida chiziladi, shrift glifiga bog‘liq emas.
3. **Joylashuv.** [`lib.rs`](../../crates/commandor-ui/src/lib.rs):
   - yuqori panel: logotip, papka tanlash tugmasi, rangli nuqtali broker holati;
   - panel kartalari: faol panelda accent ramka va yorug‘lik soyasi, `faol` belgisi, ikonkali navigatsiya (tooltip — i18n nomi);
   - yo‘l maydoni accent `Ochish` tugmasi bilan; filtr va yashirin fayllar chipi bitta qatorda;
   - jadval: saralanadigan sarlavha, qatorlar to‘liq qo‘lda chiziladi (zebra, hover, kursorda accent chiziq, tanlovda warning fon va belgi). Uzun nomlar `…` bilan qisqartiriladi. Panel 430 px’dan tor bo‘lsa sana ustuni yashiriladi;
   - holatlar: yuklanish spinneri, xato kartasi va `Qayta urinish`, bo‘sh papka, cheklangan ro‘yxat ogohlantirishi;
   - pastki qism: kartada yozuvlar soni va tanlov statistikasi; oynada F-tugma chiplari va `Tab` maslahati.

Yangi i18n kalitlari qo‘shilmadi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: ogohlantirishsiz.
- `cargo test --workspace`: 51 o‘tdi, 0 yiqildi (yangi: `theme::tests::soft_colors_are_premultiplied`, `install_sets_both_themes`).
- `cargo xtask check-boundaries`: OK (8 crate). UI crate hali ham `std::fs`/`process`/tarmoqqa kirmaydi.
- `cargo run -p commandor-app -- --smoke`: SMOKE OK.
- Vizual tekshiruv: oyna skrinshoti, faqat dark mavzu. Dastlabki variantda `▲` glifi va emoji papka ikonkasi noto‘g‘ri chiqdi; ular vektor chizmaga almashtirildi va qayta tekshirildi.

## Cheklov va qolgan ish

- Light mavzu vizual tekshirilmagan; Windows/Linux’da ko‘rinish sinalmagan.
- Tanlangan qator, xato va bo‘sh holatlar skrinshotda ko‘rilmagan. Ular faqat kod va kompilyatsiya darajasida tekshirilgan.
- Standart egui shrifti (Ubuntu-Light) qoldirildi. Inter kabi shrift qo‘shish litsenziya inventari bilan birga hal qilinadi (P8-04).
- Klaviatura bilan kursor ko‘rinmas qatorga o‘tsa, ro‘yxat avtomatik aylantirilmaydi. Bu keyingi UX ishi ([P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)).
