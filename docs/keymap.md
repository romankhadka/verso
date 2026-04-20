# verso keymap reference

This document lists every action, its default binding, and how to rebind it.

## Action catalog

| Action              | Default binding     | Purpose                                              |
| ------------------- | ------------------- | ---------------------------------------------------- |
| `move_down`         | `j` · `<Down>`      | Scroll down one page                                 |
| `move_up`           | `k` · `<Up>`        | Scroll up one page                                   |
| `page_down`         | `<Space>` · `f` · `<C-f>` | Page down                                      |
| `page_up`           | `b` · `<C-b>`       | Page up                                              |
| `half_page_down`    | `d` · `<C-d>`       | Half page down                                       |
| `half_page_up`      | `u` · `<C-u>`       | Half page up                                         |
| `goto_top`          | `gg`                | Jump to the first page                               |
| `goto_bottom`       | `G`                 | Jump to the last page                                |
| `next_chapter`      | `]]`                | Jump to the next spine item                          |
| `prev_chapter`      | `[[`                | Jump to the previous spine item                      |
| `mark_set`          | `m` + letter        | Set a mark (`ma` sets mark 'a')                      |
| `mark_jump`         | `'` + letter        | Jump to a mark (`'a` jumps to mark 'a')              |
| `search_forward`    | `/`                 | Forward search; type query, `<Enter>` to run         |
| `search_backward`   | `?`                 | Backward search                                      |
| `search_next`       | `n`                 | Next match                                           |
| `search_prev`       | `N`                 | Previous match                                       |
| `visual_select`     | `v`                 | Enter visual mode (highlight a passage)              |
| `yank_highlight`    | `y`                 | In visual mode, save the selection as a highlight    |
| `list_highlights`   | `H`                 | Open the highlights panel                            |
| `cmd`               | `:`                 | Enter command mode (e.g. `:export`, `:toc`)          |
| `quit_to_library`   | `q`                 | Return to the library (from the reader)              |
| `toggle_theme`      | `gt`                | Cycle theme: dark → sepia → light → dark             |
| `cycle_width`       | `z=`                | Cycle column width: 55 / 68 / 80                     |
| `help`              | `<F1>`              | Show the help overlay                                |

## Key-sequence grammar

Bindings are written as:

- a single character (`j`, `?`, `/`)
- a named key in angle brackets (`<Space>`, `<Enter>`, `<Esc>`, `<F1>`, `<Up>`, `<Down>`)
- a Ctrl-chord with a letter (`<C-d>`, `<C-f>`, `<C-b>`)
- a *chord* — two or more keys in sequence with no separator (`gg`, `]]`, `z=`)

## Counts

Where supported, prefix a movement with a decimal integer to repeat: `5j`
scrolls down five pages, `3G` jumps to the 3rd page, `25%` jumps to 25% of
the book. (Counts are partially implemented in v1 — see the roadmap.)

## Command mode

Press `:` to enter command mode. Available commands:

- `:toc` — table of contents
- `:hl` — list highlights for the current book
- `:export` — export highlights to Markdown
- `:w` — write progress now (normally debounced)
- `:q` — quit to library
- `:set <opt> <value>` — change a setting at runtime

## Rebinding

Edit `~/.config/verso/config.toml`:

```toml
[keymap]
# Replace default bindings entirely (each action -> array of sequences).
"move_down"   = ["j", "<Down>", "<Space>"]
"quit_to_library" = "Q"              # single-sequence shortcut
"toggle_theme" = ["gt", "<F5>"]      # add an alternate
```

Rules:

- Unknown action names fail loudly on startup.
- Chord prefix collisions are rejected (e.g. you cannot bind `g` alone while `gg` is also bound).
- Bindings are a complete override, not an addition — if you redefine `move_down`, the default `j`/`<Down>` is replaced. Re-list them in your bindings if you want both.

## Example: Arrow-key first user

```toml
[keymap]
"move_down"   = ["<Down>"]
"move_up"     = ["<Up>"]
"page_down"   = ["<PageDown>", "<Space>"]
"page_up"     = ["<PageUp>", "b"]
```
