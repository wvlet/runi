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
use runi::log;  // any bundled sub-crate is re-exported as a module
```

This is the same pattern `async-std`, `http-body-util`, and many other
crates use when their preferred name is unavailable. If you'd rather
skip the alias, depend on `runi-core` directly and import as
`runi_core::…`.

## Foundation types — always available

- `Error` and `Result` — the workspace-wide error type, built on
  [`thiserror`](https://crates.io/crates/thiserror).
- `Config` — a small configuration helper.
- `str_util` — convenience string helpers.

## Bundled sub-crates

Each workspace sub-crate (apart from the dev-only `runi-test`) is
re-exported as a module gated by a feature flag of the same name —
so `runi-log` becomes `runi_core::log` under the `log` feature. The
default features enable every bundled sub-crate.

This page is the single canonical list; the table gets a new row
whenever a sub-crate is added to the workspace.

| Feature | Default | Module          | Crate                         |
| ------- | ------- | --------------- | ----------------------------- |
| `log`   | yes     | `runi_core::log` | [`runi-log`](./runi-log.md)  |
| `cli`   | yes     | `runi_core::cli` | [`runi-cli`](./runi-cli.md)  |

Opt out of the default bundle to get only the foundation types, or
enable a narrower subset:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1", default-features = false }                     # foundation only
runi = { package = "runi-core", version = "0.1", default-features = false, features = ["log"] } # + one sub-crate
```

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

For detailed usage of each bundled sub-crate, follow its book page
(linked in the table above) or its `docs.rs` entry.
