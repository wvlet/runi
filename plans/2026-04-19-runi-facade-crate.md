# `runi-core` as both foundation and bundle

## Problem

Users of this workspace must currently add individual sub-crates
(`runi-core`, `runi-log`, …) to their `Cargo.toml` and know which
sub-crate owns which API. Most Rust ecosystems (tokio, serde, clap)
provide a single top-level crate that re-exports the common surface
and gates less-common pieces behind feature flags.

A natural answer would be a new top-level `runi` crate — but that
name on crates.io is held by an unrelated project (a unicode font
generator) so we cannot publish under it.

## Design — follow the `wvlet/uni` pattern inside `runi-core`

[`wvlet/uni`](https://github.com/wvlet/uni) solves the same shape by
making one crate carry both "foundation" and "top-level bundle" roles.
We follow that pattern: `runi-core` keeps its existing foundation
types (`Error`, `Result`, `Config`, `str_util`) and additionally
re-exports the rest of the workspace behind feature flags.

### `runi-core/Cargo.toml`

```toml
[dependencies]
thiserror.workspace = true
runi-log = { workspace = true, optional = true }
runi-cli = { workspace = true, optional = true }

[features]
default = ["log"]
log = ["dep:runi-log"]
cli = ["dep:runi-cli"]
```

### `runi-core/src/lib.rs`

```rust
pub mod config;
pub mod error;
pub mod str_util;

pub use config::Config;
pub use error::{Error, Result};

#[cfg(feature = "log")]
pub use runi_log as log;

#[cfg(feature = "cli")]
pub use runi_cli as cli;
```

Rationale for the re-export shapes:

- Foundation types stay at the crate root — the existing
  `runi_core::Error` / `Result` / `Config` imports keep working.
- `runi-log` and `runi-cli` are re-exported *as modules* (`log`,
  `cli`), not globbed, so the crate root stays clean (no stray
  `info!` / `Tint` symbols) and callers write
  `runi_core::log::info!(…)` / `runi_core::cli::Tint`.
- Each bundle re-export is feature-gated so
  `default-features = false` really does compile with only the
  foundation types.

### Clean `runi::` namespace via dependency aliasing

Cargo's `package = "..."` syntax lets consumers rename the dep at the
call site. Callers who want the clean `runi::` namespace write:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }
```

```rust
use runi::{Error, Result};
use runi::log;
use runi::cli::Tint;
```

The published crate name stays `runi-core` (no conflict with the
squatter) — each consumer chooses the local alias they want. This is
the same pattern `async-std`, `http-body-util`, and others use when
their preferred name is taken on crates.io.

### Workspace wiring

- No new crate. `runi-core` is the single entry point.
- `[workspace.package] version` bumps 0.1.0 → 0.1.1 because the
  `runi-core` surface grew (new optional deps + features) even though
  existing `runi_core::Error` / `Result` / `Config` imports are
  unchanged.

## Alternatives considered

- **Separate `runi` façade crate** — blocked by crates.io name
  squatting. Would be the cleanest answer if the name were available.
- **`runi-lib` / `runi-kit` / `runi-all`** — forces callers to type a
  suffix they didn't ask for and still leaves the "core" concept split
  across two crates. `wvlet/uni`'s single-crate-dual-role pattern is
  tidier.
- **Rebrand the project to `uni-*`** — `uni-core` is actively taken on
  crates.io (last update 2025-10-25) so a full `uni-*` prefix scheme
  would collide immediately. Dropped.
- **Email the owner** — still a good follow-up. If the `runi` name
  becomes available later, a thin `runi` façade crate that simply
  does `pub use runi_core::*;` can be added without breaking anyone.

## Acceptance

- `cargo check --workspace --all-features` passes.
- `cargo check -p runi-core --no-default-features` passes (foundation
  only).
- `cargo check -p runi-core --no-default-features --features cli`
  passes.
- `cargo test --workspace` stays green.
- `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- `cargo fmt --all --check` / `taplo fmt --check` clean.
