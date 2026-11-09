# P1-05a — GitHub repo va uch OS CI’ning birinchi ishga tushishi

- Sana: 2026-11-09
- Holat: bajarildi (build/lint/test qismi). P1-05 ning GUI smoke va prototip qismlari ochiq
- TODO: [P1-05a](../../TODO.md#1-faza--rust-platforma-asosi-r01), [P1-05](../../TODO.md#1-faza--rust-platforma-asosi-r01)
- Talablar: infratuzilma (R0.1 chiqish mezoni: uch OSda build va testlar)

## Qilingan ish

1. Loyiha git repozitoriysiga aylantirildi va public repo sifatida e’lon qilindi: <https://github.com/xamrayev/commandor> (`main`).
   - `target/` `.gitignore` orqali chiqarib tashlangan.
   - Push’dan oldin sir va katta fayllar qidirildi, hech narsa topilmadi.
2. README uch tilda yozildi: asosiysi [inglizcha](../../README.md), qo‘shimcha [ruscha](../../README.ru.md) va [o‘zbekcha](../../README.uz.md).
3. Litsenziya: [Apache-2.0](../../LICENSE). Matn apache.org’dan olingan rasmiy nusxa (SHA-256 `cfc7749b…3d30`). Workspace’da `license = "Apache-2.0"` qo‘yildi.
4. [CI](../../.github/workflows/ci.yml) yangilandi:
   - `macos-14` va `macos-13` obrazlari GitHub’da endi ishlamaydi. Birinchi yugurishda `macos-14` "configuration is not supported" xatosini berdi, `macos-13` esa navbatda qotib qoldi. Ular `macos-15` (arm64) va `macos-15-intel` (x86_64) bilan almashtirildi.
   - `actions/checkout` v4 dan v5 ga ko‘tarildi, chunki v4 eskirgan Node 20 ogohlantirishini berardi.

## Tekshiruv

Yugurish [42573662256](https://github.com/xamrayev/commandor/actions/runs/42573662256), commit `2a3e4f2`, Rust 1.94.1. Har bir job’da `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -D warnings`, `cargo test --workspace --locked` va `cargo xtask check-boundaries` bajarildi:

| Runner | Natija | Vaqt |
|---|---|---|
| ubuntu-24.04 | ✓ | 3 min 48 s |
| windows-2022 | ✓ | 3 min 26 s |
| macos-15 (arm64) | ✓ | 2 min 32 s |
| macos-15-intel | ✓ | 4 min 39 s |

Birinchi yugurish ([42573158590](https://github.com/xamrayev/commandor/actions/runs/42573158590), commit `9e414a3`): Ubuntu va Windows ✓, macOS runnerlari yuqoridagi sabablarga ko‘ra ishlamadi, keyin yugurish bekor qilindi.

Shunday qilib, Linux va Windowsdagi testlar — jumladan worker supervisor’ning haqiqiy jarayon testlari, volume filtrlari va listing fixturelari — birinchi marta haqiqiy OSda o‘tdi.

## Cheklov va qolgan ish

- Runnerlarda GUI smoke (`commandor-app -- --smoke`) ishlatilmaydi, chunki grafik sessiya kerak (Linux’da `xvfb`, Windows/macOS runner sessiyasi). Oyna faqat lokal macOS arm64da ochilgan. Bu ish P1-05 ochiq qismida qoladi.
- rusqlite FTS5 va portable-pty prototiplari hali yo‘q (P1-05).
- Light/dark mavzu va UI ko‘rinishi Windows va Linux’da vizual tekshirilmagan.
