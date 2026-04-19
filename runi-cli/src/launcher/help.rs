use crate::tint::{Tint, supports_color};

use super::schema::{CLArgument, CLOption, CommandSchema};

/// Format help output for a schema. The result is already ANSI-styled when
/// `supports_color()` is true, and plain when writing to a non-TTY.
pub struct HelpPrinter;

impl HelpPrinter {
    /// Produce help text for a schema. Prefer [`HelpPrinter::print`] in
    /// production; this method is primarily here for tests.
    pub fn format(schema: &CommandSchema) -> String {
        let color = supports_color();
        let mut out = String::new();

        if !schema.description.is_empty() {
            out.push_str(&schema.description);
            out.push_str("\n\n");
        }

        out.push_str(&bold("Usage:", color));
        out.push(' ');
        out.push_str(&usage_line(schema));
        out.push_str("\n\n");

        if !schema.arguments.is_empty() {
            out.push_str(&bold("Arguments:", color));
            out.push('\n');
            for arg in &schema.arguments {
                format_argument(&mut out, arg, color);
            }
            out.push('\n');
        }

        out.push_str(&bold("Options:", color));
        out.push('\n');
        for opt in &schema.options {
            format_option(&mut out, opt, color);
        }
        out.push_str(&format_help_line(color));
        out.push('\n');

        if !schema.subcommands.is_empty() {
            out.push('\n');
            out.push_str(&bold("Subcommands:", color));
            out.push('\n');
            for sub in &schema.subcommands {
                format_subcommand(&mut out, sub, color);
            }
        }

        out
    }

    /// Print help text to stdout. Use this for user-requested help
    /// (`--help`) so output can be piped or redirected normally.
    pub fn print(schema: &CommandSchema) {
        print!("{}", Self::format(schema));
    }

    /// Print help text to stderr. Use this alongside an error message so
    /// both are grouped on the same stream.
    pub fn print_error(schema: &CommandSchema) {
        eprint!("{}", Self::format(schema));
    }
}

fn usage_line(schema: &CommandSchema) -> String {
    let mut parts: Vec<String> = vec![schema.name.clone()];
    if !schema.options.is_empty() {
        parts.push("[OPTIONS]".to_string());
    }
    // Match the parser: positionals bind before subcommand dispatch, so the
    // usage line must present them in the same order the user types them.
    for arg in &schema.arguments {
        if arg.required {
            parts.push(format!("<{}>", arg.name));
        } else {
            parts.push(format!("[{}]", arg.name));
        }
    }
    if !schema.subcommands.is_empty() {
        parts.push("<COMMAND>".to_string());
    }
    parts.join(" ")
}

fn format_option(out: &mut String, opt: &CLOption, color: bool) {
    let mut head = String::new();
    match (&opt.short, &opt.long) {
        (Some(s), Some(l)) => {
            head.push_str(s);
            head.push_str(", ");
            head.push_str(l);
        }
        (Some(s), None) => head.push_str(s),
        (None, Some(l)) => {
            head.push_str("    ");
            head.push_str(l);
        }
        (None, None) => {}
    }
    if opt.takes_value {
        head.push_str(&format!(" <{}>", opt.value_name));
    }
    let head = if color {
        Tint::cyan().paint(&head)
    } else {
        head
    };
    out.push_str("  ");
    out.push_str(&head);
    if !opt.description.is_empty() {
        pad_to(out, 4);
        out.push_str(&dim(&opt.description, color));
    }
    out.push('\n');
}

fn format_argument(out: &mut String, arg: &CLArgument, color: bool) {
    let display = if arg.required {
        format!("<{}>", arg.name)
    } else {
        format!("[{}]", arg.name)
    };
    let head = if color {
        Tint::green().paint(&display)
    } else {
        display
    };
    out.push_str("  ");
    out.push_str(&head);
    if !arg.description.is_empty() {
        pad_to(out, 4);
        out.push_str(&dim(&arg.description, color));
    }
    out.push('\n');
}

fn format_subcommand(out: &mut String, sub: &CommandSchema, color: bool) {
    let head = if color {
        Tint::cyan().paint(&sub.name)
    } else {
        sub.name.clone()
    };
    out.push_str("  ");
    out.push_str(&head);
    if !sub.description.is_empty() {
        pad_to(out, 4);
        out.push_str(&dim(&sub.description, color));
    }
    out.push('\n');
}

fn format_help_line(color: bool) -> String {
    let head = "-h, --help";
    let head = if color {
        Tint::cyan().paint(head)
    } else {
        head.to_string()
    };
    let mut line = format!("  {head}");
    pad_to(&mut line, 4);
    line.push_str(&dim("Show this help message", color));
    line
}

fn pad_to(out: &mut String, spaces: usize) {
    for _ in 0..spaces {
        out.push(' ');
    }
}

fn bold(s: &str, color: bool) -> String {
    if color {
        Tint::white().bold().paint(s)
    } else {
        s.to_string()
    }
}

fn dim(s: &str, color: bool) -> String {
    if color {
        Tint::white().dimmed().paint(s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;

    fn no_ansi(s: &str) -> String {
        // Strip simple CSI sequences for readable assertions.
        let bytes = s.as_bytes();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'm' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    #[test]
    fn usage_line_includes_arguments_and_subcommands() {
        let s = CommandSchema::new("app", "desc")
            .flag("-v,--verbose", "")
            .argument("file", "");
        assert_eq!(usage_line(&s), "app [OPTIONS] <file>");
    }

    #[test]
    fn usage_line_puts_positionals_before_subcommand() {
        let s = CommandSchema::new("app", "")
            .argument("workspace", "")
            .subcommand(CommandSchema::new("run", ""));
        assert_eq!(usage_line(&s), "app <workspace> <COMMAND>");
    }

    #[test]
    fn help_format_contains_expected_sections() {
        let s = CommandSchema::new("app", "The app")
            .flag("-v,--verbose", "Verbose output")
            .option("-n,--count", "Count")
            .argument("file", "Input file")
            .subcommand(CommandSchema::new("run", "Run it"));
        let out = no_ansi(&HelpPrinter::format(&s));
        assert!(out.contains("The app"));
        assert!(out.contains("Usage:"));
        assert!(out.contains("<file>"));
        assert!(out.contains("Arguments:"));
        assert!(out.contains("Options:"));
        assert!(out.contains("--verbose"));
        assert!(out.contains("--count <val>"));
        assert!(out.contains("Subcommands:"));
        assert!(out.contains("run"));
        assert!(out.contains("-h, --help"));
    }
}
