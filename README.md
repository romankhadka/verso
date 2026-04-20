# verso

A terminal EPUB reader with vim navigation, a Kindle-style library, and first-class Markdown highlight export to Obsidian/Logseq/Zotero.

## Install

```bash
cargo install verso
```

## Quickstart

1. Drop EPUB files into `~/Books/`.
2. Run `verso`.
3. Use `j`/`k` to move through the library; `enter` to open a book; `q` to exit.

## Keys

See `docs/keymap.md` for the full bindings. A summary: `j/k/gg/G/]][[/n/N/v/y/ma/'a/q`.

## Configuration

`~/.config/verso/config.toml`. All keys rebindable (including chords).

## Status

EPUB only in v1. PDF, MOBI, covers, and sync arrive in later releases — see `docs/superpowers/specs/2026-04-20-verso-design.md`.
