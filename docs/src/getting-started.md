# Getting Started

## Install

Most callers only need `runi-core` — it bundles every other workspace
sub-crate behind feature flags, all enabled by default. Cargo's
`package = "..."` alias lets you reach everything through the clean
`runi::` namespace:

```toml
# Cargo.toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }

[dev-dependencies]
runi-test = "0.1"
```

`runi-test` stays out of the bundle because it's a development-only
helper.

### Narrower dependencies

Opt out of the default bundle to get only the foundation types, or a
subset:

```toml
runi = { package = "runi-core", version = "0.1", default-features = false }                     # foundation only
runi = { package = "runi-core", version = "0.1", default-features = false, features = ["log"] } # + one sub-crate
```

You can also depend on any sub-crate directly (`runi-log`, `runi-cli`,
…) — they're independent and live on crates.io as standalone crates.
All `runi-*` crates share a single workspace version, so pin them to
the same version.

## A minimal example

```rust,ignore
use runi::log::{info, warn};

fn main() {
    runi::log::init();

    info!(app = "demo", "starting up");
    warn!("disk space is low");
}
```

Run it with:

```sh
cargo run
# Control the log level with the RUNI_LOG env var:
RUNI_LOG=debug cargo run
```

When stderr is a terminal you get Uni-style colored output; when it is
piped or redirected the same events are emitted as JSON, so log
collectors can parse them directly.

## Next steps

- [Crates Overview](./crates/overview.md) — map of the workspace and
  how the pieces fit together.
- [`runi-core`](./crates/runi-core.md) — canonical list of bundled
  features and the foundation types.
