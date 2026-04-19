# runi-cli Launcher — Phase 1 Plan

Date: 2026-04-19
Status: in progress
Related design: `~/tdx/work/local/default/notes/2026-04-19-design-runi-cli-launcher-zero-dependency-argument-parsing.md`
Related item:   `~/tdx/work/local/default/items/2026-04-19-implement-runi-cli-launcher-phase-1-core-parser-engine.md`

## Goal

Build the zero-dependency command-line argument parsing engine for `runi-cli` with a manually-implemented `Command` trait and builder API. No proc macros in this phase — they arrive in Phase 2.

## Scope

Add `src/launcher/` to `runi-cli` with:

- `types.rs` — `FromArg` trait wrapping `FromStr` with readable errors; impls for `bool`, `String`, `i32/i64/u32/u64`, `f32/f64`, `PathBuf`.
- `schema.rs` — `CLOption`, `CLArgument`, `CommandSchema` builder API. Uni-style comma-separated prefix (`"-v,--verbose"`) parsed internally.
- `parser.rs` — `OptionParser` hand-rolled tokenizer; `ParseResult` with typed extractors (`flag`, `get`, `require`, `all`, `subcommand`). Handles `-v`, `--verbose`, `--key=value`, `--` separator, positional args.
- `help.rs` — `HelpPrinter` using the existing `Tint` API for colored help output.
- `launcher.rs` — `Command` trait, `Launcher::of::<T>()`, `.command::<S>("name")`, `.execute()`, `.parse(args)`. Subcommand `run(&self, global: &G)` dispatch.

## Non-Goals (for Phase 1)

- Derive macro (Phase 2).
- Shell completions (Phase 3).
- `-Dkey=value` key-value options (Phase 3).
- Default subcommand, nested option groups (Phase 3).

## Acceptance

- `cargo build -p runi-cli` succeeds with zero new runtime deps.
- `cargo test -p runi-cli` passes covering: flag options, typed options, required/optional args, repeatable options (`-f a -f b`), `--` separator, subcommand dispatch with global opts.
- `--help` prints Tint-styled help for root + subcommand.
- `Launcher::of::<T>().command::<S>("name").execute()` end-to-end works.

## Open Questions (deferred)

- `--version` flag handling — left for a follow-up.
- `Launcher::execute()` on error — Phase 1 prints the error to stderr and `process::exit(1)`; `parse()` returns `Result` for tests.

## Notes / Learnings (filled during PR cycle)

- _(update after review feedback)_
