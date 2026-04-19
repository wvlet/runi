use std::fmt;

/// Errors produced by the launcher when parsing command-line arguments.
#[derive(Debug)]
pub enum Error {
    UnknownOption(String),
    UnknownSubcommand {
        name: String,
        available: Vec<String>,
    },
    MissingValue(String),
    UnexpectedValue(String),
    MissingArgument(String),
    MissingSubcommand {
        available: Vec<String>,
    },
    ExtraArgument(String),
    InvalidValue {
        name: String,
        value: String,
        message: String,
    },
    /// Sentinel used internally when the user passes `-h`/`--help`.
    HelpRequested,
    /// Wraps an error produced while parsing a subcommand so the launcher can
    /// print the right (sub)command help. `path` is the chain from the root
    /// schema to the failing subcommand, newest-last.
    InSubcommand {
        path: Vec<String>,
        source: Box<Error>,
    },
    Custom(String),
}

impl Error {
    pub fn invalid_value(
        name: impl Into<String>,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidValue {
            name: name.into(),
            value: value.into(),
            message: message.into(),
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownOption(opt) => write!(f, "unknown option: {opt}"),
            Error::UnknownSubcommand { name, available } => {
                write!(f, "unknown subcommand: {name}")?;
                if !available.is_empty() {
                    write!(f, " (available: {})", available.join(", "))?;
                }
                Ok(())
            }
            Error::MissingValue(opt) => write!(f, "missing value for option: {opt}"),
            Error::UnexpectedValue(opt) => write!(f, "option {opt} does not take a value"),
            Error::MissingArgument(name) => write!(f, "missing required argument: <{name}>"),
            Error::MissingSubcommand { available } => {
                write!(f, "a subcommand is required")?;
                if !available.is_empty() {
                    write!(f, " (available: {})", available.join(", "))?;
                }
                Ok(())
            }
            Error::ExtraArgument(arg) => write!(f, "unexpected argument: {arg}"),
            Error::InvalidValue {
                name,
                value,
                message,
            } => {
                write!(f, "invalid value '{value}' for {name}: {message}")
            }
            Error::HelpRequested => write!(f, "help requested"),
            Error::InSubcommand { path, source } => {
                write!(f, "in subcommand '{}': {}", path.join(" "), source)
            }
            Error::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
