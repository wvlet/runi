//! Runi core library.
//!
//! `runi-core` hosts the foundation types shared across the Runi
//! workspace (`Error`, `Result`, `Config`, `str_util`) and also acts as
//! a feature-gated bundle that re-exports the other Runi sub-crates,
//! so most callers only need a single dependency.
//!
//! Each workspace sub-crate (apart from the dev-only `runi-test`) is
//! re-exported as a module named after the crate's suffix and gated by
//! a feature of the same name — e.g. `runi-log` → `runi_core::log`
//! under the `log` feature. The default features enable every bundled
//! sub-crate; see `Cargo.toml` and the [book] for the current list.
//!
//! [book]: https://wvlet.github.io/runi/crates/runi-core.html
//!
//! ## Recommended: alias to `runi`
//!
//! The plain `runi` name on crates.io is held by an unrelated project,
//! so this crate ships as `runi-core`. Cargo's `package` key lets each
//! consumer rename the dependency at the call site, which gives you
//! the clean `runi::` namespace without waiting on a name transfer:
//!
//! ```toml
//! [dependencies]
//! runi = { package = "runi-core", version = "0.1" }                            # everything bundled
//! runi = { package = "runi-core", version = "0.1", default-features = false } # foundation only
//! ```
//!
//! ```ignore
//! use runi::{Error, Result};
//! use runi::log;
//! ```
//!
//! If you prefer, depend on `runi-core` directly and import as
//! `runi_core::…`.
//!
//! The bundle role follows the pattern from
//! [`wvlet/uni`](https://github.com/wvlet/uni).

pub mod config;
pub mod error;
pub mod str_util;

pub use config::Config;
pub use error::{Error, Result};

#[cfg(feature = "log")]
pub use runi_log as log;

#[cfg(feature = "cli")]
pub use runi_cli as cli;
