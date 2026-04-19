//! Runi core library.
//!
//! `runi-core` hosts the foundation types shared across the Runi
//! workspace (`Error`, `Result`, `Config`, `str_util`) and also acts as
//! a feature-gated bundle that re-exports the other Runi sub-crates so
//! most callers only need a single dependency.
//!
//! ## Recommended: alias to `runi`
//!
//! The plain `runi` name on crates.io is held by an unrelated project,
//! so this crate ships as `runi-core`. Cargo lets each consumer rename
//! a dependency at the call site with the `package` key, which gives
//! you the clean `runi::` namespace without waiting on a name
//! transfer:
//!
//! ```toml
//! [dependencies]
//! runi = { package = "runi-core", version = "0.1" }                          # + logging (default)
//! runi = { package = "runi-core", version = "0.1", features = ["cli"] }     # + cli helpers
//! runi = { package = "runi-core", version = "0.1", default-features = false } # foundation only
//! ```
//!
//! ```ignore
//! use runi::{Error, Result};
//! use runi::log;
//! use runi::cli::Tint;
//! ```
//!
//! If you prefer, you can also depend on `runi-core` directly and
//! import as `runi_core::…`.
//!
//! ## Bundle layout
//!
//! - [`runi_log`] is re-exported as [`log`] when the `log` feature is
//!   enabled (on by default).
//! - [`runi_cli`] is re-exported as [`cli`] when the `cli` feature is
//!   enabled.
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
