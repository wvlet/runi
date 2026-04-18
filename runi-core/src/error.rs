use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    pub fn msg(s: impl fmt::Display) -> Self {
        Self::Message(s.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_string() {
        let e = Error::msg("something went wrong");
        assert_eq!(e.to_string(), "something went wrong");
    }

    #[test]
    fn result_with_error() {
        let r: Result<()> = Err(Error::msg("fail"));
        assert!(r.is_err());
    }
}
