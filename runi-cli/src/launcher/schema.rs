/// A single named option (`-v,--verbose`, `--count`, etc.).
///
/// Uni's launcher lets callers describe an option with a single comma-separated
/// prefix string; we parse that same syntax here so port-over code reads the
/// same.
#[derive(Clone, Debug)]
pub struct CLOption {
    pub short: Option<String>,
    pub long: Option<String>,
    pub description: String,
    /// `true` when the option consumes the next argument as its value,
    /// `false` for boolean flags.
    pub takes_value: bool,
    /// Placeholder shown in help output (e.g. `<val>`). Ignored for flags.
    pub value_name: String,
}

impl CLOption {
    /// Parse a prefix like `"-v,--verbose"` into short/long tokens.
    ///
    /// Returns a `CLOption` with `takes_value == false` (i.e. a flag).
    /// Use [`CLOption::parse_option`] to build a value-consuming option.
    pub fn parse_flag(prefix: &str, description: impl Into<String>) -> Self {
        let (short, long) = split_prefix(prefix);
        Self {
            short,
            long,
            description: description.into(),
            takes_value: false,
            value_name: String::new(),
        }
    }

    /// Like [`CLOption::parse_flag`] but the option consumes the next argument.
    pub fn parse_option(prefix: &str, description: impl Into<String>) -> Self {
        let (short, long) = split_prefix(prefix);
        Self {
            short,
            long,
            description: description.into(),
            takes_value: true,
            value_name: "val".to_string(),
        }
    }

    /// Canonical lookup key — the long name without dashes if present,
    /// otherwise the short name without dashes. Empty schemas are rejected
    /// at build time so one of the two is always populated.
    pub fn canonical(&self) -> String {
        if let Some(long) = &self.long {
            strip_dashes(long).to_string()
        } else if let Some(short) = &self.short {
            strip_dashes(short).to_string()
        } else {
            String::new()
        }
    }

    pub fn matches_long(&self, name: &str) -> bool {
        self.long
            .as_deref()
            .map(|l| strip_dashes(l) == name)
            .unwrap_or(false)
    }

    pub fn matches_short(&self, name: &str) -> bool {
        self.short
            .as_deref()
            .map(|s| strip_dashes(s) == name)
            .unwrap_or(false)
    }
}

fn split_prefix(prefix: &str) -> (Option<String>, Option<String>) {
    let mut short = None;
    let mut long = None;
    for part in prefix.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if part.starts_with("--") {
            long = Some(part.to_string());
        } else if part.starts_with('-') {
            short = Some(part.to_string());
        }
    }
    (short, long)
}

fn strip_dashes(s: &str) -> &str {
    s.trim_start_matches('-')
}

/// A positional argument (required or optional).
#[derive(Clone, Debug)]
pub struct CLArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

impl CLArgument {
    pub fn new(name: impl Into<String>, description: impl Into<String>, required: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required,
        }
    }
}

/// Schema for a command (root or subcommand). Build with the fluent API
/// or construct the struct literally for tests.
#[derive(Clone, Debug)]
pub struct CommandSchema {
    pub name: String,
    pub description: String,
    pub options: Vec<CLOption>,
    pub arguments: Vec<CLArgument>,
    pub subcommands: Vec<CommandSchema>,
}

impl CommandSchema {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            options: Vec::new(),
            arguments: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    /// Add a boolean flag.
    pub fn flag(mut self, prefix: &str, description: &str) -> Self {
        self.options.push(CLOption::parse_flag(prefix, description));
        self
    }

    /// Add a value-consuming option.
    pub fn option(mut self, prefix: &str, description: &str) -> Self {
        self.options
            .push(CLOption::parse_option(prefix, description));
        self
    }

    /// Add a value-consuming option with a custom placeholder shown in help.
    pub fn option_named(mut self, prefix: &str, value_name: &str, description: &str) -> Self {
        let mut opt = CLOption::parse_option(prefix, description);
        opt.value_name = value_name.to_string();
        self.options.push(opt);
        self
    }

    /// Add a required positional argument.
    pub fn argument(mut self, name: &str, description: &str) -> Self {
        self.arguments
            .push(CLArgument::new(name, description, true));
        self
    }

    /// Add an optional positional argument.
    pub fn optional_argument(mut self, name: &str, description: &str) -> Self {
        self.arguments
            .push(CLArgument::new(name, description, false));
        self
    }

    /// Register a subcommand schema.
    pub fn subcommand(mut self, schema: CommandSchema) -> Self {
        self.subcommands.push(schema);
        self
    }

    pub(crate) fn find_option_long(&self, name: &str) -> Option<&CLOption> {
        self.options.iter().find(|o| o.matches_long(name))
    }

    pub(crate) fn find_option_short(&self, name: &str) -> Option<&CLOption> {
        self.options.iter().find(|o| o.matches_short(name))
    }

    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&CommandSchema> {
        self.subcommands.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;

    #[test]
    fn splits_both_short_and_long() {
        let opt = CLOption::parse_flag("-v,--verbose", "desc");
        assert_eq!(opt.short.as_deref(), Some("-v"));
        assert_eq!(opt.long.as_deref(), Some("--verbose"));
        assert_eq!(opt.canonical(), "verbose");
    }

    #[test]
    fn splits_long_only() {
        let opt = CLOption::parse_option("--count", "desc");
        assert_eq!(opt.short, None);
        assert_eq!(opt.long.as_deref(), Some("--count"));
        assert!(opt.takes_value);
    }

    #[test]
    fn splits_short_only() {
        let opt = CLOption::parse_flag("-n", "desc");
        assert_eq!(opt.short.as_deref(), Some("-n"));
        assert_eq!(opt.long, None);
        assert_eq!(opt.canonical(), "n");
    }

    #[test]
    fn matches_strip_dashes() {
        let opt = CLOption::parse_flag("-v,--verbose", "desc");
        assert!(opt.matches_long("verbose"));
        assert!(opt.matches_short("v"));
        assert!(!opt.matches_long("v"));
    }

    #[test]
    fn builder_collects_options_and_args() {
        let s = CommandSchema::new("app", "desc")
            .flag("-v,--verbose", "verbose")
            .option("-n,--count", "count")
            .argument("file", "input file")
            .optional_argument("out", "output file");
        assert_eq!(s.options.len(), 2);
        assert_eq!(s.arguments.len(), 2);
        assert!(s.arguments[0].required);
        assert!(!s.arguments[1].required);
    }
}
