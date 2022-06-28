#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Denote a size mismatch data in the RESP string.
    ///
    /// This means that observed [`super::Value::String`] length or
    /// [`super::Value::Array`] size is fewer than the expected size.
    ///
    /// + For bulk [`super::Value::String`], this can be checked by looking
    ///   at `\r\n` sequence at the end of the byte sequence;
    /// + For bulk [`super::Value::Array`], this means we parsed all the string
    ///   but the entries count of the array is fewer than expected.
    ///
    /// The `index` indicates:
    /// + For bulk [`super::Value::String`], that `\r\n` sequence was expected
    ///   at that index _(which is right after the specified bulk string size.
    ///   )_
    /// + For [`super::Value::Array`], that some byte were still expected at
    ///   that position.
    ///
    /// The `node` indicates which token was being processed when the type
    /// mismatch was noticed.
    Size { index: usize, node: Node },

    /// Denote an inconvertible data in the RESP string.
    ///
    /// The `index` indicates at which byte it happened. _(So far, the only
    /// RESP token that need type conversion is [`super::Value::Integer`].
    /// So this error will mean that the integer/size value was not parsable
    /// into [`i64`].)_
    ///
    /// The `node` indicates which token was being processed when the type
    /// mismatch was noticed.
    Type { index: usize, node: Node },

    /// Denote an unexpected byte in the RESP string.
    ///
    /// The `index` indicate at which byte it happened. _(Should its value be
    /// larger than the parser string, it will indicate that some bytes where
    /// expected there: usually, the `\r\n` sequence.)_
    ///
    /// The `node` indicates which token was being processed when the
    /// unexpected happened.
    Unexpected { index: usize, node: Node },
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Node {
    SIMPLE_STRING,
    BULK_STRING,
    UNKNOWN,
    INTEGER,
    ERROR,
    ARRAY,
    SIZE,
    NIL,
}

impl Error {
    pub fn index(&self) -> &usize {
        match &self {
            Self::Type { index, .. } => index,
            Self::Size { index, .. } => index,
            Self::Unexpected { index, .. } => index,
        }
    }

    pub fn node(&self) -> &Node {
        match &self {
            Self::Type { node, .. } => node,
            Self::Size { node, .. } => node,
            Self::Unexpected { node, .. } => node,
        }
    }

    pub fn of_size(node: Node, index: usize) -> Error {
        Error::Size { index, node }
    }

    pub fn of_type(node: Node, index: usize) -> Error {
        Error::Type { index, node }
    }

    pub fn of_unexpected(node: Node, index: usize) -> Error {
        Error::Unexpected { index, node }
    }
}
