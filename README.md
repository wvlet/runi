# Runi

A curated collection of small, composable Rust libraries for building
reliable infrastructure and CLI tools. Each crate is scoped to a
single concern and can be used on its own or combined with the rest
of the set.

## Crates

| Crate        | Role                                              |
| ------------ | ------------------------------------------------- |
| `runi-core`  | Foundation types (`Error`, `Result`, `Config`) + feature-gated bundle that re-exports the rest |
| `runi-log`   | Structured logging with a Uni-style terminal format |
| `runi-cli`   | CLI parser and terminal styling helpers           |
| `runi-test`  | Test utilities (`rstest`, `pretty_assertions`, `proptest`) |

## Quick start

Most callers only need `runi-core` — it re-exports the others behind
feature flags. Cargo's `package = "..."` alias lets you reach
everything through the clean `runi::` namespace:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }                           # + logging (default)
runi = { package = "runi-core", version = "0.1", features = ["cli"] }      # + CLI helpers
runi = { package = "runi-core", version = "0.1", default-features = false } # foundation only
```

```rust
use runi::{Error, Result};
use runi::log;
use runi::cli::Tint;

fn main() -> Result<()> {
    log::init();
    log::info!("hello from runi");
    Ok(())
}
```

Each sub-crate is also published standalone on crates.io if you
prefer narrower dependencies.

## Documentation

- Book: <https://wvlet.github.io/runi>
- API reference: [docs.rs/runi-core](https://docs.rs/runi-core),
  [docs.rs/runi-log](https://docs.rs/runi-log),
  [docs.rs/runi-cli](https://docs.rs/runi-cli),
  [docs.rs/runi-test](https://docs.rs/runi-test)

## License

Apache-2.0
