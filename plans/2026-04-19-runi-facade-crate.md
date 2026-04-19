# Top-level `runi` facade crate

## Problem

Users of this workspace must currently add individual sub-crates
(`runi-core`, `runi-log`, ...) to their `Cargo.toml` and know which
sub-crate owns which API. Most Rust ecosystems (tokio, serde, clap)
provide a single top-level façade crate that re-exports the common
surface and gates less-common pieces behind feature flags.

This plan adds a `runi` façade crate that does the same.

## Design

### Crate layout

```
runi/
  Cargo.toml
  src/lib.rs
```

### `runi/Cargo.toml`

```toml
[package]
name = "runi"
# shared workspace metadata

[dependencies]
runi-core = { workspace = true, optional = true }
runi-log  = { workspace = true, optional = true }
runi-cli  = { workspace = true, optional = true }

[features]
default = ["core", "log"]
core = ["dep:runi-core"]
log  = ["dep:runi-log"]
cli  = ["dep:runi-cli"]
```

### `runi/src/lib.rs`

```rust
#[cfg(feature = "core")]
pub use runi_core::*;

#[cfg(feature = "log")]
pub use runi_log as log;

#[cfg(feature = "cli")]
pub use runi_cli as cli;
```

Rationale:

- `runi-core` is re-exported with a glob so the common `Error`,
  `Result`, and `Config` types land at the crate root — the same
  ergonomics as `runi_core::*`.
- `runi-log` and `runi-cli` are re-exported *as modules* (`runi::log`,
  `runi::cli`) so callers get namespaced access (`runi::log::info!`,
  `runi::cli::Tint`) without polluting the crate root with tracing
  macros or terminal helpers that share short names with the
  standard library.
- All three re-exports are gated by their feature so a user who picks
  `default-features = false` + `features = ["core"]` only pulls in
  `runi-core`.

### Workspace wiring

- Add `"runi"` to `[workspace] members` in the root `Cargo.toml`.
- Add `runi = { path = "runi", version = "0.1.0" }` to
  `[workspace.dependencies]` for symmetry with the other members.

### Publishing

The crate name `runi` on crates.io is **already taken** by an
unrelated project (a unicode font generator owned by `thor314`). We
cannot publish `runi` to crates.io today.

To avoid accidentally shipping a broken `cargo publish` in a future
release, the façade is marked `publish = false` in its `Cargo.toml`
and is **not** added to `.github/workflows/release.yml`. The façade
ships as a workspace-internal convenience for now; resolving the
crates.io name (rename, acquire, or switch to a namespaced name like
`runi-lib`) is a separate follow-up and not in scope for this PR.

### Docs

- Add a `docs/src/crates/runi.md` page describing the façade.
- Link it from `docs/src/SUMMARY.md` and the introduction table.

## Acceptance

- `cargo check --workspace --all-features` passes.
- `cargo check -p runi --no-default-features --features core` passes.
- `cargo test --workspace` stays green.
- `cargo fmt --all --check` is clean.
