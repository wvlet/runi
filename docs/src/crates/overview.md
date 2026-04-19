# Crates Overview

Runi is a Cargo workspace with four crates. They share a single
workspace version (see [Versioning](#versioning) below) and each has
its own entry on [docs.rs](https://docs.rs).

| Crate                                     | Role              | Typical user                  |
| ----------------------------------------- | ----------------- | ----------------------------- |
| [`runi-core`](./runi-core.md)             | Foundation types  | Every other Runi crate        |
| [`runi-log`](./runi-log.md)               | Logging           | Application / service authors |
| [`runi-cli`](./runi-cli.md)               | CLI parser + terminal styling | CLI authors       |
| [`runi-test`](./runi-test.md)             | Test utilities    | Anyone writing tests          |

## How they fit together

- `runi-core` is the only crate the others may depend on. It exposes
  shared `Error`, `Result`, `Config`, and a few string helpers.
- `runi-log` and `runi-cli` are independent leaves — they share
  `nu-ansi-term` for ANSI styling but do not depend on each other.
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
