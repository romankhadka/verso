# verso

A terminal EPUB reader with vim navigation, a Kindle-style library, and
first-class Markdown highlight export to Obsidian / Logseq / Zotero.

> **Status:** v1 (EPUB only). PDF, MOBI, covers, and sync arrive in later
> releases — see the Roadmap section below.

---

## Why verso?

Existing terminal e-readers (epy, bk, baca, bookokrat) are either stale,
EPUB-only without real note workflows, or Kitty-locked PDF viewers. verso
fills the gap for the **technical reader who lives in the terminal and takes
notes in a PKM app**: open a book in two keystrokes, move with vim motions,
highlight a passage, get a Markdown file with YAML frontmatter you can drop
straight into your vault.

Single static binary. No daemons, no telemetry, no network. Your library
lives in a folder on disk and a single SQLite file in `~/.local/share/verso/`.

---

## Install

```bash
cargo install verso
```

Pre-built binaries for macOS (Intel + Apple Silicon) and Linux (musl,
x86_64 + aarch64) are attached to GitHub Releases for every `v*` tag.

Requirements: a terminal that handles UTF-8 (any modern terminal — verso does
not require Kitty or Sixel in v1; it intentionally renders text only).

---

## Quickstart

```bash
mkdir -p ~/Books
cp some-book.epub ~/Books/
verso
```

That's the whole onboarding: drop EPUBs into `~/Books/` (or whatever folder
you've configured), run `verso`, and the library appears.

---

## Usage guide

### 1. The library view

Run `verso` with no arguments. You'll land on a dense table of every EPUB
under your watched folder:

```
┌─ verso · Library · 7 books · 3 ─────────────────────────────────────────┐
│ Title                       Author        Pages   Progress  Left  Last  │
├─────────────────────────────────────────────────────────────────────────┤
│ ▸ Dune                      F. Herbert    688     ████░░  12%   9h  2h  │
│   SICP                      Abelson       657     ██████  47%  11h  3d  │
│   Zero to One ✓             P. Thiel      224     ██████ 100%   —  60d  │
│   Pragmatic Programmer      Hunt & Tho.   320     ██░░░░  23%   4h  5d  │
│   Deep Work                 C. Newport    304     ░░░░░░   0%   —   —   │
└─────────────────────────────────────────────────────────────────────────┘
```

**Library keys:**

| Key             | Action                                                 |
| --------------- | ------------------------------------------------------ |
| `j` / `↓`       | Select the next row                                    |
| `k` / `↑`       | Select the previous row                                |
| `enter`         | Open the highlighted book in the reader                |
| `s`             | Cycle sort: last-read → title → author → progress → added |
| `f`             | Cycle filter: all → reading → unread → finished → broken |
| `d`             | Toggle a floating detail pane for the highlighted row  |
| `q`             | Quit verso                                             |

The detail pane (`d`) shows file path, added date, finished date, parse
errors, and per-book counts of bookmarks and highlights. `Esc` or `d` again
closes it.

The library auto-rescans on filesystem changes — drop an EPUB into
`~/Books/` from another terminal and it appears within ~500 ms with no
restart.

### 2. Reading a book

Press `enter` on a row. The reader opens with auto-hiding chrome (a thin
status line at the bottom). Move with familiar vim motions:

| Key                          | Action                                |
| ---------------------------- | ------------------------------------- |
| `j` / `↓`                    | Scroll down one page                  |
| `k` / `↑`                    | Scroll up one page                    |
| `<Space>` / `f` / `<C-f>`    | Page down                             |
| `b` / `<C-b>`                | Page up                               |
| `d` / `<C-d>`                | Half page down                        |
| `u` / `<C-u>`                | Half page up                          |
| `gg`                         | Jump to the first page                |
| `G`                          | Jump to the last page                 |
| `]]` / `[[`                  | Next / previous chapter               |
| `gt`                         | Cycle theme: dark → sepia → light     |
| `q`                          | Quit back to the library              |

The chrome fades after 3 seconds of no input. Any keypress brings it back.

### 3. Bookmarks (`m` / `'`)

verso supports vim-style named marks per book.

- `ma` sets bookmark `a` at the current page. Any letter `a`–`z`.
- `'a` jumps back to bookmark `a`.

Bookmarks persist across sessions in SQLite and are tied to the book by
its stable identifier — they survive renames and even content edits where
re-anchoring succeeds.

### 4. Search (`/foo`, `n`, `N`)

- `/` enters a forward-search prompt; type your query and press `<Enter>`.
- `?` enters a backward-search prompt.
- `<Esc>` cancels the prompt.
- `n` jumps to the next match; `N` to the previous.

Search is case-insensitive across the current book's plain text. The
match cursor wraps at the ends.

### 5. Highlights (`v`, `y`)

Highlights are the differentiator. The flow:

1. Press `v` to enter visual mode (status line shows `[VIS]`).
2. Move with the same motion keys to extend the selection.
3. Press `y` to yank the selection as a highlight.

Each highlight stores the text, ~80 characters of context before and
after, the spine item, and the chapter title. That context is what
allows verso to **re-anchor highlights when the EPUB file changes** —
e.g. you re-import a corrected edition. Drifted highlights surface in
the highlights panel; lost ones keep their text but lose their pinned
location.

`<Esc>` or another `v` exits visual mode without yanking.

### 6. Exporting highlights to Markdown

Two ways:

```bash
verso export ~/Books/dune.epub
# wrote /Users/you/Books/highlights/dune.md
```

The output is Obsidian / Logseq / Zotero-friendly Markdown with YAML
frontmatter:

```markdown
---
title: Dune
author: Frank Herbert
published: 1965
exported: 2026-04-20T14:32:00Z
source: /Users/you/Books/dune.epub
---

## Chapter 4

> A beginning is the time for taking the most delicate care that the
> balances are correct.

**Note:** Irulan's epigraph — thematic through-line about initial conditions.

---
```

Drifted highlights are tagged ` *(drifted)*`; lost ones get ` *(lost)*`.
Re-running `verso export` overwrites the file (the SQLite DB is the
source of truth, the Markdown is a projection).

### 7. Other CLI subcommands

```bash
verso scan            # rescan the library folder explicitly (rare; the app does this on launch)
verso config          # print the effective merged config as TOML
verso purge-orphans   # permanently remove soft-deleted books and their notes (asks first)
verso open <path>     # open a single EPUB without going through the library
verso --help
```

---

## Configuration

verso reads `~/.config/verso/config.toml` if present. All settings have
sensible defaults — the file is purely opt-in.

```toml
[library]
path          = "~/Books"
export_subdir = "highlights"
watch         = true            # rescan on filesystem changes

[reader]
column_width  = 68              # 55 / 68 / 80
theme         = "dark"          # dark / sepia / light
chrome        = "autohide"      # autohide / full / minimal
chrome_idle_ms = 3000
wpm           = 250             # used to estimate "time left" in the library
code_wrap     = "scroll"        # scroll / wrap (for <pre> blocks)
min_term_cols = 40
min_term_rows = 10

[keymap]
# Override any binding. Each action maps to one or more key sequences.
# A sequence is a single key ("j"), a named key ("<Down>", "<Enter>", "<Esc>"),
# a Ctrl-chord ("<C-d>"), or a chord of two single keys ("gg", "]]").
"move_down"       = ["j", "<Down>"]
"quit_to_library" = "Q"
"toggle_theme"    = ["gt", "<F5>"]
```

Conflicting chord prefixes (e.g. binding `g` while `gg` is also bound)
fail at startup with a helpful message. Unknown action names also fail
loudly. See `docs/keymap.md` for the full action catalog.

---

## Where verso keeps its files

| What            | Path                                       |
| --------------- | ------------------------------------------ |
| Library books   | `~/Books/` (or wherever you configure)     |
| Highlight exports | `~/Books/highlights/`                    |
| Database        | `~/.local/share/verso/verso.db`            |
| Config          | `~/.config/verso/config.toml`              |
| Logs (rotated)  | `~/.local/state/verso/log/verso.log.*`     |

(macOS uses `~/Library/Application Support/verso/...` for the data and
state dirs by way of XDG conventions.)

---

## Troubleshooting

**The library is empty even though I have EPUBs in `~/Books/`.**
verso scans on launch, but parse errors are reported (and surfaced under
the `broken` filter — press `f` to cycle to it). Check
`~/.local/state/verso/log/verso.log.YYYY-MM-DD` for details.

**My terminal is too small.**
Below 40 cols × 10 rows, verso pauses rendering with a banner. Resize
the terminal or set lower thresholds in `[reader]`.

**Highlights aren't following my book after I re-imported an edited copy.**
verso re-anchors using ±80 chars of context. If the surrounding
paragraph changed too much, the highlight is marked *drifted* (best-fit
match) or *lost* (text not found). Both are visible in `:hl`. Lost
highlights still keep their captured text — you don't lose the note.

**Set `VERSO_LOG=debug`** to get verbose logs without rebuilding.

---

## Keymap reference

The full action catalog with default bindings, key-sequence grammar, and
rebinding examples lives in [`docs/keymap.md`](docs/keymap.md).

A short summary: `j k gg G ]] [[ / ? n N v y ma 'a gt z= q`.

---

## Roadmap

| Release | Adds                                                                  |
| ------- | --------------------------------------------------------------------- |
| v1.1    | ASCII cover thumbnails. Per-user wpm calibration.                     |
| v1.2    | Cover images (Kitty / Sixel / iTerm2) + grid view. Calibre import.    |
| v1.3    | PDF support (MuPDF rasterise + text-extract toggle).                  |
| v2.0    | Git-backed sync of progress + highlights across machines.             |
| v2.1    | OPDS catalogue browser.                                               |
| v2.2    | MOBI / AZW3 import via Calibre's `ebook-convert`.                     |

DRM removal is and will remain out of scope. Run DeDRM on your own files
before importing.

---

## Contributing

Pull requests welcome. CI runs `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test --all` on
ubuntu and macos. Please keep both gates green and add tests for
behavioural changes.

## Licence

MIT OR Apache-2.0 (your choice).
