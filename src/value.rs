use std::convert::TryFrom;
use std::cmp::PartialEq;

/// A wrapper type for a RESP value.
///
/// This enum implements the `TryFrom` trait (`TryFrom<&str>`), to provide
/// on-the-fly parsing and validation of RESP strings.
///
/// # Examples
///
/// ```rust
/// use crate::squall_dot_io_resp::Value;
///
/// // JSON: null
/// with_resp("$-1\r\n".try_into());
///
/// // JSON: 10
/// with_resp(":10\r\n".try_into());
///
/// // JSON: "Nina Simone"
/// with_resp("+Nina Simone\r\n".try_into());
///
/// // JSON: "Lorem ipsum...\r\nDolor sit amet..."
/// with_resp("$33\r\nLorem ipsum...\r\nDolor sit amet...\r\n".try_into());
///
/// // JavaScript: [null, 447, new Error("Oh oh!"), "Hourly", "Si vis pacem,\r\npara bellum"]
/// with_resp("*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n"
///           .try_into());
///
/// // NOTE: Even recursive arrays - we leave that for you to try out.
///
/// fn with_resp(input: Result<Value, String>) {
///     println!("{:?}", input);
/// }
/// ```
///
#[derive(Debug,PartialEq)]
pub enum Value {
    /// Denote the absence of value.
    Nil,
    /// Denote and integer value, wrapped as singleton tuple.
    Integer(i64),
    /// Denote an error, wrapped as descriptive message string.
    Error(String),
    /// Denote a string value, wrapped as singleton tuple.
    String(String),
    /// Denote a non-nil list of values, wrapped as singleton vector of Value.
    Array(Vec<Value>),
}

const UNEXPECTED_INPUT: &str = "Unexpected input";

impl TryFrom<&str> for Value {
    type Error = String;

    fn try_from(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        Value::internal_try_from(source).0
    }
}

impl Value {
    fn internal_try_from(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        match source.chars().next() {
            Some('*') => Value::extract_array(source),
            Some('-') => Value::extract_error(source),
            Some(':') => Value::extract_integer(source),
            Some('$') => Value::extract_bulk_string(source),
            Some('+') => Value::extract_simple_string(source),
            _ => (Err(UNEXPECTED_INPUT.into()), 0)
        }
    }

    fn extract_array(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        match Value::extract_integer(source) {
            (Ok(Value::Integer(len)), size) => {
                let mut offset = size;
                let mut values = vec![];

                while values.len() < len as usize {
                    match Value::internal_try_from(&source[offset..source.len()]) {
                        (Ok(value), size) => {
                            values.push(value);
                            offset += size;
                        },
                        r#else => return r#else,
                    }
                }

                (Ok(Value::Array(values)), offset)
            },
            r#else => return r#else,
        }
    }

    fn extract_error(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        match Value::extract_simple_string(source) {
            (Ok(Value::String(message)), size) => (Ok(Value::Error(message)), size),
            r#else => r#else,
        }
    }

    fn extract_integer(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        // TODO: Support negative numbers
        if let Some(i) = source.find("\r\n") {
            if let &Ok(value) = &source[1..i].parse::<i64>() {
                return (Ok(Value::Integer(value)), i+2);
            }
        }

        (Err(UNEXPECTED_INPUT.into()), 0)
    }

    fn extract_bulk_string(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        if source.starts_with("$-1\r\n") {
            return (Ok(Value::Nil), 5);
        }

        if let (Ok(Value::Integer(size)), _) = Self::extract_integer(source) {
            let start = 1 + size.to_string().len() + 2;
            let end = start + size as usize;

            if "\r\n" == &source[end..end+2] {
                return (Ok(Value::String(source[start..end].to_string())), end+2);
            }
        }

        (Err(UNEXPECTED_INPUT.into()), 0)
    }

    fn extract_simple_string(source: &str) -> (Result<Self, <Value as TryFrom<&str>>::Error>, usize) {
        if let Some(i) = source.find("\r\n") {
            if !source[1..i].contains('\r') && !source[1..i].contains('\n') {
                return (Ok(Value::String(source[1..i].into())), i+2);
            }
        }

        (Err(UNEXPECTED_INPUT.into()), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::{UNEXPECTED_INPUT, Value};

    #[test]
    fn value_implement_try_from_resp_nil() {
        assert_eq!("$-1\r\n".try_into(), Ok(Value::Nil));
    }

    #[test]
    fn value_implement_try_from() {
        let _value: Result<Value, String> = "".try_into();
    }

    #[test]
    fn value_implement_try_from_resp_array() {
        assert_eq!("*0\r\n".try_into(), Ok(Value::Array(vec![])));
        assert_eq!("*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n".try_into(),
        Ok(Value::Array(vec![
            Value::Nil,
            Value::Integer(447),
            Value::Error("Oh oh!".into()),
            Value::String("Hourly".into()),
            Value::String("Si vis pacem,\r\npara bellum".into()),
        ])));
    }

    #[test]
    fn value_implement_try_from_resp_nested_array() {
        let got = "*2\r\n*3\r\n+A\r\n+B\r\n+C\r\n*3\r\n:1\r\n:2\r\n:3\r\n".try_into() as Result<Value, String>;
        let expected = Ok(Value::Array(vec![
            Value::Array(vec![Value::String("A".into()), Value::String("B".into()), Value::String("C".into())]),
            Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]),
        ])) as Result<Value, String>;

        assert_eq!(got, expected);
    }

    #[test]
    fn value_implement_try_from_resp_array_with_invalid_size() {
        assert_eq!("*!\r\n$-1\r\n".try_into() as Result<Value, String>, Err(UNEXPECTED_INPUT.into()));
    }

    #[test]
    fn value_implement_try_from_resp_error() {
        assert_eq!("-My bad\r\n".try_into(), Ok(Value::Error("My bad".into())));
    }

    #[test]
    fn value_implement_try_from_resp_integer() {
        assert_eq!(":10\r\n".try_into(), Ok(Value::Integer(10i64)));
    }

    #[test]
    fn value_implement_try_from_resp_integer_with_invalid_integer() {
        assert_eq!(":Yikes\r\n".try_into() as Result<Value, String>, Err(UNEXPECTED_INPUT.into()));
    }

    #[test]
    fn value_implement_try_from_resp_bulk_string() {
        assert_eq!("$4\r\nOops\r\n".try_into(), Ok(Value::String("Oops".into())));
        assert_eq!("$7\r\nOh\r\nOh!\r\n".try_into(), Ok(Value::String("Oh\r\nOh!".into())));
    }

    #[test]
    fn value_implement_try_from_resp_bulk_string_with_invalid_len() {
        assert_eq!("$3\r\nOops\r\n".try_into() as Result<Value, String>, Err(UNEXPECTED_INPUT.into()));
    }

    #[test]
    fn value_implement_try_from_resp_simple_string() {
        assert_eq!("+Anatomy\r\n".try_into(), Ok(Value::String("Anatomy".into())));
    }

    #[test]
    fn value_implement_try_from_resp_simple_string_with_line_feed_in_value() {
        assert_eq!("+Top\nBottom\r\n".try_into() as Result<Value, String>, Err(UNEXPECTED_INPUT.into()));
    }
}
