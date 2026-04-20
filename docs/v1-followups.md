# v1 follow-ups

Implementation plan executed through all 52 tasks (commits on `verso-v1-impl`). Quality
gates green: 44/44 tests, `cargo fmt --all -- --check` clean, `cargo clippy --all-targets
-- -D warnings` clean, `cargo build --release` succeeds.

The plan intentionally wired components phase-by-phase. A few spec §13 release-checklist
items aren't fully end-to-end yet — documenting them here so v0.1.0 can be tagged after
one more focused pass. None are blockers to opening and reading an EPUB from the library.

## Known gaps from spec §13

1. **Progress write path not wired.** The `progress` table is migrated and `progress.percent`
   feeds the library view's time-left, but the reader never writes to it. No debounced 5-second
   commit, no session tracking of `time_read_s`/`words_read`. Impact: reopening a book always
   starts at page 1. Fix: add a `store::progress` module with `upsert_progress`, debounce from
   the reader's event loop, and call on quit.

2. **Config-file keymap overrides ignored.** `main.rs` constructs the reader from
   `defaults::default_entries()` directly, skipping `cfg.keymap`. Fix: merge `cfg.keymap`
   entries over defaults before handing to `Keymap::from_config`.

3. **`broken` filter never populates.** `scan::scan_folder` reports parse errors in
   `ScanReport.errors` but never sets `books.parse_error`. The `Filter::Broken` row in
   `library_view::list_rows` matches on `parse_error IS NOT NULL` and so shows nothing.
   Fix: upsert a minimal `BookRow` with `parse_error = Some(e)` when metadata extraction
   or guard validation fails.

4. **`:hl` and `:toc` modals unwired.** The keymap defines `cmd` (`:`) and `list_highlights`
   (`H`) actions; the `Dispatch::Fire(Action::ListHighlights)` branch is a no-op. `:export`,
   `:hl`, `:toc` are documented in the keymap but not implemented in the reader. Fix: add
   modal overlay rendering in `reader_app` and wire the actions.

5. **Reanchor on re-import not triggered automatically.** `library::reanchor::reanchor_book`
   exists and is tested, but `scan::scan_folder` doesn't call it when a book's `file_hash`
   changes. Fix: in `scan::scan_folder`, detect hash changes via `resolve_identity`'s return
   value and call `reanchor_book` after the upsert.

6. **aarch64-unknown-linux-musl cross-compile** in `.github/workflows/release.yml` uses
   `musl-tools` which does not ship an aarch64 cross-linker. The job will fail when
   `v*` is tagged. Fix: switch that target to `cross` or `taiki-e/upload-rust-binary-action`.

7. **Auto-`m"` bookmark on quit** mentioned in spec §7.3 isn't implemented. The reader
   does not set a `"` bookmark when quitting. Fix: on `QuitToLibrary`, if `book_id` and
   `db` are present, call `set_bookmark` with `mark = "\""`.

## Non-gap items (intentionally out of v1 scope per §3)

PDF / MOBI / covers / sync / OPDS / AI — all deferred to post-v1.

## Recommended next pass

One focused session to close gaps 1–5 would complete the v1 release-checklist and justify a
`v0.1.0` tag. Gaps 6–7 are small and can piggy-back.
