# runi-core

`runi-core` is both the foundation layer and the top-level bundle of
the Runi workspace. It hosts the shared `Error`, `Result`, and
`Config` types and, via feature flags, re-exports the rest of the
workspace so most callers only need a single dependency.

- Crate: [`runi-core` on crates.io](https://crates.io/crates/runi-core)
- API reference: [docs.rs/runi-core](https://docs.rs/runi-core)

## Recommended setup — alias to `runi`

The plain `runi` name on crates.io is held by an unrelated project, so
this crate ships as `runi-core`. Cargo lets each consumer rename a
dependency at the call site with the `package` key, which gives you
the clean `runi::` namespace today:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }
```

Then in your code:

```rust,ignore
use runi::{Error, Result};
use runi::log;           // re-exported runi-log
use runi::cli::Tint;     // re-exported runi-cli (behind the `cli` feature)
```

This is the same pattern `async-std`, `http-body-util`, and many other
crates use when their preferred name is unavailable. If you'd rather
skip the alias, depend on `runi-core` directly and import as
`runi_core::…`.

## Features

| Feature | Default | Pulls in    |
| ------- | ------- | ----------- |
| `log`   | yes     | `runi-log`  |
| `cli`   | no      | `runi-cli`  |

Opt out of defaults to pick only the foundation types:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1", default-features = false }
```

## What's in it

**Foundation types** — always available.

- `Error` and `Result` — the workspace-wide error type, built on
  [`thiserror`](https://crates.io/crates/thiserror).
- `Config` — a small configuration helper.
- `str_util` — convenience string helpers.

**Re-exports** — gated by features.

- `runi_core::log` = [`runi-log`](./runi-log.md) (`log` feature, on by default)
- `runi_core::cli` = [`runi-cli`](./runi-cli.md) (`cli` feature)

## Example

```rust,ignore
use runi::{Error, Result};
use runi::log;

fn main() -> Result<()> {
    log::init();
    log::info!("hello from runi");
    Ok(())
}
```

Detailed guides for each module are coming soon — for now the API docs
on docs.rs are the source of truth.
