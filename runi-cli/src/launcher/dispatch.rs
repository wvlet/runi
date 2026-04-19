use std::marker::PhantomData;
use std::process;

use super::error::{Error, Result};
use super::help::HelpPrinter;
use super::parser::{OptionParser, ParseResult};
use super::schema::CommandSchema;

/// A command (root or subcommand) that knows how to produce its argument
/// schema and how to construct itself from a [`ParseResult`].
pub trait Command: Sized {
    fn schema() -> CommandSchema;
    fn from_parsed(parsed: &ParseResult) -> Result<Self>;
}

/// A root command that can be run standalone. The launcher calls `run` after
/// parsing arguments when no subcommands are registered.
pub trait Runnable {
    fn run(&self) -> Result<()>;
}

/// A subcommand invoked in the context of a parent (global-options) struct `G`.
pub trait SubCommandOf<G>: Sized {
    fn run(&self, global: &G) -> Result<()>;
}

type Runner<G> = Box<dyn Fn(&G, &ParseResult) -> Result<()>>;

struct Entry<G> {
    schema: CommandSchema,
    runner: Runner<G>,
}

/// Launcher before any subcommand has been registered.
///
/// Calling [`Launcher::command`] transitions to [`LauncherWithSubs`].
pub struct Launcher<G: Command> {
    _marker: PhantomData<G>,
}

impl<G: Command + 'static> Launcher<G> {
    pub fn of() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Register the first subcommand, moving into subcommand mode.
    pub fn command<S>(self, name: &str) -> LauncherWithSubs<G>
    where
        S: Command + SubCommandOf<G> + 'static,
    {
        LauncherWithSubs::<G>::new().command::<S>(name)
    }

    /// Parse `args` into a `G` without running.
    pub fn parse(&self, args: &[String]) -> Result<G> {
        let schema = G::schema();
        let parsed = OptionParser::parse(&schema, args)?;
        G::from_parsed(&parsed)
    }

    /// Parse `std::env::args()`, run `G::run`, and exit. Prints help on
    /// `--help` and error messages to stderr before exiting.
    pub fn execute(self) -> !
    where
        G: Runnable,
    {
        let args = env_args();
        let schema = G::schema();
        let code = match OptionParser::parse(&schema, &args) {
            Ok(parsed) => match G::from_parsed(&parsed).and_then(|g| g.run()) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("error: {e}");
                    1
                }
            },
            Err(e) => report_error(e, &schema),
        };
        process::exit(code);
    }
}

/// Launcher that already has at least one subcommand registered.
pub struct LauncherWithSubs<G: Command> {
    subs: Vec<Entry<G>>,
}

impl<G: Command + 'static> LauncherWithSubs<G> {
    fn new() -> Self {
        Self { subs: Vec::new() }
    }

    /// Register a subcommand. `S` must implement [`Command`] (for parsing)
    /// and [`SubCommandOf<G>`] (for running with access to the parsed global
    /// options).
    pub fn command<S>(mut self, name: &str) -> Self
    where
        S: Command + SubCommandOf<G> + 'static,
    {
        let mut schema = S::schema();
        schema.name = name.to_string();
        let runner: Runner<G> = Box::new(|global, parsed| {
            let sub = S::from_parsed(parsed)?;
            sub.run(global)
        });
        self.subs.push(Entry { schema, runner });
        self
    }

    fn combined_schema(&self) -> CommandSchema {
        let mut schema = G::schema();
        schema
            .subcommands
            .extend(self.subs.iter().map(|e| e.schema.clone()));
        schema
    }

    /// Parse `args` and run the matched subcommand. Use this in tests to
    /// exercise the launcher without touching the process environment.
    pub fn run_args(&self, args: &[String]) -> Result<()> {
        let schema = self.combined_schema();
        let parsed = OptionParser::parse(&schema, args)?;
        let global = G::from_parsed(&parsed)?;
        let (name, sub_parsed) = parsed
            .subcommand()
            .ok_or_else(|| Error::MissingSubcommand {
                available: self.subs.iter().map(|e| e.schema.name.clone()).collect(),
            })?;
        let entry = self
            .subs
            .iter()
            .find(|e| e.schema.name == name)
            .ok_or_else(|| Error::UnknownSubcommand {
                name: name.to_string(),
                available: self.subs.iter().map(|e| e.schema.name.clone()).collect(),
            })?;
        (entry.runner)(&global, sub_parsed)
    }

    /// Parse `std::env::args()`, dispatch to the matching subcommand, and
    /// exit. Prints help on `--help` and error messages to stderr before
    /// exiting. When a subcommand parse fails, prints help for that
    /// subcommand rather than for the root command.
    pub fn execute(self) -> ! {
        let args = env_args();
        let schema = self.combined_schema();
        let code = match self.run_args(&args) {
            Ok(()) => 0,
            Err(err) => report_error(err, &schema),
        };
        process::exit(code);
    }
}

/// Print a parse error with the most specific help schema available and
/// return the exit code to use. `HelpRequested` is not an error to the user,
/// so it exits 0.
fn report_error(err: Error, root: &CommandSchema) -> i32 {
    match err {
        Error::HelpRequested => {
            HelpPrinter::print(root);
            0
        }
        Error::InSubcommand { path, source } => {
            let schema = resolve_schema(root, &path).unwrap_or(root);
            match *source {
                Error::HelpRequested => {
                    HelpPrinter::print(schema);
                    0
                }
                inner => {
                    eprintln!("error: {inner}");
                    HelpPrinter::print(schema);
                    2
                }
            }
        }
        other => {
            eprintln!("error: {other}");
            HelpPrinter::print(root);
            2
        }
    }
}

fn resolve_schema<'a>(root: &'a CommandSchema, path: &[String]) -> Option<&'a CommandSchema> {
    let mut schema = root;
    for name in path {
        schema = schema.subcommands.iter().find(|s| s.name == *name)?;
    }
    Some(schema)
}

fn env_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;
    use std::cell::RefCell;

    // ----- Root-only command ---------------------------------------------

    struct Greeter {
        loud: bool,
        target: String,
    }

    impl Command for Greeter {
        fn schema() -> CommandSchema {
            CommandSchema::new("greet", "Say hello")
                .flag("-l,--loud", "Shout")
                .argument("target", "Who to greet")
        }

        fn from_parsed(p: &ParseResult) -> Result<Self> {
            Ok(Self {
                loud: p.flag("--loud"),
                target: p.require::<String>("target")?,
            })
        }
    }

    impl Runnable for Greeter {
        fn run(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn root_command_parse() {
        let launcher = Launcher::<Greeter>::of();
        let g = launcher.parse(&["-l".into(), "world".into()]).unwrap();
        assert!(g.loud);
        assert_eq!(g.target, "world");
    }

    // ----- Subcommand mode -----------------------------------------------

    struct GitApp {
        verbose: bool,
    }

    impl Command for GitApp {
        fn schema() -> CommandSchema {
            CommandSchema::new("git", "VCS").flag("-v,--verbose", "Verbose")
        }

        fn from_parsed(p: &ParseResult) -> Result<Self> {
            Ok(Self {
                verbose: p.flag("--verbose"),
            })
        }
    }

    #[derive(Clone)]
    struct CloneCmd {
        url: String,
        depth: Option<u32>,
    }

    impl Command for CloneCmd {
        fn schema() -> CommandSchema {
            CommandSchema::new("clone", "Clone a repo")
                .option("--depth", "Clone depth")
                .argument("url", "Repository URL")
        }

        fn from_parsed(p: &ParseResult) -> Result<Self> {
            Ok(Self {
                url: p.require::<String>("url")?,
                depth: p.get::<u32>("--depth")?,
            })
        }
    }

    thread_local! {
        static CAPTURE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    impl SubCommandOf<GitApp> for CloneCmd {
        fn run(&self, global: &GitApp) -> Result<()> {
            CAPTURE.with(|c| {
                c.borrow_mut().push(format!(
                    "clone verbose={} url={} depth={:?}",
                    global.verbose, self.url, self.depth
                ))
            });
            Ok(())
        }
    }

    #[test]
    fn dispatch_subcommand_with_globals() {
        CAPTURE.with(|c| c.borrow_mut().clear());
        let launcher = Launcher::<GitApp>::of().command::<CloneCmd>("clone");
        launcher
            .run_args(&[
                "-v".into(),
                "clone".into(),
                "--depth".into(),
                "1".into(),
                "https://example.com".into(),
            ])
            .unwrap();
        CAPTURE.with(|c| {
            let captured = c.borrow();
            assert_eq!(captured.len(), 1);
            assert_eq!(
                captured[0],
                "clone verbose=true url=https://example.com depth=Some(1)"
            );
        });
    }

    #[test]
    fn missing_subcommand_error() {
        let launcher = Launcher::<GitApp>::of().command::<CloneCmd>("clone");
        let err = launcher.run_args(&[]).unwrap_err();
        assert!(matches!(err, Error::MissingSubcommand { .. }));
    }

    #[test]
    fn help_requested_error_propagates() {
        let launcher = Launcher::<GitApp>::of().command::<CloneCmd>("clone");
        let err = launcher.run_args(&["--help".into()]).unwrap_err();
        assert!(matches!(err, Error::HelpRequested));
    }

    #[test]
    fn subcommand_rejects_unknown_name() {
        let launcher = Launcher::<GitApp>::of().command::<CloneCmd>("clone");
        let err = launcher.run_args(&["nope".into()]).unwrap_err();
        match err {
            Error::UnknownSubcommand { name, .. } => assert_eq!(name, "nope"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    // Sanity check that the from_parsed path reports invalid types with the
    // argument name, not just the FromStr::Err message.
    #[derive(Debug, Clone)]
    struct NeedsInt {
        n: u32,
    }

    impl Command for NeedsInt {
        fn schema() -> CommandSchema {
            CommandSchema::new("n", "").option("-n,--num", "a number")
        }

        fn from_parsed(p: &ParseResult) -> Result<Self> {
            Ok(Self {
                n: p.require::<u32>("--num")?,
            })
        }
    }
    impl Runnable for NeedsInt {
        fn run(&self) -> Result<()> {
            let _ = self.n;
            Ok(())
        }
    }

    #[test]
    fn invalid_value_error_is_informative() {
        let launcher = Launcher::<NeedsInt>::of();
        let err = launcher.parse(&["--num".into(), "abc".into()]).unwrap_err();
        match err {
            Error::InvalidValue { name, value, .. } => {
                assert_eq!(name, "--num");
                assert_eq!(value, "abc");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
