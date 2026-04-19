# Introduction

**Runi** is a collection of small, composable Rust libraries for building
reliable infrastructure and CLI tools. Each crate is scoped to a single
concern and can be used on its own or combined with the rest of the set.

| Crate                           | Purpose                                                   |
| ------------------------------- | --------------------------------------------------------- |
| [`runi-core`](./crates/runi-core.md) | Foundation types + feature-gated bundle that re-exports the rest |
| [`runi-log`](./crates/runi-log.md)   | Structured logging with a Uni-style terminal format  |
| [`runi-cli`](./crates/runi-cli.md)   | Terminal color detection and `Tint` styling helpers  |
| [`runi-test`](./crates/runi-test.md) | Test utilities: `rstest`, `pretty_assertions`, `proptest` |

Most callers depend on `runi-core` with the `package = "runi-core"`
alias so they can write `use runi::…` at the call site. See the
[runi-core page](./crates/runi-core.md) for the full pattern.

## What this book covers

This book is the user guide for Runi — how to install the crates,
compose them, and use the major features end to end. For the full API
reference, see each crate's page on [docs.rs](https://docs.rs).

## Design principles

- **Small, focused crates.** Pull in only what you need.
- **Rust-native first.** Zero JS toolchain; the docs site itself is
  built with mdBook.
- **Readable output.** Logs and CLI styling default to something a
  human wants to look at, with structured/JSON fallbacks for machines.
- **Test-friendly.** `runi-test` bundles the fixtures and assertion
  helpers we reach for on every project.
