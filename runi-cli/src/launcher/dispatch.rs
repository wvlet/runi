use std::marker::PhantomData;
use std::process;

use super::error::{Error, Result};
use super::help::HelpPrinter;
use super::parser::{OptionParser, ParseResult};
use super::schema::{CLArgument, CommandSchema};

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

    /// Parse `std::env::args()`, run `G::run`, and exit. Parse-origin
    /// failures (including those from `G::from_parsed`, e.g. missing
    /// required args, invalid typed values) route through the help printer
    /// with exit code 2. Runtime failures from `G::run` exit with code 1
    /// and no help banner.
    pub fn execute(self) -> !
    where
        G: Runnable,
    {
        let args = env_args();
        let schema = G::schema();
        let parse_result =
            OptionParser::parse(&schema, &args).and_then(|parsed| G::from_parsed(&parsed));
        let code = match parse_result {
            Ok(g) => match g.run() {
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
        // Silently accepting a duplicate would make later registrations
        // unreachable because parsing stops at the first match. That's a
        // programmer error — fail loudly at startup.
        assert!(
            !self.subs.iter().any(|e| e.schema.name == name),
            "duplicate subcommand name: {name}",
        );
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
        // A subcommand declared directly on G::schema() would not have a
        // runner registered here, so if parsing matched it run_args would
        // report `UnknownSubcommand` at dispatch time. Force users to
        // register subcommands via Launcher::command() where a runner is
        // always attached.
        assert!(
            schema.subcommands.is_empty(),
            "G::schema() must not declare subcommands directly; register them via Launcher::command()",
        );
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
            let composed = compose_help_schema(root, &path);
            let schema = composed.as_ref().unwrap_or(root);
            match *source {
                Error::HelpRequested => {
                    HelpPrinter::print(schema);
                    0
                }
                inner => {
                    eprintln!("error: {inner}");
                    HelpPrinter::print_error(schema);
                    2
                }
            }
        }
        other => {
            eprintln!("error: {other}");
            HelpPrinter::print_error(root);
            2
        }
    }
}

/// Build a help-only schema that represents `root ... path` as a single
/// command. The usage line reads e.g. `git clone [OPTIONS] <url>` or
/// `app <workspace> run [OPTIONS] <target>` — ancestor positionals and
/// subcommand names are folded into the composed schema's `name` so they
/// appear in the order the user must actually type them. Options from the
/// whole chain are merged into a single options list. The returned schema
/// is only suitable for help printing — it is not used for parsing.
fn compose_help_schema(root: &CommandSchema, path: &[String]) -> Option<CommandSchema> {
    let mut options = root.options.clone();
    let mut name_parts = vec![root.name.clone()];
    for arg in &root.arguments {
        name_parts.push(argument_display(arg));
    }

    let mut schema = root;
    for (i, sub_name) in path.iter().enumerate() {
        schema = schema.subcommands.iter().find(|s| s.name == *sub_name)?;
        options.extend(schema.options.iter().cloned());
        name_parts.push(sub_name.clone());
        // Intermediate subcommands' positionals come between this name and
        // the next subcommand. The deepest subcommand's arguments are left
        // in the composed schema's `arguments` so the help printer renders
        // them after `[OPTIONS]`.
        if i + 1 < path.len() {
            for arg in &schema.arguments {
                name_parts.push(argument_display(arg));
            }
        }
    }

    let mut composed = schema.clone();
    composed.name = name_parts.join(" ");
    composed.options = options;
    Some(composed)
}

fn argument_display(arg: &CLArgument) -> String {
    if arg.required {
        format!("<{}>", arg.name)
    } else {
        format!("[{}]", arg.name)
    }
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
    #[should_panic(expected = "duplicate subcommand name: clone")]
    fn duplicate_subcommand_registration_panics() {
        let _ = Launcher::<GitApp>::of()
            .command::<CloneCmd>("clone")
            .command::<CloneCmd>("clone");
    }

    struct AppWithStubSub;
    impl Command for AppWithStubSub {
        fn schema() -> CommandSchema {
            CommandSchema::new("app", "").subcommand(CommandSchema::new("clone", "stub"))
        }
        fn from_parsed(_: &ParseResult) -> Result<Self> {
            Ok(Self)
        }
    }

    #[test]
    #[should_panic(expected = "G::schema() must not declare subcommands")]
    fn schema_declared_subcommands_panic() {
        // combined_schema runs at parse time. Declaring a subcommand
        // directly on G::schema() is unsafe because no runner is
        // registered for it — reject up front.
        let launcher = Launcher::<AppWithStubSub>::of().command::<CloneCmd>("clone");
        let _ = launcher.run_args(&["clone".into()]);
    }

    // Dummy SubCommandOf<AppWithStubSub> impl so the Launcher registration
    // compiles; the panic in combined_schema fires before dispatch.
    impl SubCommandOf<AppWithStubSub> for CloneCmd {
        fn run(&self, _: &AppWithStubSub) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn compose_help_schema_prefixes_root_name_and_options() {
        let root = CommandSchema::new("git", "").flag("-v,--verbose", "Verbose");
        let sub = CommandSchema::new("clone", "Clone a repo").argument("url", "URL");
        let mut with_sub = root.clone();
        with_sub.subcommands.push(sub);
        let composed =
            compose_help_schema(&with_sub, &["clone".to_string()]).expect("must resolve");
        assert_eq!(composed.name, "git clone");
        // Root's --verbose must appear alongside clone's own options.
        assert!(composed.options.iter().any(|o| o.matches_long("verbose")));
        // Clone's own positional must be preserved.
        assert!(composed.arguments.iter().any(|a| a.name == "url"));
    }

    #[test]
    fn compose_help_schema_folds_root_positionals_into_name() {
        // `app <workspace> run <target>` — the root has a positional that
        // must appear before the subcommand name in the usage line.
        let root = CommandSchema::new("app", "").argument("workspace", "");
        let sub = CommandSchema::new("run", "").argument("target", "");
        let mut with_sub = root.clone();
        with_sub.subcommands.push(sub);
        let composed = compose_help_schema(&with_sub, &["run".to_string()]).expect("must resolve");
        assert_eq!(composed.name, "app <workspace> run");
        // The deepest subcommand's own arguments stay in `arguments` so the
        // help printer renders them after `[OPTIONS]` in the usage line.
        assert_eq!(composed.arguments.len(), 1);
        assert_eq!(composed.arguments[0].name, "target");
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
