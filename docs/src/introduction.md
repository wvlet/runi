# Introduction

[![crates.io](https://img.shields.io/crates/v/runi-core.svg?label=runi-core)](https://crates.io/crates/runi-core)
[![docs.rs](https://docs.rs/runi-core/badge.svg)](https://docs.rs/runi-core)
[![CI](https://github.com/wvlet/runi/actions/workflows/test.yml/badge.svg)](https://github.com/wvlet/runi/actions/workflows/test.yml)
[![license](https://img.shields.io/crates/l/runi-core.svg)](https://github.com/wvlet/runi/blob/main/LICENSE)

**Runi** is a collection of small, composable Rust libraries for building
reliable infrastructure and CLI tools. Each crate is scoped to a single
concern and can be used on its own or combined with the rest of the set.

| Crate                           | Purpose                                                   |
| ------------------------------- | --------------------------------------------------------- |
| [`runi-core`](./crates/runi-core.md) | Foundation types + feature-gated bundle that re-exports the rest |
| [`runi-log`](./crates/runi-log.md)   | Structured logging with a Uni-style terminal format  |
| [`runi-cli`](./crates/runi-cli.md)   | Terminal color detection and `Tint` styling helpers  |
| [`runi-test`](./crates/runi-test.md) | Test utilities: `rstest`, `pretty_assertions`, `proptest` |

## Quick install

```toml
# Cargo.toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }

[dev-dependencies]
runi-test = "0.1"
```

```rust,ignore
use runi::{Error, Result};
use runi::log;
```

The `package = "runi-core"` alias lets you write `use runi::…` at the
call site even though the crate ships as `runi-core` on crates.io.
The [`runi-core` page](./crates/runi-core.md) has the full details.

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
