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

impl TryFrom<&str> for Value {
    type Error = &'static str;

    fn try_from(_source: &str) -> Result<Self, <Value as TryFrom<&str>>::Error> {
        Err("Unexpected input")
    }
}

#[cfg(test)]
mod tests {
    use super::{Value};

    #[test]
    fn value_implement_try_from() {
        let _value: Result<Value, &str> = "".try_into();
    }
}
