# verso — Design Spec

**Date:** 2026-04-20
**Status:** Approved brainstorm → pending plan
**Target:** v1.0 MVP

## 1. Summary

`verso` is a terminal e-book reader for EPUB files with a Kindle-style library view, vim-style navigation inside books, and first-class Markdown highlight export to Obsidian/Logseq/Zotero. Single static Rust binary. Watched-folder library model. No DRM, no MOBI/AZW3, no PDF in v1 — those are explicit non-goals for the first release.

The wedge, based on prior market research: the "grad student / engineer reading technical books and taking notes in a PKM app" archetype, who is currently served by nobody well. Existing terminal readers (epy, bk, baca, bookokrat) are either stale, EPUB-only without real note workflows, or Kitty-locked PDF viewers. verso fills the EPUB + polished-library + note-export gap.

## 2. Goals

- Open an EPUB and read it comfortably in any modern terminal with vim keys.
- Maintain a library of multiple books with sortable/filterable table view and per-book progress.
- Set bookmarks, highlight passages in visual mode, export all highlights to Markdown files on demand.
- Survive terminal resize without losing reading position (progress stored as character offset, not line number).
- Single binary install via `cargo install verso`.

## 3. Non-goals (v1)

- PDF, MOBI, AZW3, FB2, CBZ, DJVU.
- DRM removal of any kind.
- Cover image rendering (Kitty/Sixel).
- Cross-device sync.
- Library server / OPDS.
- Full-text search across the whole library (only within the open book).
- AI features.
- TTS.

Each of these is feasible later but out of v1 scope to keep time-to-first-release at ~4 weeks.

## 4. User stories

- As a reader, I run `verso` in my terminal and see a table of every book in `~/Books`, sorted by last-read, with progress bars and time-remaining estimates.
- I press `enter` on a book and start reading at the last position I left off, with a 60–72 char centered column and auto-hiding chrome.
- I use vim motions (`j`/`k`, `gg`/`G`, `]]`/`[[`, `/pattern`, `ma`/`'a`) without thinking about it.
- I press `v`, select a passage, press `y`, and it's saved as a highlight.
- I run `verso export dune.epub` (or press `:export`) and get a Markdown file with YAML frontmatter and every highlight, ready for Obsidian.
- I close the terminal, reopen it a week later, and everything is exactly where I left it.

## 5. Architecture

```
┌─────────────────────────────────────────────────┐
│                     main.rs                     │
│   CLI arg parse (clap) · config load · run loop │
└──┬──────────────────────────────────────────────┘
   │
   ├── ui/        Ratatui widgets (library, reader, modals)
   │
   ├── reader/    EPUB parsing (rbook) + rendering + pagination
   │     ├── paginate.rs   Knuth–Plass line breaking, Liang hyphenation
   │     ├── render.rs     Styled-text → terminal spans
   │     └── nav.rs        Vim motion engine (counts, marks, search)
   │
   ├── library/   Watched-folder scanner, metadata extraction
   │
   ├── store/     SQLite (rusqlite) — progress, bookmarks, highlights
   │
   ├── export/    Markdown highlight exporter
   │
   └── config/    TOML loader, keymap override
```

Each module has one job and a narrow public surface. `reader/` does not know about SQLite; `store/` does not know about Ratatui. The run loop in `main.rs` is the only place they meet.

## 6. Stack

| Layer            | Choice                                     | Why                                                            |
| ---------------- | ------------------------------------------ | -------------------------------------------------------------- |
| Language         | Rust (stable)                              | Single-binary distribution, fast startup, strong ecosystem.    |
| TUI framework    | Ratatui 0.27+                              | Richest TUI ecosystem, mature, active.                         |
| EPUB parser      | `rbook`                                    | Modern, ergonomic, actively maintained.                        |
| HTML render      | `scraper` + custom style walker            | Strip to styled spans; we don't need a full browser.           |
| Line breaking    | `textwrap` (with `unicode-linebreak`)      | Handles Knuth–Plass; CJK-aware.                                |
| Hyphenation     | `hyphenation` (Liang/TeX patterns)         | Monospace line quality.                                        |
| Storage          | `rusqlite` + `refinery` migrations         | Zero-ops, one file in XDG data dir.                            |
| CLI parse        | `clap` v4                                  | De-facto standard.                                             |
| Config           | `serde` + `toml`                           | Standard.                                                      |
| Errors           | `anyhow` (app) + `thiserror` (lib surfaces)| Conventional split.                                            |
| Tests            | built-in + `insta` for snapshot tests      | Snapshot-test rendered pages and exported Markdown.            |

## 7. UI design

### 7.1 Library (startup screen) — single dense row

```
┌─ verso ──────────────────────────────── Library · 42 books · 3 reading ──┐
│ Sort: last-read ▾   Filter: all ▾                                         │
│ Title                         Author        Pages  Progress    Left  Last │
├───────────────────────────────────────────────────────────────────────────┤
│ ▸ Dune                        F. Herbert    688    ████░░  12%   9h   2h │
│   SICP                        Abelson       657    ██████  47%  11h   3d │
│   Zero to One ✓               P. Thiel      224    ██████ 100%    —  60d │
│   The Pragmatic Programmer    Hunt & Tho.   320    ██░░░░  23%   4h   5d │
│   Deep Work                   C. Newport    304    ░░░░░░   0%    —   —  │
│   Godel, Escher, Bach         Hofstadter    777    ░░░░░░   8%  38h  14d │
│   Clean Code                  R.C. Martin   464    █████░  62%   3h   1d │
├───────────────────────────────────────────────────────────────────────────┤
│ [d] details  j/k move  enter open  / search  s sort  f filter  a add  q  │
└───────────────────────────────────────────────────────────────────────────┘
```

**Columns:** Title · Author · Pages · Progress bar · % · Time-left estimate · Last-read ago.
**Detail pane** (press `d` on a highlighted row): rating, format, tags, file path, added date, finished date.
**Finished books** show a ✓ after title and have `—` in the Left/Last columns.
**Unstarted books** show `—` in Left/Last.
**Time-left** is derived from a global reading-speed estimate (250 wpm default, refined per-user over time from session data).
**Default sort:** last-read descending. `s` cycles sort: last-read / title / author / progress / added.
**Filter:** `f` cycles: all / reading / unread / finished.

### 7.2 Reader (entered via `enter` on a book) — auto-hide chrome

Full chrome on entry, fades after 3 seconds of idle, any keypress brings it back for 3 more seconds. Minimal chrome visible: just a thin bottom status line showing `Title · %` dimly.

```
┌─ Dune · Frank Herbert ──────────────────────────── Ch.4 · 12% ──┐
│                                                                  │
│           A beginning is the time for taking the most           │
│           delicate care that the balances are correct.          │
│           This every sister of the Bene Gesserit knows.         │
│           [...]                                                 │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│ Ch.4  82/688 pp  12%  ~9h left     NORMAL   :toc :hl ?help  q   │
└──────────────────────────────────────────────────────────────────┘
```

**Column width:** 60–72 chars centered horizontally in the terminal. `z=` cycles narrow (55) / medium (68) / wide (80).
**Themes:** dark (default) / sepia / light. `gt` toggles.
**Rendering:** styled inline HTML (em/strong/code/links) via ANSI. Block elements (headings, blockquotes, lists) get terminal-appropriate treatment — headings bold + bottom-border, blockquotes indented + dim, lists with Unicode bullets, `<code>` blocks with dim background.

### 7.3 Vim keymap

Full keymap lives in `docs/keymap.md` (to be written in v1). Summary:

| Group     | Keys                                                                 |
| --------- | -------------------------------------------------------------------- |
| Movement  | `j k d u f space b gg G H M L { } n N ]] [[`                         |
| Counts    | `5j` · `25%` (jump to percentage) · `23G` (jump to line)             |
| Marks     | `ma` set · `'a` jump · `''` prev-position · `m"` auto-on-quit        |
| Search    | `/foo` forward · `?foo` back · `n/N` repeat                          |
| Highlight | `v` visual · `y` yank-as-highlight · `H` list highlights             |
| Commands  | `:toc` · `:hl` · `:export` · `:w` · `:q` · `:set <opt>`              |
| View      | `z=` width · `gt` theme · `?` help overlay                           |

### 7.4 Modals

- **:toc** — tree of chapters with current highlighted. `enter` jumps, `q` closes.
- **:hl** — list of highlights in this book with surrounding context. `enter` jumps to location, `d` deletes, `e` edits note.
- **?** — floating help overlay listing all keys.

## 8. Data model

### 8.1 Filesystem

```
~/Books/                         # user-configurable watched folder
├── dune.epub
├── sicp.epub
├── pragmatic.epub
└── highlights/                  # export destination
    ├── dune.md
    └── sicp.md

~/.local/share/verso/            # XDG_DATA_HOME
└── verso.db                     # SQLite — progress, bookmarks, highlights, metadata cache

~/.config/verso/                 # XDG_CONFIG_HOME
└── config.toml                  # user config + keymap overrides
```

### 8.2 SQLite schema (v1)

```sql
CREATE TABLE books (
  id           INTEGER PRIMARY KEY,
  file_hash    TEXT NOT NULL UNIQUE,       -- sha256 of file; book identity survives rename/move
  path         TEXT NOT NULL,
  title        TEXT NOT NULL,
  author       TEXT,
  language     TEXT,
  publisher    TEXT,
  published_at TEXT,
  word_count   INTEGER,                     -- for time-left estimate
  page_count   INTEGER,                     -- derived from word_count / 250 wpm typical layout
  added_at     TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at  TEXT,
  rating       INTEGER,                     -- 0–5, nullable
  tags         TEXT                         -- JSON array
);

CREATE TABLE progress (
  book_id      INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
  cfi          TEXT NOT NULL,               -- EPUB CFI-style char offset: "chapter3:1842"
  percent      REAL NOT NULL,
  last_read_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  time_read_s  INTEGER NOT NULL DEFAULT 0   -- cumulative seconds in this book
);

CREATE TABLE bookmarks (
  id          INTEGER PRIMARY KEY,
  book_id     INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  mark        TEXT NOT NULL,                -- 'a', 'b', ..., or '"' for auto
  cfi         TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(book_id, mark)
);

CREATE TABLE highlights (
  id          INTEGER PRIMARY KEY,
  book_id     INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  cfi_start   TEXT NOT NULL,
  cfi_end     TEXT NOT NULL,
  text        TEXT NOT NULL,
  note        TEXT,
  created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Reading-speed calibration (per-user wpm learning) is deferred to v1.1; v1 uses the `config.reader.wpm` value directly.

### 8.3 Progress model

Progress is stored as a **CFI-like character offset** within a chapter (`"chapter3:1842"` = 1842nd character of the 3rd spine item). This survives terminal resize, font changes, and column width changes. Percentage is derived from total word count.

### 8.4 Markdown export format

`~/Books/highlights/<slug>.md`:

```markdown
---
title: Dune
author: Frank Herbert
published: 1965
exported: 2026-04-20T14:32:00Z
progress: 12%
source: /Users/roman/Books/dune.epub
tags: [sci-fi, classics]
---

## Chapter 4

> A beginning is the time for taking the most delicate care that the balances are correct.

**Note:** Irulan's epigraph — thematic through-line about initial conditions.

— p. 82 · 2026-04-18

---

> [...]
```

Obsidian/Logseq-compatible. One file per book. Overwrites on re-export (idempotent).

## 9. Configuration

`~/.config/verso/config.toml`:

```toml
[library]
path = "~/Books"
export_subdir = "highlights"

[reader]
column_width = 68           # 55 / 68 / 80
theme = "dark"              # dark / sepia / light
chrome = "autohide"         # autohide / full / minimal
wpm = 250                   # reading-speed estimate

[keymap]
# Overrides — any vim-style key can be remapped.
# Format: "<action>" = "<key>"
# quit_to_library = "q"
# toggle_theme = "gt"
```

## 10. Error handling

- **Malformed EPUB:** show a toast in the library ("Failed to open dune.epub: missing OPF"), don't crash, log to `~/.local/state/verso/log`.
- **Missing watched folder:** on first launch, prompt to create it (`~/Books`). If user declines, exit with a helpful message.
- **DB migration failure:** refuse to start, show migration error and DB path.
- **Terminal lacks required capabilities:** Ratatui handles the basics. No hard dependencies on Kitty/Sixel in v1 since we don't render images.

Principle: **never silently swallow errors that affect user data** (progress, highlights, exports). Render errors in the open book are recoverable and logged.

## 11. Testing strategy

- **Unit tests** — line breaking, CFI parsing, time-left calculation, config loading.
- **Snapshot tests (`insta`)** — render a canonical chapter of a small test EPUB at 68 cols, dark theme, and compare.
- **Export snapshot tests** — export known highlights from a fixture book and diff the resulting `.md`.
- **Integration test** — spin up a throwaway library dir with 3 fixture EPUBs, exercise library nav + open + read + export end-to-end.
- **Fixture EPUBs** — small CC-licensed books (Standard Ebooks has perfect test candidates).

No tests against real terminal capabilities (too flaky); we render to an in-memory buffer.

## 12. v1 release checklist

- [ ] Open an EPUB, render it readably, navigate with vim keys.
- [ ] Library auto-scans `~/Books`, shows the B2 table, persists progress.
- [ ] Bookmarks (a–z) and auto-`"` on quit.
- [ ] Visual select → highlight → persist.
- [ ] `:export` writes Markdown to `~/Books/highlights/`.
- [ ] Config file loads; at least one key rebinding works.
- [ ] `cargo install verso` produces a working binary on macOS and Linux.
- [ ] README with install, keymap, screenshot.

## 13. Post-v1 roadmap (NOT in scope, for context only)

- v1.1: Cover images (Kitty/Sixel/iTerm2 with chafa fallback), grid view toggle (`gv`).
- v1.2: Calibre library import (read `metadata.db`).
- v1.3: PDF support (MuPDF rasterize + text-extract toggle).
- v2.0: Git-backed sync for progress + highlights across machines.
- v2.1: OPDS catalog browser for finding new books.
- v2.2: MOBI/AZW3 via Calibre `ebook-convert` shell-out at import.

## 14. Open questions

None — all decisions captured above. Brainstorm is complete.

## 15. Decisions log (from brainstorm)

- **Formats v1:** EPUB only. Reason: 4-week ship time vs 12+ for PDF. PDF is the wedge but lives in v1.3.
- **Library layout:** single dense row table (B2). Reason: more books visible per screen; user explicitly preferred.
- **Reader chrome:** auto-hide after 3s idle. Reason: immersive reading, info-on-demand.
- **Library source:** watched folder (`~/Books`). Reason: zero-config, no lock-in, Calibre users accommodated in v1.2.
- **Name:** `verso`. Reason: bookish, short, uncommon, has a natural sibling term `recto` for a future sync server.
- **Progress model:** CFI char offset. Reason: survives terminal resize unlike page numbers.
- **Highlight export:** Markdown with YAML frontmatter, Obsidian/Logseq-compatible. Reason: this is the market differentiator.
- **Sync / covers / PDF:** all deferred to post-v1. Reason: ship in 4 weeks, not 16.
