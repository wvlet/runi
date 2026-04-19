# runi-test

Curated test utilities: one dependency that brings
[`rstest`](https://crates.io/crates/rstest),
[`pretty_assertions`](https://crates.io/crates/pretty_assertions), and
(behind a feature flag) [`proptest`](https://crates.io/crates/proptest).

- Crate: [`runi-test` on crates.io](https://crates.io/crates/runi-test)
- API reference: [docs.rs/runi-test](https://docs.rs/runi-test)

## Install

```toml
[dev-dependencies]
runi-test = "0.1"
# With property-based testing support:
# runi-test = { version = "0.1", features = ["property"] }
```

## Example

```rust,ignore
use runi_test::prelude::*;
use runi_test::pretty_assertions::assert_eq;

#[rstest]
#[case(2, 2, 4)]
#[case(3, 5, 8)]
fn adds(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
    assert_eq!(a + b, expected);
}
```

With the `property` feature enabled you also get `proptest`'s macros and
strategies through `runi_test::prelude`.
