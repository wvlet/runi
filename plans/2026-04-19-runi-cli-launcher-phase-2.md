# runi-cli Launcher — Phase 2 Plan

Date: 2026-04-19
Status: in progress
Builds on: Phase 1 (merged as wvlet/runi#3)
Related design: `~/tdx/work/local/default/notes/2026-04-19-design-runi-cli-launcher-zero-dependency-argument-parsing.md` § Phase 2
Related item:  `~/tdx/work/local/default/items/2026-04-19-implement-runi-cli-launcher-phase-2-derive-macro.md`

## Goal

Add `#[derive(Command)]` proc-macro so users can write attribute-annotated
structs/enums and get a full `Command` trait impl generated for them,
matching Uni's `@command` / `@option` / `@argument` ergonomics.

## Scope

- New workspace crate `runi-cli-macros` (proc-macro) — uses `syn` / `quote`
  (compile-time only).
- `#[derive(Command)]` on structs:
  - `#[command(name = "foo", description = "bar")]` — command metadata.
  - `#[option("-v,--verbose")]` / `#[option("-n,--count")]` — option field.
  - `#[argument]` — positional field.
  - Field type drives runtime behavior: `bool` → flag; `Option<T>` →
    optional; `Vec<T>` → repeatable; bare `T: FromArg` → required.
  - Doc comments (`///`) on fields / struct become descriptions when no
    explicit `description = "..."` is given.
- `#[derive(Command)]` on enums — each variant (with a struct payload)
  becomes a subcommand type. Registration helper so the generated code
  can slot into `Launcher::<G>::of().command::<S>(name)`.
- `runi-cli` re-exports the derive macro so users depend only on one crate.
- Generated code targets the existing Phase 1 trait/builder surface.

## Ground Rules

- `runi-cli` runtime deps stay empty of syn/quote. Macro crate is
  compile-time only.
- Generated `from_parsed()` must call the public `ParseResult` API — no
  poking at internals.
- Prefer the existing builder API for schema construction (readable,
  matches hand-rolled impl).
- Compile-time errors for misconfigurations must surface with clear spans
  (e.g. `#[argument]` on a field typed `bool`, ambiguous attributes).

## Non-Goals

- `-Dkey=value` key-value options (Phase 3).
- Shell completions (Phase 3).
- Default subcommand (Phase 3).
- Nested option groups / `#[flatten]` (Phase 3).

## Open Questions

- `Option<bool>`: should it become a tri-state flag (absent vs explicit
  `--no-…`) or fall back to the usual optional-value semantics? I'll
  treat it as optional value for now and document; negation can land
  in Phase 3.
- Enum-derived subcommands: do they need their own run dispatch wrapper,
  or is the per-variant `SubCommandOf<G>` impl enough? Plan: the
  per-variant `Command` + `SubCommandOf<G>` impls are enough; the enum
  variant is just an ergonomic grouping.

## Acceptance

- `cargo test --workspace` green including new parity tests that compare
  derive-generated `Command` impls against the hand-rolled ones from
  Phase 1.
- `trybuild` compile-fail suite covers the key misconfigurations (wrong
  field type for `#[argument]`, bad option prefix, etc.).
- `runi-cli` runtime Cargo.toml unchanged (no new deps).

## Notes / Learnings (filled during PR cycle)

- _(update after review feedback)_
