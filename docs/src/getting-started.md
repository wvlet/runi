# Getting Started

Runi is published as several crates on [crates.io](https://crates.io).
Add only the ones you need.

## Install

```toml
# Cargo.toml
[dependencies]
runi-core = "0.1"
runi-log  = "0.1"
runi-cli  = "0.1"

[dev-dependencies]
runi-test = "0.1"
```

## A minimal example

```rust,ignore
use runi_log::{info, warn};

fn main() {
    runi_log::init();

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

- Read the [Crates Overview](./crates/overview.md) for a map of the
  workspace.
- Jump to a specific crate guide:
  [`runi-core`](./crates/runi-core.md),
  [`runi-log`](./crates/runi-log.md),
  [`runi-cli`](./crates/runi-cli.md),
  [`runi-test`](./crates/runi-test.md).
