# Crates Overview

Runi is a Cargo workspace with four crates. Each one is published
independently and has its own entry on [docs.rs](https://docs.rs).

| Crate                                     | Role              | Typical user                  |
| ----------------------------------------- | ----------------- | ----------------------------- |
| [`runi-core`](./runi-core.md)             | Foundation types  | Every other Runi crate        |
| [`runi-log`](./runi-log.md)               | Logging           | Application / service authors |
| [`runi-cli`](./runi-cli.md)               | Terminal styling  | CLI authors                   |
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

All crates currently track `0.1.x` and are released together. Breaking
changes are called out in each crate's `CHANGELOG.md` (coming soon).
