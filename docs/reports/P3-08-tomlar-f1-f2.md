# P3-08 — tomlar (disklar) ro‘yxati va F1/F2

- Sana: 2026-11-09 (kod oldingi sessiyada yozilgan, hisobotsiz qolgan; bu sessiyada tekshirildi va qayd etildi)
- Holat: bajarildi (macOS arm64)
- TODO: [P3-08](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11), [P3-02](../../TODO.md#3-faza--ikki-panelli-fayl-menejeri-r11) ning "volume ro‘yxati" qismi
- Talablar: UX-08 (volume ro‘yxati), FS-01

## Qilingan ish

- [`commandor-fs/src/volumes.rs`](../../crates/commandor-fs/src/volumes.rs): `sysinfo` (faqat `disk` feature) orqali tomlar; OS bo‘yicha `MountRules` filtri (macOS `/` va `/Volumes/*`, Linux `/proc`, `/sys`, `/run` kabilarsiz, Windows barcha drayvlar), tur aniqlash (System/Internal/Removable/Network), nom fallback va saralash.
- [`commandor-core/src/filesystem.rs`](../../crates/commandor-core/src/filesystem.rs): `Volume`, `VolumeKind`, `volume_index_for_path` (komponent chegarasini va Windows registrini hisobga oladi).
- Broker: `Command::ListVolumes` file threadda bajariladi ([`broker`](../../crates/commandor-broker/src/lib.rs)).
- UI: [`volumes_view.rs`](../../crates/commandor-ui/src/volumes_view.rs) — yo‘l ustida tom chiplari, F1 (A panel) / F2 (B panel) bilan ochiladigan popup: strelka, Enter, Esc, bo‘sh joy ko‘rsatkichi; holat [`state.rs`](../../crates/commandor-ui/src/state.rs)da (`VolumeMenu`).
- Shu sessiyada: `cargo xtask check-boundaries` `commandor-fs → sysinfo` bog‘liqligini rad etayotgan edi. `sysinfo` [arxitekturada](../ARCHITECTURE.uz.md) fayl tizimi qatlami uchun ko‘zda tutilgan, shuning uchun [qoidaga](../../xtask/src/main.rs) qo‘shildi.

## Tekshiruv

macOS arm64 (Darwin 27), Rust 1.94.1 — natijalar [P3-02d](P3-02d-tablar-xatchoplar-buyruq-paneli.md#tekshiruv) bilan bir xil yugurishda:

- `commandor-fs` volumes unit testlari (6 ta, macOS/Linux/Windows qoidalari sintetik mount satrlari bilan) va broker `list_volumes_is_answered_by_file_thread` o‘tdi.
- UI holat testlari: F1/F2 almashinuvi, aylanma strelka, Enter → mount yo‘li, joriy tom belgilanishi, bo‘sh ro‘yxat/noto‘g‘ri indeks.
- Vizual: macOS skrinshotida `Macintosh HD` va `yojik HD` chiplari ko‘rindi. Popup bu sessiyada skrinshot bilan qayta ko‘rilmadi.

## Cheklov va qolgan ish

- Linux va Windows qoidalari faqat sintetik satrlar bilan sinalgan; haqiqiy OSda ishlatilmagan (P1-05 CI).
- Tom ulanishi/uzilishi kuzatilmaydi; ro‘yxat menyu ochilganda qayta so‘raladi (watcher — P3-03).
