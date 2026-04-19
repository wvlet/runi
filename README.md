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

Most callers only need `runi-core` — it bundles the other workspace
sub-crates behind feature flags, with every bundled sub-crate enabled
by default. Cargo's `package = "..."` alias lets you reach everything
through the clean `runi::` namespace:

```toml
[dependencies]
runi = { package = "runi-core", version = "0.1" }                             # everything bundled
runi = { package = "runi-core", version = "0.1", default-features = false }   # foundation only
```

```rust
use runi::{Error, Result};
use runi::log;  // any bundled sub-crate is re-exported as a module

fn main() -> Result<()> {
    log::init();
    log::info!("hello from runi");
    Ok(())
}
```

See the [book] for the full feature list and per-sub-crate guides.
Each sub-crate is also published standalone on crates.io if you
prefer narrower dependencies.

[book]: https://wvlet.github.io/runi

## Documentation

- Book: <https://wvlet.github.io/runi>
- API reference per crate: <https://docs.rs/runi-core> (and each sibling crate)

## License

Apache-2.0
