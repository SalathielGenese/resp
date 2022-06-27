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
        if let Some(i) = source.find("\r\n") {
            if let &Ok(value) = &source[1..i].parse::<i64>() {
                return Ok(Value::Integer(value));
            }
        }

        Err(UNEXPECTED_INPUT.into())
    }

    fn extract_bulk_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        if source.starts_with("$-1\r\n") {
            return Ok(Value::Nil);
        }

        if let Ok(Value::Integer(size)) = Self::extract_integer(source) {
            let start = 1 + size.to_string().len() + 2;
            let end = start + size as usize;

            if "\r\n" == &source[end..end+2] {
                return Ok(Value::String(source[start..end].to_string()));
            }
        }

        Err(UNEXPECTED_INPUT.into())
    }

    fn extract_simple_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        if let Some(i) = source.find("\r\n") {
            if !source[1..i].contains('\r') && !source[1..i].contains('\n') {
                return Ok(Value::String(source[1..i].into()));
            }
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
