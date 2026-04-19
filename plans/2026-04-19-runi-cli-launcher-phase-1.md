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

The PR went through ~11 codex-review rounds. Real design decisions that
emerged during iteration:

- **Parent-first positional binding.** A schema with both positionals and
  subcommands is supported; required positionals bind before any
  subcommand dispatch. For optional positionals, a token that matches a
  registered subcommand name dispatches first — use `--` to force a
  subcommand-named string into the positional slot. Required parent
  positionals are always validated, even when a subcommand dispatched.

- **Dash-prefixed positional values.** `-1`, `-.5`, `-/path` bind as
  positional values (short-option heuristic: only `-<letter>`/`--<word>`
  is an option). Arbitrary dash-prefixed strings also work as option
  values, except that the *known* next option on the same schema or the
  built-in `-h`/`--help` are rejected as probable typos.

- **Error classification.** Parse-origin failures print help
  (`HelpPrinter::print_error` on stderr, exit code 2). Runtime failures
  from `SubCommandOf::run` are wrapped in `Error::Runtime` by the
  registered runner so the launcher can tell them apart, even when user
  code reuses parse-origin variants (`MissingArgument`, `InvalidValue`)
  for its own validation. `--help` output goes to stdout with an
  explicit flush.

- **Subcommand context propagation.** Parser wraps subcommand failures
  in `Error::InSubcommand { path, source }`. The launcher resolves the
  path to compose a help schema that includes the root name, ancestor
  positionals, and merged options — so `git clone --help` prints
  `Usage: git clone [OPTIONS] <url>` rather than root help.

- **Fail loudly on programmer mistakes.** Duplicate subcommand names,
  subcommands declared directly on `G::schema()` (either mode),
  conflicting subcommand names between schema and launcher, and option
  prefixes without any short/long alias all panic at startup. These
  would otherwise produce silently-wrong runtime behavior.

- **Option lookups stay separate from positionals.** `get::<T>("--x")`
  never falls back to the positional `x`. A schema with both
  `argument("config")` and `option("--config")` keeps the two lookups
  independent.

- **Color detection per stream.** `supports_color()` checks stderr;
  new `supports_color_stdout()` checks stdout. `HelpPrinter::print`
  uses the stdout check so `cmd --help > file` produces plain output
  even when stderr is a TTY.
