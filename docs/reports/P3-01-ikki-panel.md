# P3-01 — ikki panelli navigatsiya prototipi

> **Arxiv (2026-11-07):** bu hisobot o‘chirilgan Electron/TypeScript prototipiga tegishli. Kod [P0-03](P0-03-rust-qayta-shakllantirish.md) qarori bilan o‘chirildi, quyidagi kod havolalari endi ishlamaydi. Rust bo‘yicha yangi vazifalar [TODO](../../TODO.md)da.

- Sana: 2026-11-07
- Holat: jarayonda — kod va service testlari tayyor, desktop smoke kutilmoqda
- TODO: [P3-01](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11)
- Talablar: UX-01/03/04/06/07, FS-01/07ning boshlang‘ich qismi; bu IDlar to‘liq yopilgan emas

## Qilingan ish

[React interfeysi](../../src/renderer/App.tsx)da ikki mustaqil panel, faol panel belgisi, papka manzili, picker, home/up/reload, nom filtri, yashirin fayllar, metadata va bitta yozuv tanlovi yozildi. Papkaga double-click yoki Enter bilan kiriladi; ro‘yxatda ↑/↓, Backspace va Tab navigatsiyasi mavjud. Shift+Tab odatiy focus tartibini saqlaydi.

[O‘zbekcha UI matnlari](../../src/renderer/i18n/uz.ts) alohida resursga yig‘ildi. Loading, bo‘sh ro‘yxat va xato holatlari farqlanadi. Kechikkan eski so‘rov UI’dagi yangi natijani bosib ketmasligi uchun generation tekshiruvi bor.

[Fayl servisi](../../src/main/filesystem.ts) real diskdan asinxron o‘qiydi. Canonical path, oynaga tegishli opaque location/entry IDlari va directory identity tekshiriladi. Symlinklar belgilanadi; entry orqali kuzatilmaydi. Aniq kiritilgan manzil yoki native picker yangi location yaratadi. Fayl o‘zgartirish metodlari hali yo‘q.

## Tekshiruv

[Filesystem testlari](../../tests/filesystem.test.ts): real metadata, natural sort, hidden/filter, Unicode/bo‘shliq yo‘l, parent/child, panellararo IDning saqlanishi, begona ID, symlink sikli, almashtirilgan papka va noto‘g‘ri manzillar — 8/8 o‘tdi. Qo‘shimcha validation/xato testlari bilan jami 12/12.

[Desktop smoke](../../scripts/smoke.mjs) faqat sintetik temp fayllar bilan panel mustaqilligi, navigatsiya, filter/hidden, error/retry, Tab va IPC chegaralarini tekshirishga tayyorlandi. Haqiqiy natija Electron runtime o‘rnatilgach yoziladi.

## Cheklov va keyingi ish

- Listing 5 000 mos yozuv bilan cheklangan; truncate xabari bor, pagination va virtualizatsiya yo‘q. Ruxsat yetmagan ayrim entry metadatasi `—` ko‘rsatiladi.
- Bir oynada 256 location va har locationda 20 000 entry reference chegarasi bor; eskirgan ID uchun aniq xato qaytadi. Fayl tizimidagi poyga holatlariga qarshi mutatsiya darajasidagi kafolat berilmaydi.
- «Yashirin» hozir nomi `.` bilan boshlangan yozuvlar; Windows hidden atributi keyingi platforma ishiga kiradi.
- Multi-select, panel tablari, tarix, bookmarks, volume ro‘yxati, worker/watcher, rename/copy/move/trash hali yo‘q.
- Keyingi vazifalar: P3-02/P3-03; R0ning alohida ochiq ishlari P1-04/P2-01.
