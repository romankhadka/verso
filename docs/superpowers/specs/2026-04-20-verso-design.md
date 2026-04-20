# verso — Design Spec

**Date:** 2026-04-20
**Status:** Approved brainstorm → architect-reviewed → pending plan
**Target:** v1.0 MVP

## 1. Summary

`verso` is a terminal e-book reader for EPUB files with a Kindle-style library view, vim-style navigation inside books, and first-class Markdown highlight export to Obsidian/Logseq/Zotero. Single static Rust binary. Watched-folder library model. No DRM, no MOBI/AZW3, no PDF in v1 — those are explicit non-goals for the first release.

The wedge, based on prior market research: the "grad student / engineer reading technical books and taking notes in a PKM app" archetype, who is currently served by nobody well. Existing terminal readers (epy, bk, baca, bookokrat) are either stale, EPUB-only without real note workflows, or Kitty-locked PDF viewers. verso fills the EPUB + polished-library + note-export gap.

## 2. Goals

- Open an EPUB and read it comfortably in any modern terminal with vim keys.
- Maintain a library of multiple books with sortable/filterable table view and per-book progress.
- Set bookmarks, highlight passages in visual mode, export all highlights to Markdown files on demand.
- Survive terminal resize without losing reading position (progress stored as stable character offset, not line number).
- Single binary install via `cargo install verso`.

## 3. Non-goals (v1)

- PDF, MOBI, AZW3, FB2, CBZ, DJVU.
- DRM removal of any kind.
- Cover image rendering (Kitty/Sixel/iTerm2). ASCII-cover extraction revisited in v1.1.
- Cross-device sync.
- Library server / OPDS.
- Full-text search across the whole library (only within the open book).
- AI features, TTS.
- **Windows.** ConPTY and Unicode code-page issues defer to v1.x; contributors welcome, not release-blocking.
- **RTL languages (Arabic, Hebrew)** and full bidi rendering — shown left-to-right in v1 with a warning banner when detected via OPF `dir="rtl"` or Unicode bidi class. CJK text wrapping works via `unicode-linebreak`, but CJK-specific typography (e.g. kinsoku shori) is not implemented.

Each of these is feasible later but out of v1 scope to keep time-to-first-release at ~4 weeks.

## 4. User stories

- As a reader, I run `verso` in my terminal and see a table of every book in `~/Books`, sorted by last-read, with progress bars and time-remaining estimates.
- I press `enter` on a book and start reading at the last position I left off, with a 60–72 char centered column and auto-hiding chrome.
- I use vim motions (`j`/`k`, `gg`/`G`, `]]`/`[[`, `/pattern`, `ma`/`'a`) without thinking about it.
- I press `v`, select a passage, press `y`, and it's saved as a highlight.
- I run `verso export dune.epub` (or press `:export`) and get a Markdown file with YAML frontmatter and every highlight, ready for Obsidian.
- I close the terminal, reopen it a week later, and everything is exactly where I left it.
- I re-import a book that was updated (fixed typo, new cover) and my progress and highlights survive; any highlight that can't be re-anchored exactly is flagged, not silently lost.

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
   │     ├── nav.rs        Vim motion engine (counts, marks, search)
   │     └── anchor.rs     Location model + re-anchoring on re-import
   │
   ├── library/   Watched-folder scanner, fs-watcher, metadata extraction
   │
   ├── store/     SQLite (rusqlite) — progress, bookmarks, highlights, debounced writer
   │
   ├── export/    Markdown highlight exporter
   │
   └── config/    TOML loader, keymap compiler (single keys + chords)
```

Each module has one job and a narrow public surface. `reader/` does not know about SQLite; `store/` does not know about Ratatui. The run loop in `main.rs` is the only place they meet.

### 5.1 Threading model

- **Main thread** owns the Ratatui draw loop and input. Never blocks on I/O.
- **Pagination worker** (one thread, `rayon` or plain `std::thread`) paginates spine items on demand. Already-paginated items cached in-memory (LRU, capped at 32 items). Main thread awaits via `crossbeam::channel`; if a page isn't ready, the reader shows a tiny "…paginating" hint for the current render only.
- **Store writer** is a dedicated single-consumer thread draining a bounded channel of `StoreCommand` values. Progress writes are debounced to every 5 seconds or on blur/quit, whichever first.
- **File watcher** (`notify` crate) is its own thread; events funnel into the main loop via a channel and trigger incremental re-scan.

No shared mutable state across threads — every boundary is a channel.

## 6. Stack

| Layer            | Choice                                     | Why                                                                                |
| ---------------- | ------------------------------------------ | ---------------------------------------------------------------------------------- |
| Language         | Rust (stable)                              | Single-binary distribution, fast startup, strong ecosystem.                        |
| TUI framework    | Ratatui 0.27+ · `crossterm` backend        | Richest TUI ecosystem, mature, active.                                             |
| EPUB parser      | `rbook`                                    | Modern, ergonomic, actively maintained.                                            |
| HTML render      | `scraper` + `ammonia` (sanitizer)          | Strip dangerous tags before render; we don't need a full browser.                  |
| Line breaking    | `textwrap` (with `unicode-linebreak`)      | Handles Knuth–Plass; CJK-aware line-break opportunities.                           |
| Hyphenation      | `hyphenation` (Liang/TeX patterns)         | Monospace line quality.                                                            |
| Bidi detection   | `unicode-bidi`                             | To show the "RTL not supported in v1" banner when appropriate.                     |
| FS watcher       | `notify`                                   | Cross-platform inotify/kqueue/FSEvents.                                            |
| Storage          | `rusqlite` (bundled sqlite) + `refinery`   | Zero-ops, one file in XDG data dir, no system-lib dependency.                      |
| CLI parse        | `clap` v4                                  | De-facto standard.                                                                 |
| Config           | `serde` + `toml`                           | Standard.                                                                          |
| Logging          | `tracing` + `tracing-appender`             | Structured, daily-rotated file logs.                                               |
| Threading        | `crossbeam-channel`                        | MPMC channels.                                                                     |
| Errors           | `anyhow` (app) + `thiserror` (lib surfaces)| Conventional split.                                                                |
| Tests            | built-in + `insta` for snapshot tests      | Snapshot-test rendered pages and exported Markdown.                                |

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
**Detail pane** (press `d` on a highlighted row): rating, format, tags, file path, added date, finished date, parse errors if any.
**Finished books** show a ✓ after title; Left/Last columns show `—`.
**Unstarted books** show `—` in Left/Last.
**Time-left** = remaining words ÷ `config.reader.wpm` (default 250). Per-user calibration arrives in v1.1 from data already captured (see §8.2).
**Default sort:** last-read descending. `s` cycles sort: last-read / title / author / progress / added.
**Filter:** `f` cycles: `all` / `reading` / `unread` / `finished` / `broken` (books that failed to parse — their row shows the error in place of progress).

### 7.2 Reader (entered via `enter` on a book) — auto-hide chrome

Full chrome on entry, fades after 3 seconds of idle. Any keypress brings it back for 3 more seconds. Idle-state chrome is a thin bottom status line: `Title · %` dimly.

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
**Rendering:** styled inline HTML (em/strong/code/links) via ANSI. Block elements: headings bold + bottom-border, blockquotes indented + dim, lists with Unicode bullets, `<code>` inline dim, `<pre>` preserves whitespace.

### 7.3 Vim keymap (summary)

Full action catalog and default bindings live in `docs/keymap.md` (authored in the plan's first milestone). Summary below.

| Group     | Keys                                                                 |
| --------- | -------------------------------------------------------------------- |
| Movement  | `j k d u f space b gg G H M L { } n N ]] [[`                         |
| Counts    | `5j` · `25%` (jump to percentage) · `23G` (jump to line)             |
| Marks     | `ma` set · `'a` jump · `''` prev-position · `m"` auto-on-quit        |
| Search    | `/foo` forward · `?foo` back · `n/N` repeat                          |
| Highlight | `v` visual · `y` yank-as-highlight · `H` list highlights             |
| Commands  | `:toc` · `:hl` · `:export` · `:w` · `:q` · `:set <opt>`              |
| View      | `z=` width · `gt` theme · `?` help overlay · `zl`/`zh` horizontal scroll |

### 7.4 Modals

- **:toc** — tree of chapters with current highlighted. `enter` jumps, `q` closes.
- **:hl** — list of highlights in this book with surrounding context. `enter` jumps to location, `d` deletes, `e` edits note. Highlights that failed to re-anchor after re-import are shown with a warning badge.
- **?** — floating help overlay listing all keys.

### 7.5 Pagination model (load-bearing architectural decision)

Each EPUB spine item (chapter) is paginated **independently** into a virtual page stream at the current column width and font-size settings. This is the unit of rendering.

- `j`/`k` scroll by line **within the current spine item**. At the bottom of the last page of a spine item, pressing `j` loads and jumps to the first page of the next spine item.
- `]]` / `[[` jump to the first page of the next / previous spine item (chapter).
- The chrome shows: `Ch.N (page_in_spine / total_pages_in_spine) · cumulative_percent%`.
- Pagination results are cached per `(spine_idx, column_width, theme)` key. A change to column width invalidates exactly one dimension of the cache, not the whole book.
- `cumulative_percent` is computed from cumulative word-count up to the current character offset, divided by total book word-count. It is stable across column-width changes.

### 7.6 Non-text content rendering

- **Images:** render as `[image: alt-text]` placeholder. No graphical image rendering in v1 (see §3).
- **Tables:** rendered as plain-text with `│`-separated columns. Tables wider than `column_width` truncate the rightmost cells and show a `…(N more cols)` trailer. `zl`/`zh` horizontally pan within the table.
- **Code blocks (`<pre>`):** preserve whitespace. Config key `reader.code_wrap` (default `scroll`): `scroll` keeps lines intact with horizontal-scroll indicators; `wrap` soft-wraps with a continuation marker `↩`.
- **Footnotes:** rendered inline at the end of the spine item they appear in, with internal links back to the call-site.
- **Links:** underlined; `enter` with cursor on a link follows internal links (other spine items) and shows a toast for external URLs (no shell-out in v1).

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

~/.local/state/verso/            # XDG_STATE_HOME
└── log/
    └── verso.log.YYYY-MM-DD     # daily-rotated, 7 files retained

~/.config/verso/                 # XDG_CONFIG_HOME
└── config.toml                  # user config + keymap overrides
```

On startup, scan `~/Books` and diff against `books` table. At runtime, a `notify` watcher triggers incremental re-scan on create/delete/rename (500 ms debounce). First launches with >100 files show a progress indicator while the scan runs on a background thread — the library is fully interactive once the DB is populated; metadata extraction for uncached books continues in the background and rows pop in as they complete.

### 8.2 SQLite schema (v1)

```sql
-- Books: triple-tier identity for re-import robustness.
CREATE TABLE books (
  id             INTEGER PRIMARY KEY,
  stable_id      TEXT,                        -- dc:identifier from OPF (ISBN/UUID); preferred identity
  file_hash      TEXT,                        -- sha256 of file bytes; secondary identity
  title_norm     TEXT NOT NULL,               -- lowercased, stripped title for fallback match
  author_norm    TEXT,                        -- lowercased, stripped author for fallback match
  path           TEXT NOT NULL,
  title          TEXT NOT NULL,
  author         TEXT,
  language       TEXT,
  publisher      TEXT,
  published_at   TEXT,
  word_count     INTEGER,                     -- for time-left estimate
  page_count     INTEGER,                     -- derived estimate
  added_at       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at    TEXT,
  rating         INTEGER,                     -- 0–5, nullable
  parse_error    TEXT,                        -- last parse error, if any (drives "broken" filter)
  deleted_at     TEXT                         -- soft-delete; highlights survive until purge
);
CREATE UNIQUE INDEX idx_books_stable_id ON books(stable_id) WHERE stable_id IS NOT NULL;
CREATE INDEX idx_books_file_hash  ON books(file_hash)  WHERE file_hash IS NOT NULL;
CREATE INDEX idx_books_norm_match ON books(title_norm, author_norm);

-- Tags as a proper join table — enables SQL filtering and counts.
CREATE TABLE tags (
  id   INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);
CREATE TABLE book_tags (
  book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (book_id, tag_id)
);

-- Progress: structured location, not a synthetic "CFI" string.
CREATE TABLE progress (
  book_id       INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
  spine_idx     INTEGER NOT NULL,             -- 0-based index into OPF spine
  char_offset   INTEGER NOT NULL,             -- character offset within spine item's plain text
  anchor_hash   TEXT NOT NULL,                -- hash of ~50 chars around the offset, for re-anchoring
  percent       REAL NOT NULL,                -- derived from cumulative word-count
  last_read_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  time_read_s   INTEGER NOT NULL DEFAULT 0,   -- cumulative seconds in this book
  words_read    INTEGER NOT NULL DEFAULT 0    -- cumulative words read; feeds v1.1 wpm calibration
);

CREATE TABLE bookmarks (
  id           INTEGER PRIMARY KEY,
  book_id      INTEGER NOT NULL REFERENCES books(id) ON DELETE RESTRICT,
  mark         TEXT NOT NULL,                 -- 'a', 'b', ..., or '"' for auto-on-quit
  spine_idx    INTEGER NOT NULL,
  char_offset  INTEGER NOT NULL,
  anchor_hash  TEXT NOT NULL,
  created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (book_id, mark)
);

-- Highlights: anchoring metadata lets us survive re-imports with edits.
CREATE TABLE highlights (
  id              INTEGER PRIMARY KEY,
  book_id         INTEGER NOT NULL REFERENCES books(id) ON DELETE RESTRICT,
  spine_idx       INTEGER NOT NULL,
  chapter_title   TEXT,
  char_offset_start INTEGER NOT NULL,
  char_offset_end   INTEGER NOT NULL,
  text            TEXT NOT NULL,              -- the captured passage
  context_before  TEXT,                       -- ~80 chars before, for re-anchoring
  context_after   TEXT,                       -- ~80 chars after, for re-anchoring
  note            TEXT,
  anchor_status   TEXT NOT NULL DEFAULT 'ok', -- 'ok' | 'drifted' | 'lost'
  created_at      TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at      TEXT
);
CREATE INDEX idx_highlights_book ON highlights(book_id);
```

Bookmarks and highlights use `ON DELETE RESTRICT` so a book's disappearance from disk never silently destroys user-authored data. Removal from the watched folder sets `books.deleted_at` instead; the library hides soft-deleted books by default but a `--purge-orphans` CLI command (interactive confirmation) permanently removes them and their artifacts.

Reading-speed calibration (per-user wpm learning) is deferred to v1.1; v1 uses `config.reader.wpm` directly. `progress.words_read` and `progress.time_read_s` are populated in v1 so v1.1 can compute wpm without a migration.

### 8.3 Location model (replaces ad-hoc "CFI")

Every location inside a book is `(spine_idx, char_offset, anchor_hash)`:

- `spine_idx` — 0-based index of the spine item in the OPF.
- `char_offset` — character offset into the **plain-text extraction** of that spine item (HTML tags stripped, whitespace normalised via a deterministic function documented in `reader/anchor.rs`).
- `anchor_hash` — SHA-256 (truncated to 16 hex chars) of the ~50 characters straddling `char_offset`. Used only for re-anchoring.

This is **not** W3C EPUB CFI. We considered EPUB CFI and rejected it: the spec is complex, overkill for our needs, and no Rust crate implements the full grammar maintainedly. Our location model is simpler, serialises as JSON `{s:3,o:1842,h:"9f8a…"}`, and round-trips deterministically.

**Re-anchoring** (triggered when a book's `file_hash` changes but `stable_id` or normalised title+author matches an existing row): for each location, we recompute plain text for the new spine item, check whether the stored `text` (for highlights) or ±2-char region around `char_offset` (for progress/bookmarks) is still at the same offset. If not, we search the new plain text for `context_before + text + context_after` (highlights) or the anchor hash region (progress/bookmarks). On success, we update offsets and mark `anchor_status='ok'`. On failure for a highlight, `anchor_status='drifted'` (fuzzy match found) or `'lost'` (no match at all); the UI surfaces drifted/lost highlights clearly in `:hl`.

### 8.4 Book identity & re-import

Identity is resolved in this order on scan:

1. **`stable_id`** — `dc:identifier` from the OPF (typically ISBN or a UUID). Authoritative when present.
2. **`file_hash`** — sha256 of the whole file. Used when no stable_id.
3. **Normalised `(title_norm, author_norm)`** — whitespace-collapsed, lowercased, punctuation-stripped. Tertiary fallback for books with no identifier and a changed hash.

When a match is found via #1 or #3 and the `file_hash` differs, we update the row and trigger re-anchoring (§8.3). When no match is found, the book is inserted as new.

### 8.5 SQLite durability

Connection opens with:

```
PRAGMA journal_mode = WAL;
PRAGMA synchronous  = NORMAL;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
```

Progress writes are debounced to every 5 seconds of reading-time, or on any of: `:q`, window blur, SIGTERM, SIGINT, or a `focus_lost` event. On quit, we flush the write queue and explicitly `fsync` before exiting. Highlights and bookmarks write through immediately (not debounced) because their creation is a discrete user action.

Migrations are **forward-only** in v1; opening a newer DB with an older binary refuses to start with a clear message. Users are directed to back up `verso.db` before upgrading. No automatic rollback.

### 8.6 Markdown export format

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

Obsidian/Logseq-compatible. One file per book. Overwrites on re-export (the DB is canonical; the Markdown file is a projection). Drifted highlights are exported with a `*(drifted)*` marker; lost highlights are exported without a location but with their captured text preserved.

## 9. Configuration

`~/.config/verso/config.toml`:

```toml
[library]
path          = "~/Books"
export_subdir = "highlights"
watch         = true          # enable notify-based runtime rescan

[reader]
column_width  = 68            # 55 / 68 / 80
theme         = "dark"        # dark / sepia / light
chrome        = "autohide"    # autohide / full / minimal
chrome_idle_ms = 3000
wpm           = 250
code_wrap     = "scroll"      # scroll / wrap
min_term_cols = 40
min_term_rows = 10

[keymap]
# Each action maps to one or more key sequences.
# A sequence is either a single key ("j"), a named key ("<Down>", "<Enter>", "<Esc>"),
# or a chord ("gg", "]]") of single keys typed in order.
# Multiple bindings per action: give an array.
# Conflict resolution: on load, if two actions share a prefix+full chord
# (e.g. "g" is an action and "gg" is another), verso refuses to start with a
# helpful error naming both bindings.
"move_down"     = ["j", "<Down>"]
"move_up"       = ["k", "<Up>"]
"page_down"     = ["<Space>", "f", "<C-f>"]
"page_up"       = ["b", "<C-b>"]
"half_page_down" = ["d", "<C-d>"]
"half_page_up"   = ["u", "<C-u>"]
"goto_top"      = "gg"
"goto_bottom"   = "G"
"next_chapter"  = "]]"
"prev_chapter"  = "[["
"mark_set"      = "m"         # takes a follow-up letter argument
"mark_jump"     = "'"
"search_forward"  = "/"
"search_backward" = "?"
"search_next"     = "n"
"search_prev"     = "N"
"visual_select"   = "v"
"yank_highlight"  = "y"
"list_highlights" = "H"
"cmd"             = ":"
"quit_to_library" = "q"
"toggle_theme"    = "gt"
"cycle_width"     = "z="
"help"            = "?"
```

The full action catalog (every binding above plus internal actions like `cmd_toc`, `cmd_hl`, `cmd_export`) is specified in `docs/keymap.md`. Unknown actions in `config.toml` fail loudly on startup.

## 10. Error handling, observability, security

### 10.1 User-facing errors

- **Malformed EPUB:** parse error stored on the `books.parse_error` row; the book appears in the library in the `broken` filter, with its row showing the error inline. Opening with `enter` shows the full error and a suggestion.
- **Missing watched folder:** on first launch, offer to create it (`~/Books`). If declined, exit with a helpful message.
- **DB migration failure:** refuse to start; show migration error and DB path.
- **Terminal too small:** if < `min_term_cols` × `min_term_rows`, show a "Terminal too small" banner and pause rendering until resize.
- **Terminal lacks required capabilities:** Ratatui handles the basics. No hard dependencies on Kitty/Sixel in v1 since we don't render images.

Principle: **never silently swallow errors that affect user data** (progress, highlights, exports). Render errors in the open book are recoverable and logged.

### 10.2 Logging

`tracing` + `tracing-appender` with daily rotation, 7 files retained, at `~/.local/state/verso/log/verso.log.YYYY-MM-DD`. Default level `info`; `VERSO_LOG` env var overrides (e.g. `VERSO_LOG=debug`). Human-readable text format; JSON deferred. Progress writes, scans, and parse errors log at `info`; every keypress logs at `trace`.

### 10.3 EPUB security

EPUB is a ZIP archive. Hostile EPUBs are handled as follows:

- **Decompressed-size cap** 256 MB, **per-entry cap** 16 MB, **entry-count cap** 10,000. Exceeding any cap aborts import with a `parse_error`.
- **Internal hrefs** are resolved only inside the archive. Any `..`, absolute path, or `file:`/`http:`/`javascript:` URL in internal navigation is rejected.
- **HTML sanitisation** via `ammonia` before rendering: strip `<script>`, `<iframe>`, `<object>`, `<embed>`, `<link rel>`, and every `on*=` attribute. We never execute JS; there is no JS engine bundled.
- **External images/resources** (as opposed to those inside the EPUB) are never fetched.
- **Symlinks** inside the archive (rare but legal in ZIP) are rejected.

## 11. Testing strategy

- **Unit tests** — anchor hashing, location round-trip, time-left calc, config loading, keymap parser, URL rejection rules.
- **Snapshot tests (`insta`)** — rendered pages and exported Markdown. Policy: one "golden" fixture book rendered across all 3 widths × 3 themes = 9 snapshots; other fixture books tested only at `dark` × `68`.
- **Export snapshot tests** — export known highlights from a fixture book and diff the resulting `.md`.
- **Re-anchoring tests** — take a fixture book, modify it (insert a paragraph, fix a typo, rewrite a chapter), reopen, assert highlights are correctly re-anchored / drifted / lost.
- **Integration test** — throwaway library dir with 3 fixture EPUBs; exercise library nav + open + read + export end-to-end. Includes a step that copies a modified EPUB over an existing one and verifies progress/highlights survive.
- **Hostile-EPUB tests** — zip-bomb, path-traversal href, `<script>`-injected chapter, oversized entry. Each should fail gracefully with a `parse_error`.

**Fixtures** (`tests/fixtures/`): two Standard Ebooks public-domain titles pinned by commit — *The Time Machine* (H. G. Wells) and *Pride and Prejudice* (Jane Austen). Licence note committed alongside.

No tests against real terminal capabilities (too flaky); we render to an in-memory buffer.

## 12. Distribution

- **`cargo install verso`** for source installers.
- **Release artefacts** (GitHub Releases, driven from CI):
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`

Musl is chosen for Linux to produce a single binary that runs across distros without glibc-version mismatches. Homebrew tap considered for v1.1.

## 13. v1 release checklist

- [ ] Open an EPUB, render it readably, navigate with vim keys.
- [ ] Library auto-scans `~/Books`, shows the dense-row table, persists progress.
- [ ] File-watcher picks up new/deleted EPUBs without restart.
- [ ] Bookmarks (a–z) and auto-`"` on quit.
- [ ] Visual select → highlight → persist, including context before/after.
- [ ] `:export` writes Markdown to `~/Books/highlights/`.
- [ ] Re-import a modified EPUB preserves progress; drifted highlights surface in `:hl`.
- [ ] Broken EPUBs appear in the `broken` filter with inline error, never crash the app.
- [ ] Config file loads; keymap overrides work for single keys and chords; conflicting chords fail on startup with a helpful message.
- [ ] SQLite in WAL with FK enforcement; progress writes survive `kill -9` mid-read up to the last 5-second debounce.
- [ ] Release binaries for all four platform targets in §12.
- [ ] README with install, keymap, screenshot. `docs/keymap.md` with full action catalog.

## 14. Post-v1 roadmap (NOT in scope, for context only)

- v1.1: ASCII cover thumbnails (extract EPUB cover, render as block-art). Reading-speed per-user calibration using `progress.words_read` + `time_read_s`.
- v1.2: Cover images (Kitty/Sixel/iTerm2) + grid view (`gv`). Calibre library import (read `metadata.db`).
- v1.3: PDF support (MuPDF rasterise + text-extract toggle).
- v2.0: Git-backed sync for progress + highlights across machines.
- v2.1: OPDS catalogue browser.
- v2.2: MOBI/AZW3 via Calibre `ebook-convert` shell-out at import.

## 15. Open questions

None — all decisions captured above. Implementation plan next.

## 16. Decisions log

- **Formats v1:** EPUB only. 4-week ship time vs 12+ for PDF.
- **Library layout:** single dense row table. User-preferred.
- **Reader chrome:** auto-hide after 3 s idle. Immersive reading, info-on-demand.
- **Library source:** watched folder (`~/Books`). Zero-config; Calibre users accommodated in v1.2.
- **Name:** `verso` (left-hand page of a book).
- **Location model:** `(spine_idx, char_offset, anchor_hash)`, not EPUB CFI. Simpler, sufficient, Rust-ecosystem-friendly.
- **Book identity:** three-tier — `stable_id` (OPF `dc:identifier`) → `file_hash` → normalised title+author.
- **FK cascade:** bookmarks and highlights use `RESTRICT`; books soft-delete. No silent loss of user notes.
- **Highlight anchoring:** `(spine_idx, char_offset_start/end, text, context_before/after)`. Drifted/lost states surfaced in UI.
- **Pagination:** per-spine-item virtual page streams, not whole-book reflow. `j`/`k` within spine, `]]`/`[[` between.
- **SQLite:** WAL + NORMAL synchronous + 5-second debounce + fsync-on-quit for progress; synchronous writes for highlights/bookmarks.
- **Security:** zip-bomb caps, path-traversal rejection, `ammonia` HTML sanitiser, no JS execution, no external fetches.
- **Threading:** main UI thread never blocks; dedicated workers for pagination, store writes, fs-watch.
- **Platforms v1:** macOS (x86_64, arm64) + Linux (musl, x86_64, aarch64). Windows deferred.
- **RTL:** banner + left-to-right rendering in v1; full bidi deferred.
- **Highlight export:** Markdown with YAML frontmatter; DB is canonical, Markdown is a projection.
- **Sync / covers / PDF:** all deferred to post-v1.
