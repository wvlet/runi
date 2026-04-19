# runi (façade)

`runi` is the top-level façade crate. It re-exports the rest of the
workspace behind feature flags so most callers only need a single
dependency line.

> **Note** — the `runi` crate is currently a workspace-internal
> convenience and is **not published to crates.io** (the name is held
> by an unrelated project). Pull the sub-crates (`runi-core`,
> `runi-log`, `runi-cli`) directly until the name situation is
> resolved.

## What's in it

- Re-exports [`runi-core`](./runi-core.md) with a glob so `Error`,
  `Result`, and `Config` land at the crate root.
- Re-exports [`runi-log`](./runi-log.md) as `runi::log`.
- Re-exports [`runi-cli`](./runi-cli.md) as `runi::cli` (behind the
  `cli` feature).

## Features

| Feature | Default | Pulls in     |
| ------- | ------- | ------------ |
| `core`  | yes     | `runi-core`  |
| `log`   | yes     | `runi-log`   |
| `cli`   | no      | `runi-cli`   |

Opt out of defaults to pick only what you need:

```toml
[dependencies]
runi = { version = "0.1", default-features = false, features = ["core"] }
```

## Example

```rust,ignore
use runi::{Error, Result};       // from runi-core via glob
use runi::log;                   // runi-log

fn main() -> Result<()> {
    log::init();
    log::info!("hello from runi");
    Ok(())
}
```
