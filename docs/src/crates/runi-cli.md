# runi-cli

Terminal styling helpers. `runi-cli` provides `Tint`, a small chainable
API for ANSI-colored text, and a terminal-detection helper.

- Crate: [`runi-cli` on crates.io](https://crates.io/crates/runi-cli)
- API reference: [docs.rs/runi-cli](https://docs.rs/runi-cli)

## Example

```rust,ignore
use runi_cli::{Tint, supports_color};

fn main() {
    if supports_color() {
        println!("{}", Tint::red().bold().paint("error: something went wrong"));
        println!("{}", Tint::cyan().italic().paint("hint: try --help"));
    }
}
```

## What's included

- Foreground and background colors (basic + bright)
- ANSI 256-color and 24-bit RGB support
- Chainable style modifiers: `bold`, `italic`, `underline`, `dimmed`,
  `strikethrough`
- A `supports_color()` helper that returns `true` when stderr is a
  terminal

See `runi-cli/examples/tint_demo.rs` in the repo for a full demo.
