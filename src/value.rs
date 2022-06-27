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

impl Value {
    fn extract_integer(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        // TODO: Support negative numbers
        let mut chars = source.chars();
        let mut length = 0usize;

        chars.next();
        while let Some('0'..='9') = chars.next() { length += 1; }
        if "\r" == &source[1+length..2+length] && (Some('\n') == chars.next()) {
            let raw = source[1..1+length].to_string();
            return match raw.parse::<i64>() {
                Ok(value) => Ok(Value::Integer(value)),
                _ => Err(format!("Cannot parse '{}' into i64", raw)),
            };
        }

        Err(UNEXPECTED_INPUT.into())
    }

    fn extract_bulk_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        match Self::extract_integer(source) {
            Ok(Value::Integer(len)) => {
                let len = len as usize;
                let post_size_index = 1 + len.to_string().len();
                let post_value_index = post_size_index + 2 + len;
                let value = source[post_size_index+2..post_value_index].to_string();

                if len == value.as_bytes().len() && "\r\n" == &source[post_value_index..post_value_index+2] {
                    return Ok(Value::String(value));
                }

                Err(UNEXPECTED_INPUT.into())
            },
            Err(reason) => Err(reason),
            _ => Err(UNEXPECTED_INPUT.into()),
        }
    }

    fn extract_simple_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        let mut chars = source.chars();
        let mut length = 0usize;

        chars.next();
        while let Some(char) = chars.next() {
            if char != '\r' && char != '\n' {length += 1} else {break}
        }
        if "\r" == &source[1+length..2+length] && (Some('\n') == chars.next()) {
            return Ok(Value::String(source[1..1+length].to_string()));
        }

        Err(UNEXPECTED_INPUT.into())
    }
}

impl TryFrom<&str> for Value {
    type Error = String;

    fn try_from(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        match source.chars().next() {
            Some(':') => Value::extract_integer(source),
            Some('$') => Value::extract_bulk_string(source),
            Some('+') => Value::extract_simple_string(source),
            Some('-') => match Value::extract_simple_string(source) {
                Ok(Value::String(message)) => Ok(Value::Error(message)),
                Err(reason) => Err(reason),
                _ => Err(UNEXPECTED_INPUT.into()),
            },
            _ => Err(UNEXPECTED_INPUT.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Value};

    #[test]
    fn value_implement_try_from() {
        let _value: Result<Value, String> = "".try_into();
    }

    #[test]
    fn value_implement_try_from_resp_error() {
        let value: Result<Value, String> = "-My bad\r\n".try_into();
        assert_eq!(value, Ok(Value::Error("My bad".into())));
    }

    #[test]
    fn value_implement_try_from_resp_integer() {
        let value: Result<Value, String> = ":10\r\n".try_into();
        assert_eq!(value, Ok(Value::Integer(10i64)));
    }

    #[test]
    fn value_implement_try_from_resp_bulk_string() {
        assert_eq!("$4\r\nOops\r\n".try_into(), Ok(Value::String("Oops".into())));
        assert_eq!("$7\r\nOh\r\nOh!\r\n".try_into(), Ok(Value::String("Oh\r\nOh!".into())));
    }

    #[test]
    fn value_implement_try_from_resp_simple_string() {
        let value: Result<Value, String> = "+Anatomy\r\n".try_into();
        assert_eq!(value, Ok(Value::String("Anatomy".into())));
    }
}
