use nu_ansi_term::{Color, Style};
use tracing::{Level, Subscriber};
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

pub struct UniFormatter {
    pub show_source: bool,
    pub use_color: bool,
}

impl UniFormatter {
    pub fn new(use_color: bool) -> Self {
        Self {
            show_source: true,
            use_color,
        }
    }

    fn level_color(&self, level: &Level) -> Style {
        if !self.use_color {
            return Style::default();
        }
        match *level {
            Level::ERROR => Color::Red.normal(),
            Level::WARN => Color::Yellow.normal(),
            Level::INFO => Color::Cyan.normal(),
            Level::DEBUG => Color::Green.normal(),
            Level::TRACE => Color::Purple.normal(),
        }
    }

    fn dim(&self) -> Style {
        if self.use_color {
            Color::Blue.normal()
        } else {
            Style::default()
        }
    }

    fn white(&self) -> Style {
        if self.use_color {
            Color::White.bold()
        } else {
            Style::default()
        }
    }
}

impl<S, N> FormatEvent<S, N> for UniFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let level = meta.level();
        let dim = self.dim();
        let level_style = self.level_color(level);
        let white = self.white();

        // Timestamp: YYYY-MM-DD HH:MM:SS.mmm±HHMM
        let now = chrono::Local::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S%.3f%z").to_string();
        write!(writer, "{} ", dim.paint(&ts))?;

        // Level: right-padded to 5 chars
        let level_str = format!("{:>5}", level);
        write!(writer, "{} ", level_style.paint(&level_str))?;

        // Logger name: short leaf name from target
        let target = meta.target();
        let leaf = target.rsplit("::").next().unwrap_or(target);
        write!(writer, "[{}] ", white.paint(leaf))?;

        // Span context
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                let exts = span.extensions();
                if let Some(fields) = exts.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(writer, "{}{{{}}} ", level_style.paint(span.name()), fields)?;
                    } else {
                        write!(writer, "{} ", level_style.paint(span.name()))?;
                    }
                }
            }
        }

        // Message
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);
        write!(writer, "{}", level_style.paint(&visitor.message))?;

        // Structured fields (key=value)
        if !visitor.fields.is_empty() {
            write!(writer, " {}", dim.paint(&visitor.fields))?;
        }

        // Source location: - (filename:line)
        if let (true, Some(file), Some(line)) = (self.show_source, meta.file(), meta.line()) {
            let filename = file.rsplit('/').next().unwrap_or(file);
            write!(
                writer,
                " {} {}",
                dim.paint("-"),
                dim.paint(format!("({}:{})", filename, line))
            )?;
        }

        writeln!(writer)
    }
}

struct MessageVisitor {
    message: String,
    fields: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: String::new(),
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        } else {
            if !self.fields.is_empty() {
                self.fields.push_str(", ");
            }
            self.fields
                .push_str(&format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            if !self.fields.is_empty() {
                self.fields.push_str(", ");
            }
            self.fields
                .push_str(&format!("{}=\"{}\"", field.name(), value));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if !self.fields.is_empty() {
            self.fields.push_str(", ");
        }
        self.fields.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if !self.fields.is_empty() {
            self.fields.push_str(", ");
        }
        self.fields.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if !self.fields.is_empty() {
            self.fields.push_str(", ");
        }
        self.fields.push_str(&format!("{}={}", field.name(), value));
    }
}
