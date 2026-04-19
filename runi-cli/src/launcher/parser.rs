use std::collections::{HashMap, HashSet};

use super::error::{Error, Result};
use super::schema::{CLOption, CommandSchema};
use super::types::FromArg;

/// Outcome of parsing a raw argv slice against a [`CommandSchema`].
///
/// Holds typed-but-unconverted values; callers pull values out via
/// [`ParseResult::flag`], [`ParseResult::get`], etc. Conversion happens
/// lazily at extraction time so a single result can be probed with
/// different target types during tests without repeated parsing.
#[derive(Debug, Default)]
pub struct ParseResult {
    /// Option values keyed by canonical name. Multi-use options append.
    values: HashMap<String, Vec<String>>,
    /// Flags that appeared on the command line, keyed by canonical name.
    flags: HashSet<String>,
    /// Positional arguments keyed by name.
    args: HashMap<String, String>,
    /// Short → canonical lookup, so callers can ask for `-v` or `--verbose`
    /// interchangeably.
    short_to_canonical: HashMap<String, String>,
    /// Matched subcommand, if any.
    subcommand: Option<(String, Box<ParseResult>)>,
}

impl ParseResult {
    /// Look up the canonical key for any user-supplied option token.
    fn canonical_key(&self, name: &str) -> String {
        let stripped = name.trim_start_matches('-');
        self.short_to_canonical
            .get(stripped)
            .cloned()
            .unwrap_or_else(|| stripped.to_string())
    }

    /// Check whether a boolean flag was provided.
    pub fn flag(&self, name: &str) -> bool {
        let key = self.canonical_key(name);
        self.flags.contains(&key)
    }

    /// Get the last value supplied for an option, converted via [`FromArg`].
    /// Returns `Ok(None)` when the option is absent.
    ///
    /// Looks up positional arguments as a fallback so callers don't need two
    /// code paths for "option or argument by name".
    pub fn get<T: FromArg>(&self, name: &str) -> Result<Option<T>> {
        let key = self.canonical_key(name);
        if let Some(values) = self.values.get(&key) {
            if let Some(last) = values.last() {
                return T::from_arg(last)
                    .map(Some)
                    .map_err(|m| Error::invalid_value(name, last, m));
            }
        }
        if let Some(raw) = self.args.get(&key) {
            return T::from_arg(raw)
                .map(Some)
                .map_err(|m| Error::invalid_value(name, raw, m));
        }
        Ok(None)
    }

    /// Like [`ParseResult::get`] but errors if the value is missing.
    pub fn require<T: FromArg>(&self, name: &str) -> Result<T> {
        self.get::<T>(name)?
            .ok_or_else(|| Error::MissingArgument(name.to_string()))
    }

    /// Get all values supplied for a repeatable option.
    pub fn all<T: FromArg>(&self, name: &str) -> Result<Vec<T>> {
        let key = self.canonical_key(name);
        let Some(values) = self.values.get(&key) else {
            return Ok(Vec::new());
        };
        values
            .iter()
            .map(|v| T::from_arg(v).map_err(|m| Error::invalid_value(name, v, m)))
            .collect()
    }

    /// Return the matched subcommand name and its parse result, if any.
    pub fn subcommand(&self) -> Option<(&str, &ParseResult)> {
        self.subcommand
            .as_ref()
            .map(|(n, r)| (n.as_str(), r.as_ref()))
    }

    /// Raw access for advanced callers.
    pub fn raw_value(&self, name: &str) -> Option<&str> {
        let key = self.canonical_key(name);
        self.values
            .get(&key)
            .and_then(|v| v.last())
            .map(String::as_str)
            .or_else(|| self.args.get(&key).map(String::as_str))
    }
}

/// Hand-rolled tokenizer. Translates a flat argv slice into a [`ParseResult`]
/// guided by the schema.
pub struct OptionParser;

impl OptionParser {
    /// Parse `args` against `schema`, producing a [`ParseResult`] or an error.
    pub fn parse(schema: &CommandSchema, args: &[String]) -> Result<ParseResult> {
        let mut result = ParseResult::default();
        populate_short_map(&mut result.short_to_canonical, schema);

        let mut i = 0;
        let mut positional_idx = 0;
        let mut dash_dash = false;

        while i < args.len() {
            let arg = &args[i];

            if dash_dash {
                consume_positional(&mut result, schema, &mut positional_idx, arg)?;
                i += 1;
                continue;
            }

            if arg == "--" {
                dash_dash = true;
                i += 1;
                continue;
            }

            if arg == "-h" || arg == "--help" {
                return Err(Error::HelpRequested);
            }

            // Long option: --name or --name=value
            if let Some(rest) = arg.strip_prefix("--") {
                let (name, inline) = split_eq(rest);
                let opt = schema
                    .find_option_long(name)
                    .ok_or_else(|| Error::UnknownOption(arg.clone()))?;
                i = consume_option(opt, args, i, inline, &mut result)?;
                continue;
            }

            // Short option: -x (possibly with attached value later; simple form
            // "-x value" only in Phase 1, no clustering).
            if arg.len() > 1 && arg.starts_with('-') {
                let name = &arg[1..];
                let opt = schema
                    .find_option_short(name)
                    .ok_or_else(|| Error::UnknownOption(arg.clone()))?;
                i = consume_option(opt, args, i, None, &mut result)?;
                continue;
            }

            // Subcommand name or positional.
            if !schema.subcommands.is_empty() {
                if let Some(sub) = schema.find_subcommand(arg) {
                    let sub_result = OptionParser::parse(sub, &args[i + 1..])?;
                    result.subcommand = Some((sub.name.clone(), Box::new(sub_result)));
                    return finalize(result, schema);
                }
                return Err(Error::UnknownSubcommand {
                    name: arg.clone(),
                    available: schema.subcommands.iter().map(|s| s.name.clone()).collect(),
                });
            }

            consume_positional(&mut result, schema, &mut positional_idx, arg)?;
            i += 1;
        }

        finalize(result, schema)
    }
}

fn populate_short_map(map: &mut HashMap<String, String>, schema: &CommandSchema) {
    for opt in &schema.options {
        if let (Some(short), Some(long)) = (&opt.short, &opt.long) {
            let short = short.trim_start_matches('-').to_string();
            let long = long.trim_start_matches('-').to_string();
            map.insert(short, long);
        }
    }
}

fn split_eq(s: &str) -> (&str, Option<&str>) {
    match s.find('=') {
        Some(idx) => (&s[..idx], Some(&s[idx + 1..])),
        None => (s, None),
    }
}

fn consume_option(
    opt: &CLOption,
    args: &[String],
    mut i: usize,
    inline: Option<&str>,
    result: &mut ParseResult,
) -> Result<usize> {
    let key = opt.canonical();
    let token = &args[i];
    if opt.takes_value {
        let value = if let Some(v) = inline {
            v.to_string()
        } else {
            i += 1;
            args.get(i)
                .ok_or_else(|| Error::MissingValue(token.clone()))?
                .clone()
        };
        result.values.entry(key).or_default().push(value);
    } else {
        if inline.is_some() {
            return Err(Error::UnexpectedValue(token.clone()));
        }
        result.flags.insert(key);
    }
    Ok(i + 1)
}

fn consume_positional(
    result: &mut ParseResult,
    schema: &CommandSchema,
    positional_idx: &mut usize,
    value: &str,
) -> Result<()> {
    let arg_def = schema
        .arguments
        .get(*positional_idx)
        .ok_or_else(|| Error::ExtraArgument(value.to_string()))?;
    result.args.insert(arg_def.name.clone(), value.to_string());
    *positional_idx += 1;
    Ok(())
}

fn finalize(result: ParseResult, schema: &CommandSchema) -> Result<ParseResult> {
    // Skip validation when a subcommand took over — positional args belong to
    // the subcommand, not the parent.
    if result.subcommand.is_some() {
        return Ok(result);
    }

    for arg in &schema.arguments {
        if arg.required && !result.args.contains_key(&arg.name) {
            return Err(Error::MissingArgument(arg.name.clone()));
        }
    }

    if !schema.subcommands.is_empty() && result.subcommand.is_none() {
        return Err(Error::MissingSubcommand {
            available: schema.subcommands.iter().map(|s| s.name.clone()).collect(),
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_flag_and_value_option() {
        let schema = CommandSchema::new("app", "")
            .flag("-v,--verbose", "v")
            .option("-n,--count", "n");
        let r = OptionParser::parse(&schema, &args(&["-v", "--count", "3"])).unwrap();
        assert!(r.flag("--verbose"));
        assert!(r.flag("-v"));
        assert_eq!(r.get::<u32>("--count").unwrap(), Some(3));
        assert_eq!(r.get::<u32>("-n").unwrap(), Some(3));
    }

    #[test]
    fn parses_equals_form() {
        let schema = CommandSchema::new("app", "").option("--count", "");
        let r = OptionParser::parse(&schema, &args(&["--count=7"])).unwrap();
        assert_eq!(r.get::<u32>("--count").unwrap(), Some(7));
    }

    #[test]
    fn required_argument_reported_when_missing() {
        let schema = CommandSchema::new("app", "").argument("file", "input");
        let err = OptionParser::parse(&schema, &args(&[])).unwrap_err();
        assert!(matches!(err, Error::MissingArgument(ref n) if n == "file"));
    }

    #[test]
    fn optional_argument_absent_is_ok() {
        let schema = CommandSchema::new("app", "").optional_argument("out", "output");
        let r = OptionParser::parse(&schema, &args(&[])).unwrap();
        assert!(r.get::<String>("out").unwrap().is_none());
    }

    #[test]
    fn repeated_option_captures_all() {
        let schema = CommandSchema::new("app", "").option("-f,--file", "file");
        let r = OptionParser::parse(&schema, &args(&["-f", "a", "--file", "b"])).unwrap();
        assert_eq!(
            r.all::<String>("--file").unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn dash_dash_treats_remainder_as_positional() {
        let schema = CommandSchema::new("app", "")
            .flag("-v,--verbose", "")
            .argument("first", "")
            .argument("second", "");
        let r = OptionParser::parse(&schema, &args(&["-v", "--", "-x", "-y"])).unwrap();
        assert!(r.flag("-v"));
        assert_eq!(r.require::<String>("first").unwrap(), "-x");
        assert_eq!(r.require::<String>("second").unwrap(), "-y");
    }

    #[test]
    fn help_requested_returns_sentinel() {
        let schema = CommandSchema::new("app", "");
        let err = OptionParser::parse(&schema, &args(&["--help"])).unwrap_err();
        assert!(matches!(err, Error::HelpRequested));
    }

    #[test]
    fn subcommand_dispatch() {
        let sub = CommandSchema::new("clone", "")
            .argument("url", "")
            .option("--depth", "");
        let schema = CommandSchema::new("git", "")
            .flag("-v,--verbose", "")
            .subcommand(sub);
        let r = OptionParser::parse(
            &schema,
            &args(&["-v", "clone", "--depth", "1", "https://x"]),
        )
        .unwrap();
        assert!(r.flag("-v"));
        let (name, sub_r) = r.subcommand().unwrap();
        assert_eq!(name, "clone");
        assert_eq!(sub_r.require::<u32>("--depth").unwrap(), 1);
        assert_eq!(sub_r.require::<String>("url").unwrap(), "https://x");
    }

    #[test]
    fn missing_subcommand_reported() {
        let schema = CommandSchema::new("git", "").subcommand(CommandSchema::new("init", ""));
        let err = OptionParser::parse(&schema, &args(&[])).unwrap_err();
        assert!(matches!(err, Error::MissingSubcommand { .. }));
    }

    #[test]
    fn unknown_subcommand_reports_alternatives() {
        let schema = CommandSchema::new("git", "").subcommand(CommandSchema::new("init", ""));
        let err = OptionParser::parse(&schema, &args(&["clone"])).unwrap_err();
        match err {
            Error::UnknownSubcommand { name, available } => {
                assert_eq!(name, "clone");
                assert_eq!(available, vec!["init".to_string()]);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn unknown_option_rejected() {
        let schema = CommandSchema::new("app", "");
        let err = OptionParser::parse(&schema, &args(&["--nope"])).unwrap_err();
        assert!(matches!(err, Error::UnknownOption(ref s) if s == "--nope"));
    }

    #[test]
    fn flag_with_inline_value_rejected() {
        let schema = CommandSchema::new("app", "").flag("--verbose", "");
        let err = OptionParser::parse(&schema, &args(&["--verbose=1"])).unwrap_err();
        assert!(matches!(err, Error::UnexpectedValue(_)));
    }

    #[test]
    fn extra_positional_rejected() {
        let schema = CommandSchema::new("app", "").argument("file", "");
        let err = OptionParser::parse(&schema, &args(&["a", "b"])).unwrap_err();
        assert!(matches!(err, Error::ExtraArgument(ref s) if s == "b"));
    }
}
