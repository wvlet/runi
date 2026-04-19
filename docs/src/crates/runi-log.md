# runi-log

Structured logging with a Uni-style terminal format and a JSON fallback
for non-terminal output. Built on top of
[`tracing`](https://crates.io/crates/tracing).

- Crate: [`runi-log` on crates.io](https://crates.io/crates/runi-log)
- API reference: [docs.rs/runi-log](https://docs.rs/runi-log)

## Quick start

```rust,ignore
use runi_log::{info, warn, error};

fn main() {
    runi_log::init();

    info!(user = "alice", "request received");
    warn!(retries = 3, "slow upstream");
    error!("failed to connect");
}
```

The default format looks like:

```text
2026-04-18T23:12:04 INFO  [my_app] request received - (src/main.rs:8)
```

## Controlling log level

`runi_log::init()` reads the `RUNI_LOG` environment variable (default
`info`), using the standard
[`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
syntax:

```sh
RUNI_LOG=debug cargo run
RUNI_LOG=warn,my_crate=trace cargo run
```

Use `init_with_env("MY_APP_LOG")` to change the env var name, or
`init_with_level("debug")` to hard-code a level.

## Terminal vs JSON

`init()` detects whether stderr is a terminal:

- **Terminal:** colored Uni-style output with file/line locations.
- **Redirected / piped:** JSON with `target`, `file`, and `line_number`
  fields, suitable for log aggregation pipelines.
