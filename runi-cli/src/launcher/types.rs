use std::fmt;
use std::str::FromStr;

/// Convert a raw command-line argument string into a typed value.
///
/// A blanket implementation covers every `T: FromStr` with a `Display` error,
/// so any type that already implements `FromStr` works automatically. Implement
/// this trait directly only for types that cannot use `FromStr` (for example,
/// to support a custom syntax).
pub trait FromArg: Sized {
    fn from_arg(raw: &str) -> Result<Self, String>;

    /// Human-readable name of the expected type, used in error messages.
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> FromArg for T
where
    T: FromStr,
    T::Err: fmt::Display,
{
    fn from_arg(raw: &str) -> Result<Self, String> {
        raw.parse::<T>().map_err(|e| e.to_string())
    }
}

/// Sanity checks for the commonly used built-in types. These exist so regressions
/// in the blanket impl surface immediately.
#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn parses_strings_and_paths() {
        assert_eq!(String::from_arg("hello").unwrap(), "hello".to_string());
        assert_eq!(
            PathBuf::from_arg("/tmp/x").unwrap(),
            PathBuf::from("/tmp/x")
        );
    }

    #[test]
    fn parses_numbers() {
        assert_eq!(i32::from_arg("-7").unwrap(), -7);
        assert_eq!(u64::from_arg("42").unwrap(), 42u64);
        assert!((f64::from_arg("2.5").unwrap() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn parses_booleans() {
        assert!(bool::from_arg("true").unwrap());
        assert!(!bool::from_arg("false").unwrap());
    }

    #[test]
    fn reports_error_for_invalid_input() {
        let err = i32::from_arg("not-a-number").unwrap_err();
        assert!(!err.is_empty());
    }
}
