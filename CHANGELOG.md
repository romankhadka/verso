# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-04-21

First public release.

### Added

- **Library view** — Kindle-style dense table of every EPUB under
  `~/Books/`, with title, author, page count, progress bar, %, time-left
  estimate, and last-read timestamp.
- **Sort and filter** — `s` cycles last-read / title / author / progress /
  added; `f` cycles all / reading / unread / finished / broken.
- **Detail pane** (`d`) — floating overlay with file path, dates, parse
  errors, and per-book counts of bookmarks and highlights.
- **Cross-chapter reading** — `]]` and `[[` step through spine items;
  `:toc` opens a jump-anywhere modal showing every chapter title (resolved
  via the EPUB's NCX/NAV with a "Chapter N" fallback).
- **Vim navigation** — `j`/`k`/`gg`/`G`/`d`/`u`/`f`/`b`/`<Space>` and
  Ctrl-variants; auto-hide chrome after 3 s of idle input.
- **Bookmarks** — `ma`/`'a` for named marks (any letter `a`–`z`); auto-`"`
  bookmark set on every quit so `''` returns to the last-read position.
- **Search** — case-insensitive `/foo` forward, `?foo` backward, `n`/`N`
  cycling.
- **Highlights** — `v` enters visual mode, `y` yanks the selection as a
  highlight with ±80 chars of context.
- **Re-anchoring** — when an EPUB is re-imported with a different
  `file_hash`, highlights re-anchor automatically using their captured
  context. Drifted highlights surface in `:hl` with a status badge; lost
  ones keep their captured text.
- **Markdown export** — `:export` (in-reader) or `verso export <path>`
  (CLI) writes Obsidian / Logseq / Zotero-compatible Markdown with YAML
  frontmatter to `~/Books/highlights/<slug>.md`.
- **Command prompt** (`:`) — `:toc`, `:hl`, `:export`, `:w`, `:q`. Unknown
  commands surface as a 3 s toast.
- **Highlights modal** (`:hl`) — list of every highlight for the current
  book; `<Enter>` jumps to the chapter and offset, `d` deletes.
- **Progress persistence** — saved on quit and every 5 s; restored to the
  exact chapter and page on reopen; spine index clamped if a re-imported
  EPUB has fewer chapters.
- **Configurable keymap** — `~/.config/verso/config.toml` `[keymap]`
  overrides merge over defaults; chord prefix collisions fail loudly at
  startup.
- **Library auto-rescan** via `notify` watcher with 500 ms debounce —
  drop an EPUB into `~/Books/` from another terminal and it appears.
- **Soft-delete** on file removal; `verso purge-orphans` to clean up.
- **Broken EPUBs** populate the `broken` filter with their parse error
  inline rather than crashing the app.
- **EPUB security** — zip-bomb caps (256 MB total / 16 MB per entry / 10k
  entries), path-traversal rejection, symlink rejection, `ammonia`-based
  HTML sanitisation (no JS execution, no external fetches).
- **WAL SQLite** with `synchronous=NORMAL`, `busy_timeout=5000`, and
  `foreign_keys=ON`. Refinery migrations.
- **CLI:** `verso`, `verso open <path>`, `verso scan`, `verso export
  <path>`, `verso purge-orphans`, `verso config`.
- **Daily-rotated logs** at `~/.local/state/verso/log/verso.log.YYYY-MM-DD`
  (7 files retained). `VERSO_LOG=debug` for verbose.
- **CI on push** — fmt + clippy + tests on ubuntu and macos.
- **Tagged releases** auto-build binaries for `x86_64-apple-darwin`,
  `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and
  `aarch64-unknown-linux-musl` via `taiki-e/upload-rust-binary-action`.

### Notes

- Distributed on crates.io as `verso-reader` (the `verso` package name was
  taken). The installed binary is still called `verso`.
- EPUB only in v1. PDF, MOBI, covers, and sync are on the roadmap.

[Unreleased]: https://github.com/romankhadka/verso/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/romankhadka/verso/releases/tag/v0.1.0
