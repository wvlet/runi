//! End-to-end tests for `#[derive(Command)]`. These live under `tests/`
//! rather than `#[cfg(test)] mod tests` so the derive macro is exercised
//! exactly as downstream users would — through `runi_cli::Command`.
//!
//! Fields read only via the derived `from_parsed` are flagged dead_code
//! by rustc; silence that so the fixtures stay readable.
#![allow(dead_code)]

use runi_cli::{
    CommandSchema, Error, Launcher, OptionParser, ParseResult, Result, Runnable, SubCommandOf,
};
// Derive macro and the trait share the name — macros and traits live in
// separate Rust namespaces so this is fine.
use runi_cli::Command as CommandDerive;

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Struct derive — flag, required option, required positional.
// ---------------------------------------------------------------------------

/// Say hello.
#[derive(CommandDerive, Debug)]
#[command(name = "greet")]
struct Greeter {
    /// Shout instead of whisper.
    #[option("-l,--loud")]
    loud: bool,
    /// Who to greet.
    #[argument]
    target: String,
}

impl Runnable for Greeter {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn struct_derive_basic() {
    let schema = <Greeter as runi_cli::Command>::schema();
    assert_eq!(schema.name, "greet");
    assert_eq!(schema.description, "Say hello.");
    let r = OptionParser::parse(&schema, &args(&["--loud", "world"])).unwrap();
    let g = <Greeter as runi_cli::Command>::from_parsed(&r).unwrap();
    assert!(g.loud);
    assert_eq!(g.target, "world");
}

// ---------------------------------------------------------------------------
// Struct derive — optional and repeatable options.
// ---------------------------------------------------------------------------

#[derive(CommandDerive)]
#[command(name = "build", description = "Build the project")]
struct BuildCmd {
    /// Number of parallel jobs.
    #[option("-j,--jobs")]
    jobs: Option<u32>,
    /// Extra features to enable.
    #[option("-F,--feature")]
    features: Vec<String>,
    /// Target triple.
    #[option("--target")]
    target: String,
}

impl Runnable for BuildCmd {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn struct_derive_optional_and_repeatable() {
    let schema = <BuildCmd as runi_cli::Command>::schema();
    let r = OptionParser::parse(
        &schema,
        &args(&[
            "--jobs",
            "4",
            "-F",
            "alpha",
            "--feature",
            "beta",
            "--target",
            "x86_64-unknown-linux-gnu",
        ]),
    )
    .unwrap();
    let b = <BuildCmd as runi_cli::Command>::from_parsed(&r).unwrap();
    assert_eq!(b.jobs, Some(4));
    assert_eq!(b.features, vec!["alpha".to_string(), "beta".to_string()]);
    assert_eq!(b.target, "x86_64-unknown-linux-gnu");
}

#[test]
fn struct_derive_optional_absent_is_none() {
    let schema = <BuildCmd as runi_cli::Command>::schema();
    let r = OptionParser::parse(&schema, &args(&["--target", "aarch64-apple-darwin"])).unwrap();
    let b = <BuildCmd as runi_cli::Command>::from_parsed(&r).unwrap();
    assert!(b.jobs.is_none());
    assert!(b.features.is_empty());
}

// ---------------------------------------------------------------------------
// Struct derive — optional positional + typed parsing.
// ---------------------------------------------------------------------------

#[derive(CommandDerive)]
#[command(name = "count")]
struct CountCmd {
    /// Starting offset (may be negative).
    #[argument]
    offset: i32,
    /// Output path.
    #[argument]
    out: Option<std::path::PathBuf>,
}

#[test]
fn struct_derive_typed_required_and_optional_positional() {
    let schema = <CountCmd as runi_cli::Command>::schema();
    let r = OptionParser::parse(&schema, &args(&["-5", "/tmp/x"])).unwrap();
    let c = <CountCmd as runi_cli::Command>::from_parsed(&r).unwrap();
    assert_eq!(c.offset, -5);
    assert_eq!(c.out.as_deref(), Some(std::path::Path::new("/tmp/x")));

    let r = OptionParser::parse(&schema, &args(&["42"])).unwrap();
    let c = <CountCmd as runi_cli::Command>::from_parsed(&r).unwrap();
    assert_eq!(c.offset, 42);
    assert!(c.out.is_none());
}

// ---------------------------------------------------------------------------
// Struct derive — inline `description = "..."` on #[option]/#[argument].
// ---------------------------------------------------------------------------

#[derive(CommandDerive)]
#[command(name = "inline")]
struct InlineDesc {
    #[option("-v,--verbose", description = "inline option description")]
    verbose: bool,
    #[argument(description = "inline argument description")]
    path: String,
}

#[test]
fn inline_descriptions_parse() {
    let schema = <InlineDesc as runi_cli::Command>::schema();
    let opt = schema
        .options
        .iter()
        .find(|o| o.matches_long("verbose"))
        .unwrap();
    assert_eq!(opt.description, "inline option description");
    assert_eq!(
        schema.arguments[0].description,
        "inline argument description"
    );
}

// ---------------------------------------------------------------------------
// Struct derive — description precedence: explicit > doc-comment.
// ---------------------------------------------------------------------------

/// Struct-level doc.
#[derive(CommandDerive)]
#[command(name = "described", description = "explicit description wins")]
struct Described {
    /// field doc — should become description.
    #[option("-v,--verbose")]
    verbose: bool,
}

#[test]
fn explicit_description_beats_doc_comment() {
    let schema = <Described as runi_cli::Command>::schema();
    assert_eq!(schema.description, "explicit description wins");
    // The field's doc comment should be used as the option description.
    let opt = schema
        .options
        .iter()
        .find(|o| o.matches_long("verbose"))
        .unwrap();
    assert!(opt.description.contains("field doc"));
}

// ---------------------------------------------------------------------------
// Enum derive — variant → subcommand registration.
// ---------------------------------------------------------------------------

struct GitApp {
    verbose: bool,
}

impl runi_cli::Command for GitApp {
    fn schema() -> CommandSchema {
        CommandSchema::new("git", "VCS").flag("-v,--verbose", "Verbose")
    }
    fn from_parsed(p: &ParseResult) -> Result<Self> {
        Ok(Self {
            verbose: p.flag("--verbose"),
        })
    }
}

#[derive(CommandDerive, Clone)]
#[command(description = "Initialize a repository")]
struct InitCmd {
    /// Where to initialize.
    #[argument]
    dir: Option<String>,
}

impl SubCommandOf<GitApp> for InitCmd {
    fn run(&self, _g: &GitApp) -> Result<()> {
        Ok(())
    }
}

#[derive(CommandDerive, Clone)]
#[command(description = "Clone a repository")]
struct CloneCmd {
    /// Repository URL.
    #[argument]
    url: String,
    /// Clone depth.
    #[option("--depth")]
    depth: Option<u32>,
}

impl SubCommandOf<GitApp> for CloneCmd {
    fn run(&self, _g: &GitApp) -> Result<()> {
        Ok(())
    }
}

#[derive(CommandDerive)]
enum GitSub {
    /// Initialize a repository.
    Init(InitCmd),
    /// Clone a repository.
    Clone(CloneCmd),
}

#[test]
fn enum_derive_register_on_launcher() {
    let launcher = GitSub::register_on(Launcher::<GitApp>::of());
    launcher
        .run_args(&args(&["-v", "clone", "--depth", "1", "https://x"]))
        .unwrap();
}

#[test]
fn enum_variant_doc_becomes_subcommand_description() {
    // Variant `///` on GitSub::Init/Clone should flow through to the
    // composed launcher schema, overriding the inner struct's own
    // description at registration time.
    let launcher = GitSub::register_on(Launcher::<GitApp>::of());
    let schema = launcher.schema();
    let init = schema
        .subcommands
        .iter()
        .find(|s| s.name == "init")
        .unwrap();
    let clone = schema
        .subcommands
        .iter()
        .find(|s| s.name == "clone")
        .unwrap();
    assert_eq!(init.description, "Initialize a repository.");
    assert_eq!(clone.description, "Clone a repository.");
}

#[test]
fn command_with_description_overrides_inner_schema_description() {
    let launcher =
        Launcher::<GitApp>::of().command_with_description::<CloneCmd>("clone", "override!");
    let schema = launcher.schema();
    let clone = schema
        .subcommands
        .iter()
        .find(|s| s.name == "clone")
        .unwrap();
    assert_eq!(clone.description, "override!");
}

#[test]
fn enum_derive_default_name_is_lowercase_variant() {
    // With no explicit #[command(name = ...)], the registered name is the
    // variant ident lower-cased.
    let launcher = GitSub::register_on(Launcher::<GitApp>::of());
    launcher.run_args(&args(&["init"])).unwrap();
}

// ---------------------------------------------------------------------------
// Parity: derive output matches a hand-written Command impl.
// ---------------------------------------------------------------------------

// Hand-written baseline equivalent to Greeter above.
struct GreeterManual {
    loud: bool,
    target: String,
}

impl runi_cli::Command for GreeterManual {
    fn schema() -> CommandSchema {
        CommandSchema::new("greet", "Say hello.")
            .flag("-l,--loud", "Shout instead of whisper.")
            .argument("target", "Who to greet.")
    }
    fn from_parsed(p: &ParseResult) -> Result<Self> {
        Ok(Self {
            loud: p.flag("--loud"),
            target: p.require::<String>("target")?,
        })
    }
}

#[test]
fn derive_and_manual_schemas_match() {
    let derived = <Greeter as runi_cli::Command>::schema();
    let manual = <GreeterManual as runi_cli::Command>::schema();
    assert_eq!(derived.name, manual.name);
    assert_eq!(derived.description, manual.description);
    assert_eq!(derived.options.len(), manual.options.len());
    assert_eq!(derived.arguments.len(), manual.arguments.len());
    for (a, b) in derived.options.iter().zip(manual.options.iter()) {
        assert_eq!(a.short, b.short);
        assert_eq!(a.long, b.long);
        assert_eq!(a.description, b.description);
        assert_eq!(a.takes_value, b.takes_value);
    }
    for (a, b) in derived.arguments.iter().zip(manual.arguments.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.description, b.description);
        assert_eq!(a.required, b.required);
    }
}

// ---------------------------------------------------------------------------
// Error paths still work through derived impls.
// ---------------------------------------------------------------------------

#[test]
fn derived_missing_required_reports_error() {
    let launcher = Launcher::<Greeter>::of();
    let err = launcher.parse(&args(&["--loud"])).unwrap_err();
    assert!(matches!(err, Error::MissingArgument(ref n) if n == "target"));
}

// ---------------------------------------------------------------------------
// Unit struct: schema with no options/arguments.
// ---------------------------------------------------------------------------

/// Just respond that the server is alive.
#[derive(CommandDerive)]
#[command(name = "ping")]
struct Ping;

impl Runnable for Ping {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn unit_struct_derive() {
    let schema = <Ping as runi_cli::Command>::schema();
    assert_eq!(schema.name, "ping");
    assert!(schema.options.is_empty());
    assert!(schema.arguments.is_empty());
    let parsed = OptionParser::parse(&schema, &args(&[])).unwrap();
    let _: Ping = <Ping as runi_cli::Command>::from_parsed(&parsed).unwrap();
}

// Note: we don't add a test for derives on generic enums. The impl
// block threads impl-generics through, but every variant payload must
// implement Command + SubCommandOf<G>, which makes a truly generic-
// parameterized enum awkward to exercise in a unit test without
// additional fixtures. The relevant logic is covered by the non-
// generic enum tests above.
