# Contributing to verso

Thanks for considering a contribution. verso is a small project run by one
person (so far) — the bar for getting changes in is "make the gates pass and
keep the scope focused."

## Dev loop

```bash
git clone https://github.com/romankhadka/verso
cd verso
cargo build
cargo test --all       # 60+ tests; should be < 2 s on a modern laptop
```

Before opening a PR, run the same gates CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

If any of these fail, your PR will fail CI. Fix locally first.

## Project layout

```
src/
├── cli.rs                    # clap definitions
├── lib.rs                    # module exports
├── main.rs                   # binary entry
├── config/                   # TOML schema + loader
├── library/                  # EPUB ingestion, scanner, fs-watcher, re-anchor
├── reader/                   # EPUB parsing, pagination, line-breaking, anchors
├── store/                    # SQLite — books, progress, bookmarks, highlights
├── export/                   # Markdown export
├── ui/                       # Ratatui widgets (library, reader, modals)
└── util/                     # XDG paths, logging
migrations/                   # refinery SQL migrations
tests/                        # integration tests; one file per module
.internal/                    # gitignored design specs, plans, follow-ups
```

Each module has one job and a narrow public surface. `reader/` does not
know about SQLite; `store/` does not know about Ratatui. The run loop in
`main.rs` is the only place they meet.

## Tests

- **TDD where it makes sense.** Write the failing test first for any new
  logic.
- Tests live in `tests/<topic>.rs` (one file per logical area).
- Snapshot tests use `insta`. Accept new snapshots with `INSTA_UPDATE=always
  cargo test --test <name>` and commit the `.snap` file.
- The Time Machine fixture (`tests/fixtures/time-machine.epub`) is from
  Standard Ebooks (CC0) — safe to use in any test.

## Commits

- Conventional commit messages: `area: short verb-led summary`.
  Examples already in `git log`:
  - `reader: persist progress on quit + every 5 s; restore on open`
  - `library: broken EPUBs now populate the 'broken' filter`
  - `ci: switch release workflow to taiki-e/upload-rust-binary-action`
- Smaller commits are easier to review than one big one.
- **Do not** include AI co-author trailers (`Co-Authored-By: Claude …`,
  `Co-Authored-By: Cursor …`, etc.) on commits to this repo.

## Pull requests

- Link to the issue if there is one.
- Describe the user-visible behaviour change (or the absence of one for
  internal refactors).
- Add tests proportionate to the change.
- Keep the diff focused. Drive-by formatting / refactoring belongs in its
  own PR.
- The CI matrix runs on ubuntu-latest and macos-latest. Windows is not
  supported in v1; PRs adding Windows support are welcome but should land
  with their own CI job.

## Scope

verso is intentionally small. v1 is **EPUB only**, no DRM, no covers in
the binary form, no sync. The roadmap (in
`.internal/roadmap-and-plan.md`, kept private) sketches what's planned for
v0.2 and beyond. If you want to do something off-roadmap, open an issue
first to discuss.

## Reporting bugs

Useful bug reports include:

- The version (`verso --version`).
- Your OS + terminal.
- A minimal EPUB that triggers the bug if it's an ingestion issue
  (Standard Ebooks titles are fine to share; commercial EPUBs are not).
- The relevant section of `~/.local/state/verso/log/verso.log.<today>`.
- Set `VERSO_LOG=debug` and reproduce for verbose logs.

## Licence

By contributing, you agree your work is licensed under the project's
existing dual licence (MIT OR Apache-2.0).
