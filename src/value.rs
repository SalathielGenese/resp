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

    fn extract_simple_string(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> { // 1990-08-07
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
            Some('+') => Value::extract_simple_string(source),
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
    fn value_implement_try_from_resp_integer() {
        let value: Result<Value, String> = ":10\r\n".try_into();
        assert_eq!(value, Ok(Value::Integer(10i64)));
    }

    #[test]
    fn value_implement_try_from_resp_simple_string() {
        let value: Result<Value, String> = "+Anatomy\r\n".try_into();
        assert_eq!(value, Ok(Value::String("Anatomy".into())));
    }
}
