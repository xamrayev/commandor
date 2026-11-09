# Commandor — texnik topshiriq

Versiya: 2.0 · Sana: 2026-11-07 · Holat: Rust implementatsiyasi uchun qayta shakllantirilgan loyiha talablari. Oldingi Electron prototipi o‘chirildi; hozircha kod yo‘q.

## 1. Maqsad va ko‘lam

Commandor foydalanuvchining diskdagi fayllarini ikki panelda boshqaradi, hujjatlarni ko‘rsatadi va tahrirlaydi, papkada terminal ochadi, lokal AI yordamida hujjatlarni topish va tushunishni osonlashtiradi. Windows, macOS va Linux bir xil asosiy imkoniyatlarga ega bo‘lishi kerak.

Ushbu hujjat dastur talablarini belgilaydi. 2026-11-07da foydalanuvchi qaroriga ko‘ra ilova **Rust**da (native GUI, bitta binary) yoziladi; texnologik yechim [arxitekturada](ARCHITECTURE.uz.md). Talab IDlari kuzatuvchanlik uchun saqlandi; amaldagi holat [TODO va hisobotlar](../TODO.md)da yuritiladi. Quyidagi “bo‘lishi kerak” bandlari qabul talablaridir; ularning hammasi bajarilgan degani emas.

### 1.1. Ustuvorlik va relizlar

- **R0 — texnik prototip:** platforma, Office integratsiyasi va kichik model bo‘yicha asosiy noaniqliklarni tekshirish.
- **R1 — lokal fayl menejeri:** ikki panel, fayl amallari, matn/PDF ko‘rish va matn tahriri, native Office preview/edit, terminal.
- **R2 — AI va hujjat qidiruvi:** indeks, lokal summary, provider adapterlari, model yuklash/runtime, AI rejalari, PDF asosiy tahriri.
- **R3 — integratsiyani yakunlash:** embedded Office qabul darvozasi, sifat va performance, installerlar, imzolash va release.
- **Keyingi:** semantik qidiruv, OCR, masofaviy fayl tizimlari, kengaytmalar.

### 1.1.1. Rustga o‘tish bilan imkoniyatlarning qayta shakllanishi

| O‘zgarish | Talabga ta’siri |
|---|---|
| Web engine yo‘q, UI native (egui) | Fayl mazmuni hech qachon HTML/JS sifatida bajarilmaydi; Markdown native render (TXT-04 kuchaydi). UI rasmlari, PDF va Office preview GPU teksturasi sifatida chiziladi |
| Bitta native binary + `--worker` child jarayoni | Parser/Office crash izolyatsiyasi (NFR-06) jarayon darajasida; Node/Electron runtime yuklash talabi yo‘q |
| Matn editori o‘zimiz quriladi (rope + syntect) | TXT-01–03 R1 ko‘lami: highlight, find/replace, undo/redo, line numbers, wrap, indent. Multi-cursor, LSP, avtomatik to‘ldirish — keyingi |
| PDF PDFium orqali | PDF-01 sahifa rasterlash; PDF-02 annotation/sahifa amallari PDFium/lopdf bilan. Backend R0.4da tasdiqlanadi |
| Ichki terminal Rust PTY (`portable-pty` + `alacritty_terminal`) | TERM-02–04 R2da saqlanadi; ANSI ranglar terminal emulator orqali |
| SQLite FTS5 binary ichida | IDX-04 uchun native ABI riski yo‘q; indeks R2da qoladi |
| LibreOfficeKit FFI orqali tile’larni native oynaga chizish imkoni | OFF-07 darvozasi o‘zgarmaydi, lekin R0.2 spike LOK FFI yo‘nalishini birinchi navbatda tekshiradi |
| Yangi: buyruq paneli (Ctrl/Cmd+Shift+P) | UX-08 sifatida qo‘shildi; barcha asosiy amallarni klaviaturadan topish |
| Yangi: sozlamalar fayli (TOML) va klaviatura xaritasi | UX-09; tashqi tahrir qilinsa ham ilova sxema bilan tekshiradi |

R1 yakuniy mahsulotning barcha talablarini qoplamaydi. Yakuniy qabul R1–R3 bandlari va alohida belgilangan Office darvozasi bo‘yicha amalga oshadi. AI o‘chirilganda R1 imkoniyatlari to‘liq ishlashi shart.

### 1.2. Maqsadli platformalar

| Platforma | Arxitektura | R0/R1 tekshiruvi | Reliz sharti |
|---|---|---|---|
| Windows | x64 | Oddiy foydalanuvchi, NTFS, PowerShell | Installer, Unicode/uzun yo‘l, native helper sinovlari |
| macOS | Apple Silicon va x64 | APFS, Terminal, ruxsatlar | Har ikkala build, signing/notarization tekshiruvi |
| Linux | x64 | Ubuntu LTS, ext4, X11 va Wayland | AppImage yoki distro paketi, terminal fallback, sandbox sinovi |

Aniq minimal OS versiyasi R0da Rust toolchain/GUI backend (wgpu/glow), LibreOffice, PDFium va llama.cppning tanlangan relizlari bilan aniqlanadi; Linuxda X11 va Wayland alohida sinaladi; hali tekshirilmagan versiyalarni qo‘llab-quvvatlanadi deb e’lon qilish mumkin emas. Windows ARM/Linux ARM qo‘shimcha maqsad, birinchi reliz majburiyati emas.

## 2. Foydalanuvchi interfeysi

Asosiy ekran quyidagi joylashuvni saqlaydi:

```text
┌ Commandor ───── Qidiruv ───────── Vazifalar ─ AI/model holati ─ Sozlamalar ┐
│ [Chap tablar]                         │ [O‘ng tablar]                    │
│ Disk / breadcrumb / manzil            │ Disk / breadcrumb / manzil     │
│ Nom              Tur   Hajm   Sana    │ Nom            Tur Hajm Sana   │
│ ..                                   │ ..                              │
│ Documents/                           │ Archive/                        │
│ hisobot.docx                         │ notes.md                        │
├───────────────────────────────────────┴────────────────────────────────┤
│ Ixtiyoriy pastki panel: terminal / amallar navbati                       │
├ F3 Ko‘rish · F4 Tahrirlash · F5 Nusxa · F6 Ko‘chirish · F7 Papka · F8 Savat ┤
└────────────────────────────────────────────────────────────────────────┘
  Ixtiyoriy yon panel: preview yoki AI; kattalashtiriladigan hujjat tabi
```

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| UX-01 | Ikki mustaqil panel, har birida papka/tab/tanlov/saralash holati | R1 | Chap navigatsiya o‘ng panel manzilini o‘zgartirmaydi |
| UX-02 | Faol panel aniq ajratiladi; Tab panelni almashtiradi | R1 | F5/F6 manbasi faol panel, maqsadi boshqa panel ekanligi operatsiyada ko‘rinadi |
| UX-03 | Breadcrumb, yoziladigan yo‘l, orqaga/oldinga/yuqoriga, bookmarks | R1 | Bo‘shliq va Unicode yo‘l navigatsiyada buzilmaydi |
| UX-04 | Ko‘p tanlash, range select, saralash va nom filtri | R1 | Filtr faylni diskdan o‘chirmaydi; tanlov soni ko‘rsatiladi |
| UX-05 | Klaviatura orqali barcha asosiy amallar, focus va screen-reader nomlari | R3 | Faqat klaviatura bilan fayl ochish, ko‘chirish va dialogdan chiqish mumkin |
| UX-06 | O‘zbekcha asosiy UI, ruscha/inglizcha tarjima resurslariga tayyor tuzilma | R1/R3 | Foydalanuvchi matnlari kodga tarqoq yozilmaydi; texnik xatolar tushunarli tarjima qilinadi |
| UX-07 | Loading, bo‘sh papka, ruxsat yo‘q, uzilgan disk va xato holatlari | R1 | Bo‘sh panel bilan xato holati bir xil ko‘rinmaydi |
| UX-08 | Buyruq paneli: amallarni nom bo‘yicha qidirish va bajarish | R1 | Har asosiy amal paneldan topiladi; tasdiq talab qiladigan amal tasdiqni chetlab o‘tmaydi |
| UX-09 | TOML sozlamalar va qayta tayinlanadigan klaviatura xaritasi | R1/R3 | Noto‘g‘ri sozlama fayli ilovani yiqitmaydi; xato ko‘rsatilib sukut qiymat ishlatiladi |

Oyna kengligi kichrayganda preview/AI yon paneli yig‘iladi; fayl panellari bir panel rejimiga o‘ta oladi. Yashirin fayllar alohida tugma bilan ko‘rsatiladi. Desktopdagi haqiqiy fayl yo‘li doim ko‘rinadigan bo‘lishi kerak.

## 3. Fayl boshqarish

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| FS-01 | Disklar, kataloglar, fayllar va metadata ko‘rsatish | R1 | Ruxsat berilgan lokal disklar ochiladi; unavailable volume tushunarli belgilanadi |
| FS-02 | Fayl/papkani nusxalash va ko‘chirish; navbat, progress, bekor qilish | R1 | Progress fayl/byte asosida; cancel yarim faylni to‘liq deb ko‘rsatmaydi |
| FS-03 | Rename, yangi papka, bo‘sh matn fayli yaratish | R1 | Noqonuniy nom OSga mos tekshiriladi; case-only rename alohida sinaladi |
| FS-04 | Mavjud maqsad konflikti: skip, keep both, replace | R1 | Replace aniq tanlanmaguncha asl fayl saqlanadi; batch qarori ko‘rinadi |
| FS-05 | Tizim savatiga tashlash; doimiy o‘chirish alohida amal | R1 | Savat bo‘lmasa avtomatik doimiy o‘chirish qilinmaydi |
| FS-06 | Fayl o‘zgarishlarini kuzatish, reload, tashqi edit konflikti | R1 | Eskirgan editor asl faylni jim bosib yozmaydi |
| FS-07 | Symlinklar va chegaralar | R1 | Link belgilanadi; rekursiv amallar sukut bo‘yicha linkni kuzatmaydi; siklga kirmaydi |
| FS-08 | Amallar jurnali, muvaffaqiyat/xato, tiklashga yaroqlilik | R2 | Ko‘p faylli ishning qaysi qismi tugagani va qaysi biri qolganini aniqlash mumkin |
| FS-09 | ZIP ko‘rish/ajratish | Keyingi | Path traversal, symlink va zip-bomb limitlari bilan |
| FS-10 | SFTP/SMB/WebDAV virtual filesystem adapterlari | Keyingi | Lokal MVP uchun bog‘liqlik emas |

Bir diskdagi rename bilan disklararo move farqlanadi: ikkinchisida copy tugab, hajm/integrity tekshirilgandan keyin manba o‘chiriladi. Ko‘p faylli operatsiyalar global atomik tranzaksiya deb ko‘rsatilmaydi. Undo faqat haqiqatan qaytariladigan amallar uchun chiqadi.

## 4. Matn, Markdown, YAML va PDF

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| TXT-01 | UTF-8 matnni ko‘rish/tahrirlash/Save/Save As | R1 | BOM va CRLF/LF saqlanishi; dirty indikator; yopishda save/discard/cancel |
| TXT-02 | Syntax highlighting: Markdown, YAML, JSON, JS/TS, Python, Rust, C/C++, Java, Go, HTML/CSS, SQL, shell | R1 | Til extension asosida va qo‘lda tanlanadi; kod hech qachon avtomatik bajarilmaydi |
| TXT-03 | Find/replace, line numbers, undo/redo, indent, wrap | R1 | Unicode, multiline matn va katta qatorlarda to‘g‘ri ishlash |
| TXT-04 | Markdown source/preview/split | R1 | Xom HTML va skript bajarilmaydi; tashqi resurslar sukut bo‘yicha yuklanmaydi |
| TXT-05 | YAML/JSON xatolarini ko‘rsatish | R2 | Parser custom tag orqali kod bajarmaydi; xatoga qaramay originalni saqlash mumkin |
| TXT-06 | Katta fayl: read-only bo‘laklab ko‘rish, encoding tanlash | R2 | Limitdan kattani to‘liq RAMga yuklamaydi; noma’lum encoding jim buzilmaydi |
| PDF-01 | Sahifalar, zoom, qidiruv, tanlash, matn nusxalash | R1 | 100 sahifali PDF virtual sahifalar bilan ochiladi |
| PDF-02 | Highlight, matnli izoh, sahifa aylantirish/o‘chirish/qayta tartiblash va Save As | R2 | Natija boshqa PDF viewerda qayta ochiladi; original saqlanadi |
| PDF-03 | Parolli, buzilgan, skanerlangan PDF holatlari | R1/R2 | Parol faqat sessiyada; OCR yo‘q bo‘lsa “matn qatlami yo‘q” deyiladi |

**PDF tahriri chegarasi:** ushbu bazaviy ko‘lam izoh va sahifa amallaridir. Mavjud PDF matn/paragraflarini Word kabi erkin qayta joylashtirish, to‘liq font almashtirish, raqamli imzo yaratish va ishonchli redaction alohida kengaytma hisoblanadi. Highlight yoki qora to‘rtburchak redaction deb atalmasligi kerak. Imzolangan PDFni o‘zgartirish imzaga ta’sir qilishi mumkinligi Save As oldidan aytiladi. Bu scope foydalanuvchining keng “PDF tahriri” talabiga taklif qilingan aniq talqindir.

## 5. Office va ikki bosqichli ishga tushirish

Tasdiqlangan dvigatel oilasi: **LibreOffice**. Word/Excel/PowerPoint deganda birinchi navbatda **DOCX/XLSX/PPTX** formatlari tushuniladi; Microsoft ilovalari o‘rnatilgan bo‘lishi talab qilinmaydi.

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| OFF-01 | DOCX/XLSX/PPTXni faqat preview rejimida ochish | R1 | Ochishda GUI editor jarayoni ishga tushmaydi; cached preview tezda ko‘rsatiladi |
| OFF-02 | Headless render → PDF/rasm sahifalari, worker va cache | R1 | UI javob beradi; timeout/crash UIga tarqalmaydi; render manba faylni o‘zgartirmaydi |
| OFF-03 | Faqat Edit bosilganda tahrirlash dvigateli/session boshlanadi | R1 | Editdan oldin editor moduli/sessioni yo‘q; holatlar UI orqali ko‘rinadi |
| OFF-04 | Saqlash, Save As, tashqi o‘zgarish konflikti va preview yangilanishi | R1 | Edit → save → qayta previewda yangi mazmun ko‘rinadi; eski cache qaytarilmaydi |
| OFF-05 | Writer: matn, uslub, jadval, rasm; Calc: katak, formula, format, sheet; Impress: slayd, matn, rasm, tartib | R1 native / R3 embedded | Har bir format uchun fixture ochish-edit-save-qayta ochishdan o‘tadi |
| OFF-06 | Native LibreOffice oynasi orqali fallback | R1 | Ilova uning tashqi oynada ochilganini aniq bildiradi; bu embedded deb belgilanmaydi |
| OFF-07 | Ilova ichidagi Office editor | R3 darvoza | Uch OSda klaviatura, IME, clipboard, save, crash recovery va lazy launch isbotlanadi |
| OFF-08 | Makrolar/external linklar va begona kontent | R1 | Preview worker makro bajarmaydi; tashqi resurslarni yuklash boshqariladi |
| OFF-09 | Legacy DOC/XLS/PPT va ODT/ODS/ODP | R2 | Alohida compatibility matritsasida natija ko‘rsatiladi |

**Ikki bosqich — ikkita alohida o‘rnatish degani emas.** Ilova va preview qismi avval ishlaydi. Office preview yaratish uchun headless LibreOffice baribir kerak bo‘lishi mumkin; “render mutlaqo yengil” kafolati berilmaydi. To‘liq editor UI va tahrir sessiyasi Edit harakatigacha ishga tushmaydi. Cache bor bo‘lsa preview hech qanday Office jarayonini boshlamaydi.

O‘rnatilmagan LibreOffice holatida ilova uni topish/o‘rnatish bo‘yicha yo‘l ko‘rsatadi. Avtomatik katta runtime yuklash foydalanuvchi harakatisiz bo‘lmaydi. Ilova foydalanuvchining boshqa ochiq LibreOffice oynalarini yopmasligi yoki profilini egallamasligi kerak.

**Embedded bo‘yicha muhandislik darvozasi:** R0da LibreOfficeKit/native bridge prototipi tekshiriladi; zarur bo‘lsa LibreOffice oilasidagi web editor adapteri alohida baholanadi. Native LibreOffice oynasini ilova oynasiga oddiy reparent qilish yechim deb qabul qilinmaydi; birinchi nomzod — LibreOfficeKit C API (FFI) orqali tile render va input uzatish. Uch OSda ishonchli embedded natija bo‘lmasa OFF-07 bajarilmagan hisoblanadi; native fallbackni yakuniy embedded natijaga tenglashtirish mumkin emas. Shu nuqtada resurs/ko‘lam qarori olinadi.

### 5.1. Office mosligi

| Format | Asosiy qabul | Alohida tekshiriladigan xususiyat |
|---|---|---|
| DOCX | Matn, uslub, jadval, header/footer, rasm, sahifalar | Track changes, murakkab layout, o‘rnatilmagan shrift |
| XLSX | Formula, format, sheet, basic chart | Pivot, external link, katta workbook, Excelga xos formulalar |
| PPTX | Slayd, matn, rasm, shakl, notes | Animatsiya, transition, video, maxsus shrift |
| DOCM/XLSM/PPTM | Dastlab read-only yoki aniq rad etish | Makro bajarish ko‘lamga kirmaydi; jim macro yo‘qotish mumkin emas |

Microsoft Office bilan pikselma-piksel tenglik va barcha format xususiyatlarining yo‘qotishsiz saqlanishi oldindan kafolatlanmaydi. Mos kelmagan xususiyat bo‘lsa originalni saqlash va yangi nusxaga yozish yo‘li beriladi.

## 6. Terminal

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| TERM-01 | Har qanday kirish ruxsati mavjud papkada tizim terminalini ochish | R1 | `pwd`/`Get-Location` aynan tanlangan katalogni qaytaradi |
| TERM-02 | Ichki PTY terminal, tab va resize | R2 | Interaktiv shell, Ctrl+C va terminalga mos ranglar ishlaydi |
| TERM-03 | Shell tanlash: PowerShell/cmd, zsh/bash va mavjud executable | R2 | Yo‘l argument sifatida uzatiladi; fayl nomi shell kodiga aylanmaydi |
| TERM-04 | Terminal holati va sessiyani yopish | R2 | Oyna yopilganda faqat unga tegishli PTY child jarayonlari boshqariladi |

“Istalgan papka” OS ruxsatlari doirasida talqin qilinadi. AI terminalga erkin buyruqni avtomatik yubormaydi. Oddiy foydalanuvchi terminalga o‘zi yozgan buyruqlar standart shell vakolatida ishlaydi.

## 7. AI provayderlari va AI orqali boshqarish

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| AI-01 | Ollama ulanishi, model ro‘yxati, ulanish testi | R2 | Server/model yo‘q holati tushunarli; local fayllar avtomatik yuborilmaydi |
| AI-02 | Boshqariladigan llama.cpp va mavjud local compatible server | R2 | Health-checkdan keyingina ready; noto‘g‘ri endpointda aniq xato |
| AI-03 | Tashqi OpenAI-compatible yoki custom adapter | R2 | URL, model, maxfiy kalit; provider imkoniyatlari dinamik; har provider bir xil deb faraz qilinmaydi |
| AI-04 | Chat, streaming, cancel, conversation reset | R2 | UI oqim davomida ishlaydi; bekor qilinganda so‘rov to‘xtaydi |
| AI-05 | Tanlangan hujjatni tushuntirish/summary | R2 | Qaysi fayl yoki parcha ishlatilgani va qaysi model ko‘rsatiladi |
| AI-06 | Tabiiy tilda fayl qidirish | R2 | So‘rov tekshiriladigan filtr/qidiruvga aylantiriladi; natija real indeksdan olinadi |
| AI-07 | Copy/move/rename/mkdir/trash uchun amal rejasi | R2 | Manba, maqsad, konflikt va fayllar soni ko‘rsatiladi; tasdiqsiz mutatsiya yo‘q |
| AI-08 | Summary/indekslash uchun local-only siyosat | R2 | Remote endpoint va redirect rad etiladi; lokal server ishlamasa bulutga fallback yo‘q |
| AI-09 | Maxfiy kalit va loglar | R2 | Kalit OS secret storeda; hujjat mazmuni va tokenlar odatiy logga yozilmaydi |

AI tool allowlist: `list_directory`, `search_documents`, `read_excerpt`, `preview_file`, `propose_copy`, `propose_move`, `propose_rename`, `propose_mkdir`, `propose_trash`. Mutatsiya bajarish funksiyasi to‘g‘ridan-to‘g‘ri modelga berilmaydi. Tasdiqlangan reja xeshi va fayl revisionlari qayta tekshiriladi. Model JSONni noto‘g‘ri tuzsa cheklangan retrydan keyin oddiy UI amali taklif qilinadi.

Model “men faylni ko‘chirdim” deb yozgani bajarilganlik dalili emas: UI faqat operation servicening real natijasini ko‘rsatadi. Hujjatlar ichidagi ko‘rsatmalar tizim buyruqlari sifatida qabul qilinmaydi. Lokal server manzili fayl ma’lumoti tashqariga chiqmasligini yolg‘iz o‘zi kafolatlamaydi; foydalanuvchi ulangan serverni boshqarishi kerak, ilovaning o‘zi remote fallback qilmaydi.

## 8. Qidiruv, matn ajratish va summary

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| IDX-01 | Tanlangan disk/papkalarni qo‘shish, chiqarish va istisnolar | R2 | Butun disk foydalanuvchi tanlamaguncha yashirincha skanerlanmaydi |
| IDX-02 | Nom/path/tur/sana/hajm bo‘yicha qidiruv | R2 | UI filtrlar va ildiz doirasini ko‘rsatadi |
| IDX-03 | Matn, PDF text-layer va Office mazmunini ajratish | R2 | DOCX paragraflar, XLSX sheet/katak, PPTX slayd/notes manzillari yo‘qolmaydi |
| IDX-04 | SQLite FTS5 orqali content va summary qidiruvi | R2 | Natija highlight, manba yo‘li va fragment bilan; original faylga o‘tish mumkin |
| IDX-05 | Lokal AI summary yaratish va saqlash | R2 | Summary qayta ishga tushganda mavjud; hash/model/prompt/extractor versiyasi qayd etiladi |
| IDX-06 | Incremental update, rename/delete va stale holat | R2 | O‘chgan fayl “mavjud” deb chiqmaydi; o‘zgargan hujjat summarysi stale deyiladi |
| IDX-07 | Pause/cancel/resume, progress, xatolar statistikasi | R2 | Bitta buzilgan/parolli fayl navbatni to‘xtatmaydi |
| IDX-08 | Indeksdan rootni olib tashlash va cache purge | R2 | Shu rootdan olingan matn/summary qoldiqlari tegishli jadvallardan tozalanadi |
| IDX-09 | OCR va semantik qidiruv | Keyingi | Alohida model/runtime; sifat va xotira o‘lchovi; oddiy qidiruvdan mustaqil |

Sukut bo‘yicha `.git`, dependency kataloglari, OS cachelari, credential kataloglari va foydalanuvchi exclude patternlari indekslanmaydi. Symlink bo‘ylab rootdan chiqish taqiqlanadi. Tarmoq yoki chiqarib olingan diskdagi eski yozuv “offline” sifatida ko‘rsatilishi mumkin, lekin mazmunini ko‘rsatish saqlash siyosati va joriy ruxsatga mos bo‘ladi.

Summary bosqichi: matn ajratish → tokenizer bilan bo‘laklash → bo‘lak summarylari → umumlashtirish. Har bo‘lak uchun manba lokatori saqlanadi. Kerakli bo‘laklar cheklov sabab tashlab ketilgan bo‘lsa coverage `<100%` ko‘rsatiladi. Kichik modelning xulosasi hujjat o‘rniga rasmiy haqiqat deb belgilanmaydi; manbaga qaytish yo‘li mavjud.

Avtomatik summary rejimi foydalanuvchi yoqqanidan keyin boshlanadi. Faol chat/editor ishlaganda fon summary navbati pasayadi; batareya va RAM bosimi bo‘lsa pauza qilinadi. Embedding modeli generativ summary modelidan alohida tanlanadi; Qwen 2B avtomatik ravishda embedding modeli deb olinmaydi.

## 9. Hugging Face model menejeri va llama.cpp

| ID | Talab | Reliz | Qabul mezoni |
|---|---|---|---|
| MODEL-01 | Qurilma RAM/CPU/GPU/disk imkoniyatini aniqlash | R2 | Aniq aniqlanmagan VRAM “noma’lum”; taxmin fakt sifatida ko‘rsatilmaydi |
| MODEL-02 | Kichik GGUF modellar katalogi va tavsiyalar | R2 | Manba, quantization, hajm, litsenziya, runtime mosligi va sabab ko‘rinadi |
| MODEL-03 | Hugging Face’dan yuklash, progress, cancel/resume | R2 | `.part` fayl incomplete; resume revision/ETag bilan moslashtiriladi |
| MODEL-04 | Revision pin, SHA-256 va fayl formati tekshiruvi | R2 | Mismatchda model “installed” emas; LFS pointer model sifatida ochilmaydi |
| MODEL-05 | Lokal GGUF import, modelni o‘chirish va joyni ko‘rsatish | R2 | Ishlayotgan modelni o‘chirishdan oldin to‘xtatiladi |
| MODEL-06 | llama.cpp start/stop/restart, context/GPU sozlamalari | R2 | Lokal bind, health-check, timeout, port konflikti va OOM holatlari boshqariladi |
| MODEL-07 | Runtime tanlash va versiyalash | R2/R3 | OS/arch/backend mos; model support minimum build bilan tekshiriladi |
| MODEL-08 | Model yuklash internetdan alohida, inference offline | R2 | Model/runtime o‘rnatilgach offline testda summary ishlaydi |

Boshlang‘ich katalog nomzodi: **Qwen3.5-2B, GGUF Q4_K_M**, [Unsloth repozitoriydagi aniq fayl](https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/blob/main/Qwen3.5-2B-Q4_K_M.gguf). Original model manbasi — [Qwen](https://huggingface.co/Qwen/Qwen3.5-2B). Ularning mualliflik va konvertatsiya manbalari alohida ko‘rsatiladi. Bu tavsiya qilingan boshlang‘ich nomzod, o‘zbekcha sifat/tezlik bo‘yicha hali benchmarkdan o‘tmagan.

Qurilmaga mos tavsiya uchun boshlang‘ich sinov katalogi:

| Nomzod | Taklif etiladigan sinov vazifasi | Muhim cheklov |
|---|---|---|
| [Qwen3.5-0.8B Q4_K_M](https://huggingface.co/unsloth/Qwen3.5-0.8B-GGUF/blob/main/Qwen3.5-0.8B-Q4_K_M.gguf) | Eng kam resursli matn tasnifi, qisqa metadata | Murakkab reja va uzun summary sifati alohida o‘lchanadi |
| [Qwen3.5-2B Q4_K_M](https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/blob/main/Qwen3.5-2B-Q4_K_M.gguf) | Asosiy local summary/search-intent nomzodi | Sahifada 2026-11-06 tekshiruvda taxminan 1.28 GB artifact ko‘rsatilgan; bu jami RAM sarfi emas |
| [Qwen3.5-4B Q4_K_M](https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/blob/main/Qwen3.5-4B-Q4_K_M.gguf) | Ko‘proq bo‘sh RAM bo‘lsa sifatni solishtirish | Kattaroq model har vazifada yaxshiroq degan kafolat yo‘q; latency va xotira benchmarki kerak |

Bu vazifa taqsimoti loyihaviy gipoteza, manbalarda isbotlangan sifat reytingi emas. Tavsiya haqiqiy bo‘sh xotira, tanlangan context va o‘lchangan profilga qarab chiqariladi; foydalanuvchi barcha modelni yuklashi shart emas.

Model tanlashdagi `4q` talabi amalda aniq quantization nomi bilan ifodalanadi. Xotira bahosi `vazn + context/KV + runtime buferlari + OS/ilova zaxirasi`; faqat fayl hajmi bo‘yicha RAM yetadi deb aytilmaydi. Dastlab 4096 context taklif qilinadi, model runtime mosligiga qarab tekshiriladi. Vision/mmproj va katta context alohida qo‘shimcha, matn summarysi uchun majburiy emas.

## 10. Nofunksional talablar

Quyidagilar o‘lchangan natija emas, qabul uchun **boshlang‘ich maqsadlar**. R0 baseline qurilmalari: 16 GB RAM, SSD, qo‘llab-quvvatlanadigan OS; minimal 8 GB konfiguratsiya alohida sinovdan o‘tadi. AI/Office yuklari alohida hisoblanadi.

| ID | Talab va maqsad | O‘lchash usuli |
|---|---|---|
| NFR-01 | Cold start: asosiy panel p95 ≤ 1.5 s | Har OSda 20 ishga tushirish; indeks/AI/Office autostartsiz |
| NFR-02 | 10 000 yozuvli lokal katalog: birinchi ko‘rinish p95 ≤ 1 s, incremental listing | SSD test katalogi; butun metadata tayyorligini kutmaslik |
| NFR-03 | 100 000 indekslangan hujjat: iliq FTS so‘rovi p95 ≤ 300 ms | Belgilangan fixture va 100 so‘rov; indekslash paytidagi natija alohida |
| NFR-04 | UI threadda asosiy interaksiya payti >16 ms kadr va >100 ms bloklanish yo‘q | `tracing`/profiler (puffin yoki Tracy); IO, render, index va hash UI threaddan tashqarida |
| NFR-05 | Idle bazaviy ilova RSS maqsadi ≤ 150 MB; installer hajmi (PDFiumsiz) ≤ 40 MB maqsad | Barcha asosiy child jarayonlari yig‘indisi; Office/AI alohida ko‘rsatiladi |
| NFR-06 | Render worker crashi asosiy ilovani yopmaydi | Workerga majburiy crash berib UI va navbatni kuzatish |
| NFR-07 | Uzoq ish canceli UIga ≤ 1 s ichida aks etadi | Download/index/copy/inference uchun; cleanup davomiyligi alohida |
| NFR-08 | Offline core va local AI | Tarmoq uzilgan holda R1 hamda o‘rnatilgan model bilan R2 sinovlari |
| NFR-09 | Cache kvotasi: boshlang‘ich preview 1 GB, sozlanadigan | LRU tozalash; faol/dirtied hujjat yoki recovery fayli o‘chirilmasligi |
| NFR-10 | Avtomatik crash loglarida kalit/hujjat matni yo‘q; telemetry sukut bo‘yicha o‘chiq | Secret canary va network inspection |

AI token/s yoki Office render vaqti tekshirilmagan apparat uchun kafolatlanmaydi. R0 profilingdan keyin har model/backend uchun o‘lchangan jadval tayyorlanadi. Maqsad bajarilmasa release oldidan optimallashtirish yoki ochiq scope/baseline o‘zgarishi kerak.

## 11. Yakuniy yetkazib beriladiganlar

Implementatsiya bosqichining kutiladigan natijalari: manba kod, dependency lockfile/SBOM, uch OS installer, sozlash va foydalanish qo‘llanmasi, migratsiya/recovery qo‘llanmasi, fixturelar, test hisobotlari, model katalogi, release note va ma’lum cheklovlar ro‘yxati. Yetkazib berilgan qismlar [TODO](../TODO.md) orqali kuzatiladi.

Umumiy qabul: R1–R3 majburiy IDlari [sinov rejasiga](DELIVERY.uz.md) bog‘langan; bajarilmagan band yashirilmaydi; data loss bo‘yicha ochiq kritik xato yo‘q; barcha “tayyor” funksiyalar uch OSda real test daliliga ega. Embedded Office bajarilmasa u alohida ochiq scope qarori bo‘lib qoladi.
