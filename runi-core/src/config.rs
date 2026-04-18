use std::collections::HashMap;
use std::env;

#[derive(Debug, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env(prefix: &str) -> Self {
        let mut values = HashMap::new();
        let prefix_upper = format!("{}_", prefix.to_uppercase());
        for (key, value) in env::vars() {
            if let Some(stripped) = key.strip_prefix(&prefix_upper) {
                values.insert(stripped.to_lowercase(), value);
            }
        }
        Self { values }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::pretty_assertions::assert_eq;
    use runi_test::prelude::*;

    #[fixture]
    fn config() -> Config {
        let mut c = Config::new();
        c.set("host", "localhost");
        c.set("port", "8080");
        c.set("name", "runi");
        c
    }

    #[test]
    fn empty_config() {
        let c = Config::new();
        assert!(c.is_empty());
        assert_eq!(c.get("missing"), None);
    }

    #[rstest]
    fn get_values(config: Config) {
        assert_eq!(config.get("host"), Some("localhost"));
        assert_eq!(config.get("port"), Some("8080"));
        assert_eq!(config.len(), 3);
    }

    #[rstest]
    fn get_with_default(config: Config) {
        assert_eq!(config.get_or("missing", "fallback"), "fallback");
        assert_eq!(config.get_or("host", "fallback"), "localhost");
    }

    #[rstest]
    #[case("port", Some(8080))]
    #[case("name", None)]
    #[case("missing", None)]
    fn parse_u64(config: Config, #[case] key: &str, #[case] expected: Option<u64>) {
        assert_eq!(config.get_u64(key), expected);
    }

    #[test]
    fn from_env() {
        unsafe {
            env::set_var("RUNI_TEST_HOST", "localhost");
            env::set_var("RUNI_TEST_PORT", "9090");
        }
        let c = Config::from_env("RUNI_TEST");
        assert_eq!(c.get("host"), Some("localhost"));
        assert_eq!(c.get("port"), Some("9090"));
        unsafe {
            env::remove_var("RUNI_TEST_HOST");
            env::remove_var("RUNI_TEST_PORT");
        }
    }
}
