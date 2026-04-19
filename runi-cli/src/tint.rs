use nu_ansi_term::{Color as AnsiColor, Style};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Tint {
    style: Style,
}

impl Tint {
    fn new(style: Style) -> Self {
        Self { style }
    }

    // Foreground colors
    pub fn red() -> Self {
        Self::new(AnsiColor::Red.normal())
    }
    pub fn green() -> Self {
        Self::new(AnsiColor::Green.normal())
    }
    pub fn yellow() -> Self {
        Self::new(AnsiColor::Yellow.normal())
    }
    pub fn blue() -> Self {
        Self::new(AnsiColor::Blue.normal())
    }
    pub fn purple() -> Self {
        Self::new(AnsiColor::Purple.normal())
    }
    pub fn cyan() -> Self {
        Self::new(AnsiColor::Cyan.normal())
    }
    pub fn white() -> Self {
        Self::new(AnsiColor::White.normal())
    }
    pub fn black() -> Self {
        Self::new(AnsiColor::Black.normal())
    }

    // Bright foreground colors
    pub fn bright_red() -> Self {
        Self::new(AnsiColor::LightRed.normal())
    }
    pub fn bright_green() -> Self {
        Self::new(AnsiColor::LightGreen.normal())
    }
    pub fn bright_yellow() -> Self {
        Self::new(AnsiColor::LightYellow.normal())
    }
    pub fn bright_blue() -> Self {
        Self::new(AnsiColor::LightBlue.normal())
    }
    pub fn bright_purple() -> Self {
        Self::new(AnsiColor::LightPurple.normal())
    }
    pub fn bright_cyan() -> Self {
        Self::new(AnsiColor::LightCyan.normal())
    }

    // Style modifiers (chainable)
    pub fn bold(mut self) -> Self {
        self.style = self.style.bold();
        self
    }
    pub fn dimmed(mut self) -> Self {
        self.style = self.style.dimmed();
        self
    }
    pub fn italic(mut self) -> Self {
        self.style = self.style.italic();
        self
    }
    pub fn underline(mut self) -> Self {
        self.style = self.style.underline();
        self
    }
    pub fn strikethrough(mut self) -> Self {
        self.style = self.style.strikethrough();
        self
    }

    // Background colors (chainable)
    pub fn bg_red(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Red);
        self
    }
    pub fn bg_green(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Green);
        self
    }
    pub fn bg_yellow(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Yellow);
        self
    }
    pub fn bg_blue(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Blue);
        self
    }
    pub fn bg_purple(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Purple);
        self
    }
    pub fn bg_cyan(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Cyan);
        self
    }
    pub fn bg_white(mut self) -> Self {
        self.style = self.style.on(AnsiColor::White);
        self
    }
    pub fn bg_black(mut self) -> Self {
        self.style = self.style.on(AnsiColor::Black);
        self
    }

    // ANSI 256 color
    pub fn color(n: u8) -> Self {
        Self::new(AnsiColor::Fixed(n).normal())
    }
    pub fn bg_color(mut self, n: u8) -> Self {
        self.style = self.style.on(AnsiColor::Fixed(n));
        self
    }

    // RGB color
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(AnsiColor::Rgb(r, g, b).normal())
    }
    pub fn bg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.style = self.style.on(AnsiColor::Rgb(r, g, b));
        self
    }

    /// Apply this style to a string and return the styled output.
    pub fn paint(&self, text: &str) -> String {
        self.style.paint(text).to_string()
    }
}

impl fmt::Display for Tint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.style)
    }
}

/// Check if stderr supports color output.
pub fn supports_color() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stderr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_colors() {
        let s = Tint::red().paint("error");
        assert!(s.contains("error"));

        let s = Tint::green().paint("success");
        assert!(s.contains("success"));
    }

    #[test]
    fn chained_styles() {
        let s = Tint::red().bold().underline().paint("important");
        assert!(s.contains("important"));
    }

    #[test]
    fn background_colors() {
        let s = Tint::white().bg_red().paint("alert");
        assert!(s.contains("alert"));
    }

    #[test]
    fn ansi256_color() {
        let s = Tint::color(208).paint("orange");
        assert!(s.contains("orange"));
    }

    #[test]
    fn rgb_color() {
        let s = Tint::rgb(255, 128, 0).paint("custom");
        assert!(s.contains("custom"));
    }

    #[test]
    fn style_reuse() {
        let header = Tint::cyan().bold();
        let a = header.paint("Title A");
        let b = header.paint("Title B");
        assert!(a.contains("Title A"));
        assert!(b.contains("Title B"));
    }
}
