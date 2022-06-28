use crate::error::Error as TError;
use crate::Node::{ARRAY, BULK_STRING, ERROR, INTEGER, SIMPLE_STRING, SIZE, UNKNOWN};

/// A wrapper type for a RESP value.
///
/// This enum implements the `TryFrom` trait (`TryFrom<&str>`), to provide
/// on-the-fly parsing and validation of RESP strings.
///
/// # Examples
///
/// ```rust
/// use squall_dot_io_resp::{
///     Node::{self, NIL, SIZE, ARRAY, ERROR, INTEGER, UNKNOWN, SIMPLE_STRING, BULK_STRING},
///     Value::{self, Nil, Error, Array, String, Integer},
///     Error as VError,
///     ValueResult,
/// };
///
/// assert_eq!( // Empty RESP
///     "".try_into() as ValueResult,
///     Err(VError::Unexpected {node: UNKNOWN, index: 0}));
///
/// assert_eq!( // Unterminated number: missing "\r\n"
///     ":0".try_into() as ValueResult,
///     Err(VError::Unexpected {node: INTEGER, index: 2}));
///
/// assert_eq!( // Not enough elements in the array
///     "*2\r\n$-1\r\n".try_into() as ValueResult,
///     Err(VError::Size {node: ARRAY, index: 9}));
///
/// assert_eq!( // Longer bulk string: got more that 2-bytes
///     "$2\r\nHello\r\n".try_into() as ValueResult,
///     Err(VError::Size {node: BULK_STRING, index: 6}));
///
/// assert_eq!( // Sorter bulk string: shorter by 1-byte (capital A acute is 2-bytes)
///     "$3\r\nÂ\r\n".try_into() as ValueResult,
///     Err(VError::Size {node: BULK_STRING, index: 7}));
///
/// // JSON: null
/// assert_eq!(
///     Value::try_from("$-1\r\n"),
///     Ok(Nil)
/// );
///
/// // JSON: 10
/// assert_eq!(
///     Value::try_from(":10\r\n"),
///     Ok(Integer(10))
/// );
///
/// // JSON: "Nina Simone"
/// assert_eq!(
///     Value::try_from("+Nina Simone\r\n"),
///     Ok(String("Nina Simone".into()))
/// );
///
/// // JSON: "Lorem ipsum...\r\nDolor sit amet..."
/// assert_eq!(
///     Value::try_from("$33\r\nLorem ipsum...\r\nDolor sit amet...\r\n"),
///     Ok(String("Lorem ipsum...\r\nDolor sit amet...".into()))
/// );
///
/// // JavaScript: [null, 447, new Error("Oh oh!"), "Hourly", "Si vis pacem,\r\npara bellum"]
/// assert_eq!(
///     Value::try_from("*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n"),
///     Ok(Array(vec![
///         Nil,
///         Integer(447),
///         Error("Oh oh!".into()),
///         String("Hourly".into()),
///         String("Si vis pacem,\r\npara bellum".into())
///     ]))
/// );
///
/// // NOTE: Even recursive arrays - we leave that for you to try out.
/// ```
#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
struct Input<'a> {
    /// String range to be processed.
    source: &'a str,
    /// Bytes count of this range first [`char`], in the original [`&str`].
    position: usize,
}

type InnerResult<'a> = (ValueResult<'a>, usize);

/// Just a type alias
pub type ValueResult<'a> = Result<Value, <Value as TryFrom<&'a str>>::Error>;

impl TryFrom<&str> for Value {
    type Error = TError;

    fn try_from(source: &str) -> ValueResult {
        Value::internal_try_from(Input {
            position: 0,
            source,
        })
        .0
    }
}

impl Value {
    fn internal_try_from(input: Input) -> InnerResult {
        match input.source.chars().next() {
            Some('*') => Value::extract_array(input),
            Some('-') => Value::extract_error(input),
            Some(':') => Value::extract_integer(input),
            Some('$') => Value::extract_bulk_string(input),
            Some('+') => Value::extract_simple_string(input),
            _ => (Err(TError::of_unexpected(UNKNOWN, input.position)), 0),
        }
    }

    fn extract_array(input: Input) -> InnerResult {
        let integer_input = Input {
            position: input.position,
            ..input
        };
        match Value::extract_integer(integer_input) {
            (Ok(Value::Integer(len)), size) => {
                let mut values = vec![];
                let len = len as usize;
                let mut offset = size;

                while values.len() < len {
                    let next_input = Input {
                        position: input.position + offset,
                        source: &input.source[offset..input.source.len()],
                    };

                    if "" == next_input.source {
                        return (Err(TError::of_size(ARRAY, offset)), offset);
                    }

                    match Value::internal_try_from(next_input) {
                        (Ok(value), size) => {
                            values.push(value);
                            offset += size;
                        }
                        r#else => return r#else,
                    }
                }

                if len == values.len() {
                    (Ok(Value::Array(values)), offset)
                } else {
                    (Err(TError::of_size(ARRAY, offset + 1)), offset + 1)
                }
            }
            r#else => return r#else,
        }
    }

    fn extract_error(input: Input) -> InnerResult {
        match Value::extract_simple_string(input) {
            (Ok(Value::String(message)), size) => (Ok(Value::Error(message)), size),
            r#else => r#else,
        }
    }

    fn extract_integer(input: Input) -> InnerResult {
        // TODO: Support negative numbers
        let node = match &input.source[0..1] {
            ":" => INTEGER,
            _ => SIZE,
        };
        let position = input.position + 1;

        if let Some(i) = input.source.find("\r\n") {
            return match input.source[1..i].parse::<i64>().ok() {
                Some(value) => (Ok(Value::Integer(value)), i + 2),
                _ => (Err(TError::of_type(node, position)), position),
            };
        }

        (
            Err(TError::of_unexpected(node, input.source.len())),
            input.source.len(),
        )
    }

    fn extract_bulk_string(input: Input) -> InnerResult {
        if input.source.starts_with("$-1\r\n") {
            return (Ok(Value::Nil), 5);
        }

        match Self::extract_integer(Input { ..input }) {
            (Ok(Value::Integer(size)), _) => {
                let start = 1 + size.to_string().len() + 2;
                let end = start + size as usize;

                return if input.source[end..input.source.len()].starts_with("\r\n") {
                    (
                        Ok(Value::String(input.source[start..end].to_string())),
                        end + 2,
                    )
                } else if end < input.source.len() {
                    let position = input.position + end;
                    (Err(TError::of_size(BULK_STRING, position)), position)
                } else {
                    let position = input.position + end + 1;
                    (Err(TError::of_size(BULK_STRING, position)), position)
                };
            }
            (Err(error), size) => (Err(error), size),
            _ => (
                Err(TError::of_unexpected(BULK_STRING, input.position + 1)),
                input.position + 1,
            ),
        }
    }

    fn extract_simple_string(input: Input) -> InnerResult {
        let node = match &input.source[0..1] {
            "+" => SIMPLE_STRING,
            _ => ERROR,
        };
        let mut position = input.position + 1;

        if let Some(i) = input.source.find("\r\n") {
            // @formatter::off
            match input.source.find('\r').filter(|&p| p < i)
                .or_else(|| input.source.find('\n').filter(|&p| p < i))
            // @formatter::on
            {
                Some(shift) => position = input.position + shift,
                _ => return (Ok(Value::String(input.source[1..i].into())), i + 2),
            }
        }

        (Err(TError::of_unexpected(node, position)), position)
    }
}

#[cfg(test)]
mod tests {
    use crate::Node::{ARRAY, BULK_STRING, INTEGER, SIMPLE_STRING, SIZE};

    use super::super::{Error, Value};

    #[test]
    fn value_implement_try_from_resp_nil() {
        assert_eq!("$-1\r\n".try_into(), Ok(Value::Nil));
    }

    #[test]
    fn value_implement_try_from() {
        let _value: Result<Value, Error> = "".try_into();
    }

    #[test]
    fn value_implement_try_from_resp_array() {
        assert_eq!("*0\r\n".try_into(), Ok(Value::Array(vec![])));
        assert_eq!(
            "*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n"
                .try_into(),
            Ok(Value::Array(vec![
                Value::Nil,
                Value::Integer(447),
                Value::Error("Oh oh!".into()),
                Value::String("Hourly".into()),
                Value::String("Si vis pacem,\r\npara bellum".into()),
            ]))
        );
    }

    #[test]
    fn value_implement_try_from_resp_nested_array() {
        let got = "*2\r\n*3\r\n+A\r\n+B\r\n+C\r\n*3\r\n:1\r\n:2\r\n:3\r\n".try_into()
            as Result<Value, Error>;
        let expected = Ok(Value::Array(vec![
            Value::Array(vec![
                Value::String("A".into()),
                Value::String("B".into()),
                Value::String("C".into()),
            ]),
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        ])) as Result<Value, Error>;

        assert_eq!(got, expected);
    }

    #[test]
    fn value_implement_try_from_resp_array_with_mismatching_size() {
        assert_eq!(
            "*2\r\n$-1\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_size(ARRAY, 9))
        );
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
        assert_eq!(
            ":Yikes\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_type(INTEGER, 1))
        );
        assert_eq!(
            ":0".try_into() as Result<Value, Error>,
            Err(Error::of_unexpected(INTEGER, 2))
        );
    }

    #[test]
    fn value_implement_try_from_resp_bulk_string() {
        assert_eq!(
            "$4\r\nOops\r\n".try_into(),
            Ok(Value::String("Oops".into()))
        );
        assert_eq!(
            "$7\r\nOh\r\nOh!\r\n".try_into(),
            Ok(Value::String("Oh\r\nOh!".into()))
        );
    }

    #[test]
    fn value_implement_try_from_resp_bulk_string_with_mismatching_len() {
        assert_eq!(
            "$5\r\nOops\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_size(BULK_STRING, 9))
        );
        assert_eq!(
            "$3\r\nOops\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_size(BULK_STRING, 7))
        );
    }

    #[test]
    fn value_implement_try_from_resp_simple_string() {
        assert_eq!(
            "+Anatomy\r\n".try_into(),
            Ok(Value::String("Anatomy".into()))
        );
    }

    #[test]
    fn value_implement_try_from_resp_simple_string_with_line_feed_or_carriage_return_in_value() {
        assert_eq!(
            "+Top\nBottom\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_unexpected(SIMPLE_STRING, 4))
        );
        assert_eq!(
            "+Top\rBottom\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_unexpected(SIMPLE_STRING, 4))
        );
    }

    #[test]
    fn value_implement_try_from_resp_with_invalid_size_type() {
        assert_eq!(
            "*!\r\n$-1\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_type(SIZE, 1))
        );
        assert_eq!(
            "$!\r\n$-1\r\n".try_into() as Result<Value, Error>,
            Err(Error::of_type(SIZE, 1))
        );
    }
}
