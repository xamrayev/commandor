# Commandor

**English** · [Русский](README.ru.md) · [O‘zbekcha](README.uz.md)

**A cross-platform, two-panel (orthodox) file manager written in Rust.**

> **Status: early development (v0.1.0, November 2026).** Only browsing works so far. Copy, move, rename and delete are not implemented yet. Neither are the viewer, editor, Office support, terminal, search index or local AI. The app has only been run on **macOS arm64**. CI for Windows and Linux is written but has not been run yet.

## What works today

- Two independent panels with real directory listings. Each panel has path input, home/up/refresh, back/forward history, a name filter and hidden files.
- **Tabs in each panel.** Each tab has its own location, history, selection and scroll position.
- **Volume list.** Drives are shown as chips above the path. F1 opens the list for the left panel, F2 for the right one.
- **Command palette.** Cmd/Ctrl+Shift+P lists all panel commands and bookmarks, with search.
- **Bookmarks** with ★ or Cmd/Ctrl+D. They are kept for the current session only.
- Sorting by name, size or date; multi-select with selected count and total size.
- Native OS folder picker, and dark and light themes that follow the system.
- Interface languages: Uzbek (primary), Russian, English (Fluent).

### Keyboard

| Key | Action |
|---|---|
| Tab | Switch active panel |
| ↑ ↓ · PageUp PageDown · Home End | Move the cursor |
| Enter · Backspace | Open folder · go up |
| Alt+← / Alt+→ | Back / forward |
| Space / Insert | Select item and move down |
| Cmd/Ctrl+T · Cmd/Ctrl+W · Ctrl+Tab | New tab · close tab · next tab |
| Cmd/Ctrl+Shift+P | Command palette |
| Cmd/Ctrl+D | Bookmark current folder |
| Cmd/Ctrl+A · Cmd/Ctrl+Shift+A | Select all · clear selection |
| Cmd/Ctrl+1 / 2 / 3 | Sort by name / size / date |
| F1 / F2 | Volumes for left / right panel |

## Build and run

You need a stable Rust toolchain (`rustup`). The project declares MSRV 1.92 but has only been tested with 1.94.1.

```sh
cargo run -p commandor-app                     # GUI
cargo run -p commandor-app -- --smoke          # opens, waits for both listings, closes
cargo test --workspace                         # unit and worker-process tests
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask check-boundaries                   # enforces crate dependency direction
```

## Architecture in brief

- **Rust + `egui`/`eframe`.** The app ships as a single native binary.
- **Security boundary.** The `commandor-ui` crate cannot touch the file system, processes or the network. Clippy rules and `#![forbid]` enforce this. The UI talks only to the broker through `AppHandle`, and the broker validates every command.
- **Isolated worker.** Parsers for untrusted content, such as PDF and OOXML, run in a separate `commandor --worker` process. If the worker crashes, the UI keeps running.
- **Crates:** [`crates/`](crates/)

  | Crate | Purpose |
  |---|---|
  | `commandor-core` | Types and protocol |
  | `commandor-broker` | UI facade |
  | `commandor-fs` | Listing and volumes |
  | `commandor-ui` | egui interface |
  | `commandor-app` | Binary |
  | `commandor-ipc` | Worker protocol |
  | `commandor-i18n` | Translations |

  Translations live in [`resources/i18n/`](resources/i18n/), and repository checks in [`xtask/`](xtask/).

### Planned

- File operations with a job queue, conflict UI and trash.
- A text editor (`ropey` + `syntect`).
- Native Markdown preview and PDF via PDFium.
- Office preview and editing through headless LibreOffice. Embedded Office editing is still an open question, so it is not promised.
- Full-text search with SQLite FTS5.
- Optional local AI (Ollama or `llama-server`) that only *proposes* plans; nothing runs until the user confirms it.

## Documentation

The design documents are written in Uzbek.

| Document | Contents |
|---|---|
| [Technical specification](docs/TECHNICAL-SPEC.uz.md) | Requirements and acceptance criteria |
| [Architecture](docs/ARCHITECTURE.uz.md) | Crates, process model, ADRs |
| [Contracts](docs/CONTRACTS.uz.md) | Command/Event types, data model, errors |
| [Delivery plan](docs/DELIVERY.uz.md) | Phases, test matrix, risks |
| [TODO](TODO.md) · [reports](docs/reports/) | Current progress with evidence |

## License

No license has been chosen yet (`UNLICENSED`). All rights reserved.
