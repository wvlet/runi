//! Top-level façade crate for the Runi library collection.
//!
//! Depend on `runi` with the features you need instead of pulling in
//! each sub-crate individually:
//!
//! ```toml
//! [dependencies]
//! runi = "0.1"                              # core + log
//! runi = { version = "0.1", features = ["cli"] } # + cli helpers
//! ```
//!
//! - [`runi_core`] is re-exported with a glob so `Error`, `Result`,
//!   and `Config` are available at the crate root.
//! - [`runi_log`] is re-exported as [`log`] (`runi::log::info!`, …).
//! - [`runi_cli`] is re-exported as [`cli`] when the `cli` feature is
//!   enabled.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "core")]
pub use runi_core::*;

#[cfg(feature = "log")]
pub use runi_log as log;

#[cfg(feature = "cli")]
pub use runi_cli as cli;
