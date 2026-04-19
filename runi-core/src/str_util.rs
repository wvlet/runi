pub fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use runi_test::prelude::*;
    use runi_test::pretty_assertions::assert_eq;

    #[rstest]
    #[case("hello", 10, "hello")]
    #[case("hello", 5, "hello")]
    #[case("hello world", 5, "hello")]
    #[case("こんにちは", 6, "こん")]
    fn test_truncate(#[case] input: &str, #[case] max: usize, #[case] expected: &str) {
        assert_eq!(truncate(input, max), expected);
    }

    #[rstest]
    #[case("", true)]
    #[case("   ", true)]
    #[case(" \t\n ", true)]
    #[case("a", false)]
    fn test_is_blank(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_blank(input), expected);
    }

    #[rstest]
    #[case("HelloWorld", "hello_world")]
    #[case("helloWorld", "hello_world")]
    #[case("hello", "hello")]
    fn test_to_snake_case(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_snake_case(input), expected);
    }

    #[rstest]
    #[case("hello_world", "helloWorld")]
    #[case("hello-world", "helloWorld")]
    #[case("hello", "hello")]
    fn test_to_camel_case(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_camel_case(input), expected);
    }
}
