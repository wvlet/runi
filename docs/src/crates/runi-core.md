# runi-core

Foundation types shared by the rest of the Runi workspace.

- Crate: [`runi-core` on crates.io](https://crates.io/crates/runi-core)
- API reference: [docs.rs/runi-core](https://docs.rs/runi-core)

## What's in it

- `Error` and `Result` — the workspace-wide error type, built on
  [`thiserror`](https://crates.io/crates/thiserror).
- `Config` — a small configuration helper.
- `str_util` — convenience string helpers.

## Example

```rust,ignore
use runi_core::{Error, Result};

fn load() -> Result<()> {
    // ...
    Ok(())
}
```

Detailed guides for each module are coming soon — for now the API docs
on docs.rs are the source of truth.
