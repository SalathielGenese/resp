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

impl TryFrom<&str> for Value {
    type Error = String;

    fn try_from(source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        Err(UNEXPECTED_INPUT.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Value};

    #[test]
    fn value_implement_try_from() {
        let _value: Result<Value, String> = "".try_into();
    }
}
