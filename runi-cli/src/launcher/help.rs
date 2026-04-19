use crate::tint::{Tint, supports_color, supports_color_stdout};

use super::schema::{CLArgument, CLOption, CommandSchema};

/// Format help output for a schema. The result is ANSI-styled when the
/// destination stream is a TTY and plain otherwise.
pub struct HelpPrinter;

impl HelpPrinter {
    /// Produce help text sized for stderr (color when stderr is a TTY).
    /// Prefer [`HelpPrinter::print`] / [`HelpPrinter::print_error`] in
    /// production; this method is primarily here for tests.
    pub fn format(schema: &CommandSchema) -> String {
        Self::format_with_color(schema, supports_color())
    }

    fn format_with_color(schema: &CommandSchema, color: bool) -> String {
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
            let rows: Vec<Row> = schema
                .arguments
                .iter()
                .map(|a| argument_row(a, color))
                .collect();
            write_rows(&mut out, &rows);
            out.push('\n');
        }

        out.push_str(&bold("Options:", color));
        out.push('\n');
        let mut option_rows: Vec<Row> = schema
            .options
            .iter()
            .map(|o| option_row(o, color))
            .collect();
        option_rows.push(help_row(color));
        write_rows(&mut out, &option_rows);

        if !schema.subcommands.is_empty() {
            out.push('\n');
            out.push_str(&bold("Subcommands:", color));
            out.push('\n');
            let rows: Vec<Row> = schema
                .subcommands
                .iter()
                .map(|s| subcommand_row(s, color))
                .collect();
            write_rows(&mut out, &rows);
        }

        out
    }

    /// Print help text to stdout. Use this for user-requested help
    /// (`--help`) so output can be piped or redirected normally. Color is
    /// keyed off stdout, so redirected stdout is always plain.
    /// Flushes stdout before returning so callers that `process::exit`
    /// immediately afterwards still see all bytes land.
    pub fn print(schema: &CommandSchema) {
        use std::io::Write;
        let text = Self::format_with_color(schema, supports_color_stdout());
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        let _ = lock.write_all(text.as_bytes());
        let _ = lock.flush();
    }

    /// Print help text to stderr. Use this alongside an error message so
    /// both are grouped on the same stream.
    pub fn print_error(schema: &CommandSchema) {
        use std::io::Write;
        let text = Self::format_with_color(schema, supports_color());
        let stderr = std::io::stderr();
        let mut lock = stderr.lock();
        let _ = lock.write_all(text.as_bytes());
        let _ = lock.flush();
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

/// One formatted row (argument, option, subcommand, or the `-h, --help`
/// line). `head_plain` is kept alongside the styled `head` so the layout
/// pass can align columns on visible width — ANSI escape sequences in
/// `head` would otherwise inflate `.chars().count()`.
struct Row {
    head_plain: String,
    head: String,
    description: String,
}

fn option_row(opt: &CLOption, color: bool) -> Row {
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
    Row {
        head_plain: head.clone(),
        head: if color {
            Tint::cyan().paint(&head)
        } else {
            head
        },
        description: dim(&opt.description, color),
    }
}

fn argument_row(arg: &CLArgument, color: bool) -> Row {
    let head_plain = if arg.required {
        format!("<{}>", arg.name)
    } else {
        format!("[{}]", arg.name)
    };
    Row {
        head: if color {
            Tint::green().paint(&head_plain)
        } else {
            head_plain.clone()
        },
        head_plain,
        description: dim(&arg.description, color),
    }
}

fn subcommand_row(sub: &CommandSchema, color: bool) -> Row {
    Row {
        head_plain: sub.name.clone(),
        head: if color {
            Tint::cyan().paint(&sub.name)
        } else {
            sub.name.clone()
        },
        description: dim(&sub.description, color),
    }
}

fn help_row(color: bool) -> Row {
    let head_plain = "-h, --help".to_string();
    Row {
        head: if color {
            Tint::cyan().paint(&head_plain)
        } else {
            head_plain.clone()
        },
        head_plain,
        description: dim("Show this help message", color),
    }
}

/// Align descriptions by padding each `head` to the longest plain-width in
/// the section, plus a 4-space gutter. Rows without descriptions are
/// emitted unpadded.
fn write_rows(out: &mut String, rows: &[Row]) {
    let max_head = rows
        .iter()
        .map(|r| r.head_plain.chars().count())
        .max()
        .unwrap_or(0);
    for row in rows {
        out.push_str("  ");
        out.push_str(&row.head);
        if !row.description.is_empty() {
            let pad = max_head.saturating_sub(row.head_plain.chars().count()) + 4;
            for _ in 0..pad {
                out.push(' ');
            }
            out.push_str(&row.description);
        }
        out.push('\n');
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
    fn options_descriptions_are_column_aligned() {
        // Different head widths must produce the same description column.
        let s = CommandSchema::new("app", "")
            .flag("-v,--verbose", "Verbose output")
            .option("-n,--count", "Count");
        let out = no_ansi(&HelpPrinter::format(&s));
        // Find the start column of each description.
        let verbose_col = out.find("Verbose output").unwrap();
        let count_col = out.find("Count").unwrap();
        // Each line starts at column 0 after a newline; compute offset on
        // its own line.
        let verbose_line = out[..verbose_col].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let count_line = out[..count_col].rfind('\n').map(|i| i + 1).unwrap_or(0);
        assert_eq!(
            verbose_col - verbose_line,
            count_col - count_line,
            "option descriptions must start in the same column"
        );
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
