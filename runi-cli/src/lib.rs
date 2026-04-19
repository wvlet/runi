pub mod launcher;
pub mod tint;

pub use launcher::{
    CLArgument, CLOption, Command, CommandSchema, Error, FromArg, HelpPrinter, Launcher,
    LauncherWithSubs, OptionParser, ParseResult, Result, Runnable, SubCommandOf,
};
pub use tint::{Tint, supports_color, supports_color_stdout};

/// `#[derive(Command)]` — generate a `Command` trait impl from attributed
/// fields. Requires the default `derive` feature.
#[cfg(feature = "derive")]
pub use runi_cli_macros::Command;
