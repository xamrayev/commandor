# P1-01–03 — desktop platforma asosi

> **Arxiv (2026-11-07):** bu hisobot o‘chirilgan Electron/TypeScript prototipiga tegishli. Kod [P0-03](P0-03-rust-qayta-shakllantirish.md) qarori bilan o‘chirildi, quyidagi kod havolalari endi ishlamaydi. Rust bo‘yicha yangi vazifalar [TODO](../../TODO.md)da.

- Sana: 2026-11-07
- Holat: jarayonda — build va unit testlar o‘tdi, desktop smoke kutilmoqda
- TODO: [1-faza](../../TODO.md#1-faza--platforma-asosi-r01)
- Qamrov: R0.1ning lokal desktop/infratuzilma qismi; barcha platformalar uchun qabul emas

## Qilingan ish

- [package.json](../../package.json) va [lockfile](../../package-lock.json): Electron 45.2.3, React 19.3.1, TypeScript 7.0.2, Vite 8.3.6 va aniq pinlangan bog‘liqliklar.
- [Build/dev](../../scripts/build.mjs): main/preload CommonJS bundle, React production bundle; [ishga tushirish](../../README.md#ishni-boshlash).
- [Main](../../src/main/index.ts): sandbox, context isolation, Node integratsiyasiz renderer, lokal `commandor://app` protocol, navigation/popup/webview va permission cheklovlari.
- [Preload](../../src/preload/index.ts): faqat `resolveLocation`, `chooseDirectory`, `list`; rendererga erkin `invoke` berilmaydi.
- [Kontraktlar](../../src/shared/contracts.ts), [validation](../../src/main/validation.ts) va [xatolar](../../src/main/errors.ts): API version/request/window envelope, argument/sender tekshiruvi, yo‘l/stackni oshkor qilmaydigan xato kodlari.
- [AGENTS.md](../../AGENTS.md): keyingi sessiyalarda ham TODO/hisobot tartibini davom ettirish ko‘rsatmasi.

## Tekshiruv

Muhit: macOS arm64, Node.js 25.9.0, npm 11.12.1.

| Tekshiruv | Natija |
|---|---|
| `npm run typecheck` | O‘tdi |
| `npm test` | 12/12 o‘tdi; 0 failed/skipped |
| `npm run build` | O‘tdi; `dist/main.cjs`, `dist/preload.cjs`, `dist/renderer/` |
| `npm run test:desktop` | Electron runtime yuklanmoqda; natija hali yo‘q |

Testlarda faqat sintetik temp papkalar ishlatiladi. Tekshiruv davomida TypeScript CSS import deklaratsiyasi tuzatildi. `tsx` CLI uchun sandboxdagi IPC cheklovini talab qilmaydigan `node --import tsx --test`ga o‘tildi.

## Cheklov va keyingi ish

P1-04 ochiq: Windows/Linux/macOS x64, native SQLite/PTY bindinglar va CI hali sinalmagan. Installer/signing, performance hamda to‘liq security acceptance hali yo‘q. Dev rejimi hozir build → launch; hot reload yo‘q.

IPC va sandbox qarorlari [Electron security](https://www.electronjs.org/docs/latest/tutorial/security) va [sandbox hujjati](https://www.electronjs.org/docs/latest/tutorial/sandbox) bilan tekshirildi. Build muhiti [Vite qo‘llanmasi](https://vite.dev/guide/)ga mos tekshirildi.
