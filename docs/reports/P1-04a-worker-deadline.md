# P1-04a — worker javobiga umumiy deadline

- Sana: 2026-11-07
- Holat: bajarildi
- TODO: [P1-04a](../../TODO.md#1-faza--rust-platforma-asosi-r01)

## Topilgan muammo va tuzatish

[Supervisor](../../crates/commandor-ipc/src/supervisor.rs) har kutilmagan yoki eski xabardan keyin `recv_timeout(timeout)`ni to‘liq muddat bilan qayta chaqirardi. Worker muntazam begona IDli xabar yuborsa, haqiqiy javob hech kelmasa ham kutish cheksiz cho‘zilishi mumkin edi.

Endi javob kutish boshlanganda `Instant` olinadi va har takrorlashda faqat qolgan muddat ishlatiladi. Deadline tugaganda mavjud `reset` yo‘li workerni to‘xtatadi; keyingi request restart siyosati bilan ishlaydi.

## Tekshiruv

Yangi regression testida begona IDli xabarlar har 10 msda yuboriladi; mos javob 200 msdan keyin keladi. 30 ms deadline bilan natija `Timeout` bo‘ladi. Oldingi 6 ta haqiqiy worker jarayoni testi (ping/Unicode, diag policy, crash/restart, timeout, restart limit, usage) ham o‘tdi.

Yakuniy workspace natijasi: 40/40 test, clippy `-D warnings`, fmt va dependency boundary check o‘tdi. [Umumiy tekshiruv hisoboti](P3-01-rust-listing.md).

## Chegara

Bu tuzatish **javob kutish** muddatiga tegishli. `write_frame`ning stdin pipe’ga bloklovchi yozuvi uchun deadline/cancel alohida hardening talab qiladi; ushbu patch end-to-end request timeout kafolati deb talqin qilinmaydi. Katta worker payloadlari qo‘shilishidan oldin shu yo‘l tekshirilishi kerak.
