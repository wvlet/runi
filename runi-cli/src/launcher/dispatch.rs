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
        let name_owned = schema.name.clone();
        let runner: Runner<G> = Box::new(move |global, parsed| {
            // Wrap parse-origin failures (e.g., missing required argument)
            // with subcommand context so the launcher prints help for the
            // right command. The subcommand's own runtime errors pass
            // through unwrapped and are treated as runtime failures.
            let sub = S::from_parsed(parsed).map_err(|e| Error::InSubcommand {
                path: vec![name_owned.clone()],
                source: Box::new(e),
            })?;
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
    /// exit. Prints help on `--help` and parse error messages to stderr
    /// before exiting. When the subcommand's own `run` returns an error,
    /// that is treated as a runtime failure (exit code 1) without printing
    /// help, so legitimate runtime errors aren't reported as bad CLI
    /// syntax.
    pub fn execute(self) -> ! {
        let args = env_args();
        let schema = self.combined_schema();
        let code = match self.run_args(&args) {
            Ok(()) => 0,
            Err(e) if e.is_parse_error() => report_error(e, &schema),
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
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

    // Subcommand whose run() returns a runtime error.
    struct FailingCmd;
    impl Command for FailingCmd {
        fn schema() -> CommandSchema {
            CommandSchema::new("fail", "always fails")
        }
        fn from_parsed(_: &ParseResult) -> Result<Self> {
            Ok(Self)
        }
    }
    impl SubCommandOf<GitApp> for FailingCmd {
        fn run(&self, _: &GitApp) -> Result<()> {
            Err(Error::custom("something went wrong"))
        }
    }

    #[test]
    fn runtime_error_is_not_a_parse_error() {
        let launcher = Launcher::<GitApp>::of().command::<FailingCmd>("fail");
        let err = launcher.run_args(&["fail".into()]).unwrap_err();
        assert!(!err.is_parse_error());
        assert!(matches!(err, Error::Custom(_)));
    }

    // Subcommand that requires an argument — exercises parse-error wrapping
    // from inside the runner (S::from_parsed path).
    #[derive(Debug)]
    struct Needy {
        _name: String,
    }
    impl Command for Needy {
        fn schema() -> CommandSchema {
            CommandSchema::new("needy", "").argument("name", "required")
        }
        fn from_parsed(p: &ParseResult) -> Result<Self> {
            Ok(Self {
                _name: p.require::<String>("name")?,
            })
        }
    }
    impl SubCommandOf<GitApp> for Needy {
        fn run(&self, _: &GitApp) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn subcommand_from_parsed_error_wrapped_with_context() {
        // The parser accepts `needy` with no further args (the positional is
        // declared on the subcommand schema but nothing violates parse shape
        // there), so the MissingArgument surfaces from the runner's call to
        // from_parsed and must be wrapped with the subcommand path for the
        // launcher to pick the right help schema.
        let launcher = Launcher::<GitApp>::of().command::<Needy>("needy");
        let err = launcher.run_args(&["needy".into()]).unwrap_err();
        match err {
            Error::InSubcommand { path, source } => {
                assert_eq!(path, vec!["needy".to_string()]);
                assert!(matches!(*source, Error::MissingArgument(_)));
            }
            other => panic!("expected InSubcommand, got {other:?}"),
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
