//! Procedural macros for `runi-cli`. The runtime crate stays zero-dep;
//! this crate uses `syn` and `quote` at compile time only.
//!
//! Use via `runi_cli::Command` — this crate is re-exported by `runi-cli`
//! so downstream users add one dependency, not two.

use proc_macro::TokenStream;

mod command;

/// Derive `runi_cli::Command` for a struct or enum.
///
/// On a struct, each field becomes an option or positional argument
/// depending on its attribute and type. On an enum, each variant becomes
/// a subcommand (the variant struct is registered on a `Launcher<G>`).
#[proc_macro_derive(Command, attributes(command, option, argument))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    command::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
