use std::convert::TryFrom;

/// A wrapper type for a RESP value.
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
const UNSUPPORTED_FEATURE_NESTED_ARRAY: &str = "Unsupported feature: nested array";

impl TryFrom<&str> for Value {
    type Error = String;

    fn try_from(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        match source.chars().next() {
            Some('*') => Value::extract_array(source),
            Some('-') => Value::extract_error(source),
            Some(':') => Value::extract_integer(source),
            Some('$') => Value::extract_bulk_string(source),
            Some('+') => Value::extract_simple_string(source),
            _ => Err(UNEXPECTED_INPUT.into())
        }
    }
}

impl Value {
    fn extract_array(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        match Value::extract_integer(source) {
            Ok(Value::Integer(len)) => {
                let len = len as usize;
                let mut values = vec![];
                let mut offset = 1 + len.to_string().len() + 2;

                while values.len() < len {
                    match Value::try_from(&source[offset..source.len()]) {
                        // TODO: Support nested arrays
                        Ok(Value::Array(_)) => return Err(UNSUPPORTED_FEATURE_NESTED_ARRAY.into()),
                        Ok(Value::Integer(value)) => {
                            offset += 1 + value.to_string().len() + 2;
                            values.push(Value::Integer(value));
                        },
                        Ok(Value::Error(message)) => {
                            offset += 1 + message.len() + 2;
                            values.push(Value::Error(message));
                        },
                        Ok(Value::String(content)) => {
                            offset += 1 + content.len() + 2;
                            values.push(Value::String(content));
                        },
                        Ok(Value::Nil) => {
                            offset += 5;
                            values.push(Value::Nil);
                        },
                        Err(reason) => {
                            return Err(reason);
                        },
                    }
                }

                Ok(Value::Array(values))
            },
            r#else => return r#else,
        }
    }

    fn extract_error(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        match Value::extract_simple_string(source) {
            Ok(Value::String(message)) => Ok(Value::Error(message)),
            r#else => r#else,
        }
    }

    fn extract_integer(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        // TODO: Support negative numbers
        let mut chars = source.chars();
        let mut length = 0usize;

        chars.next();
        while let Some('0'..='9') = chars.next() { length += 1; }
        if "\r\n" == &source[1+length..3+length] {
            let raw = source[1..1+length].to_string();
            return match raw.parse::<i64>() {
                Ok(value) => Ok(Value::Integer(value)),
                _ => Err(format!("Cannot parse '{}' into i64", raw)),
            };
        }

        Err(UNEXPECTED_INPUT.into())
    }

    fn extract_bulk_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        if source.starts_with("$-1\r\n") { return Ok(Value::Nil); }

        match Self::extract_integer(source) {
            Ok(Value::Integer(len)) => {
                let len = len as usize;
                let post_size_index = 1 + len.to_string().len();
                let post_value_index = post_size_index + 2 + len;
                let value = source[post_size_index+2..post_value_index].to_string();

                if "\r\n" == &source[post_value_index..post_value_index+2] {
                    if len == value.len() {
                        return Ok(Value::String(value));
                    }
                }

                Err(UNEXPECTED_INPUT.into())
            },
            r#else => r#else,
        }
    }

    fn extract_simple_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        let mut chars = source.chars();
        let mut length = 0usize;

        chars.next();
        while let Some(char) = chars.next() {
            if char != '\r' && char != '\n' {length += 1} else {break}
        }
        if "\r\n" == &source[1+length..3+length] {
            return Ok(Value::String(source[1..1+length].to_string()));
        }

        Err(UNEXPECTED_INPUT.into())
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
