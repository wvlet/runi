# Crates Overview

Runi is a Cargo workspace. Most callers depend on `runi-core` — which
is both the foundation crate and, via feature flags, a bundle that
re-exports the rest of the workspace. The sub-crates can also be used
directly if you prefer narrower dependencies. Crates share a single
workspace version (see [Versioning](#versioning) below) and each has
its own entry on [docs.rs](https://docs.rs).

| Crate                                     | Role              | Typical user                  |
| ----------------------------------------- | ----------------- | ----------------------------- |
| [`runi-core`](./runi-core.md)             | Foundation types + bundle / re-exports | Most callers |
| [`runi-log`](./runi-log.md)               | Logging           | Application / service authors |
| [`runi-cli`](./runi-cli.md)               | CLI parser + terminal styling | CLI authors       |
| [`runi-test`](./runi-test.md)             | Test utilities    | Anyone writing tests          |

Following the [`wvlet/uni`](https://github.com/wvlet/uni) pattern,
`runi-core` plays the dual role of "foundation" and "one-line bundle"
so users can write `runi = { package = "runi-core", ... }` in their
`Cargo.toml` and reach every workspace crate through the clean
`runi::` namespace. See the [runi-core page](./runi-core.md) for the
full alias pattern.

## How they fit together

- `runi-core` re-exports `runi-log` (default) and `runi-cli` (opt-in)
  behind its `log` / `cli` features. Its own `Error`, `Result`,
  `Config`, and `str_util` are always available.
- `runi-log` and `runi-cli` are independent leaves — they share
  `nu-ansi-term` for ANSI styling but do not depend on each other or
  on `runi-core`.
- `runi-test` is a `dev-dependencies`-only helper; it re-exports
  `rstest`, `pretty_assertions`, and (behind the `property` feature)
  `proptest`.

## Versioning

Runi uses **unified workspace versioning** — every crate shares the same
version and they are released as a set. The version is defined once in
the workspace `Cargo.toml` (`[workspace.package]`) and inherited by each
member. Intra-workspace dependencies live in `[workspace.dependencies]`.

All crates currently track `0.1.x`. When the workspace bumps, every
crate bumps together — pick a single version for all four `runi-*`
entries in your `Cargo.toml`. Breaking changes will be called out in
each crate's `CHANGELOG.md` (coming soon).

Independent per-crate versioning can be adopted later if release
cadences diverge; for now the set moves as one.
