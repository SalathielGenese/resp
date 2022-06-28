# RESP parser and validator

A RESP (REdis Serialization Protocol) parser implementation,
written with edge performance in mind.

If you are not familiar with RESP, consider starting here with
[RESP specs]. RESP is a binary safe serialization
protocol. Initially developed for the ReDiS project, it is injection
safe (needs no escaping) and is fast-forward as it requires no
look-back in parsing.

This crate aims to parse and validate your RESP strings. Since the
protocol can be used beyond its initial scope, to a general-purpose
communication scheme.

To do so, its reuses Rust `TryInto` trait to try and parse your `&str`
as a valid RESP. Implemented on a `Value` enum of RESP tokens, it
returns a Rust `Result<Value, Error>`.

## Usage

Add dependency to your project:
```editorconfig
; Cargo.toml
[dependecies]
squall_dot_io_resp = "0.1.2"
```

Here are example with code:
```rust
use squall_dot_io_resp::{
    Node::{self, NIL, SIZE, ARRAY, ERROR, INTEGER, UNKNOWN, SIMPLE_STRING, BULK_STRING},
    Value::{self, Nil, Error, Array, String, Integer},
    Error as VError,
    ValueResult,
};

assert_eq!( // Empty RESP
    "".try_into() as ValueResult,
    Err(VError::Unexpected {node: UNKNOWN, index: 0}));

assert_eq!( // Unterminated number: missing "\r\n"
    ":0".try_into() as ValueResult,
    Err(VError::Unexpected {node: INTEGER, index: 2}));

assert_eq!( // Not enough elements in the array
    "*2\r\n$-1\r\n".try_into() as ValueResult,
    Err(VError::Size {node: ARRAY, index: 9}));

assert_eq!( // Longer bulk string: got more that 2-bytes
    "$2\r\nHello\r\n".try_into() as ValueResult,
    Err(VError::Size {node: BULK_STRING, index: 6}));

assert_eq!( // Sorter bulk string: shorter by 1-byte (capital A acute is 2-bytes)
    "$3\r\nÂ\r\n".try_into() as ValueResult,
    Err(VError::Size {node: BULK_STRING, index: 7}));
```

```rust
use squall_dot_io_resp::{
    Node::{self, NIL, SIZE, ARRAY, ERROR, INTEGER, UNKNOWN, SIMPLE_STRING, BULK_STRING},
    Value::{self, Nil, Error, Array, String, Integer},
    Error as VError,
    ValueResult,
};
// JSON: null
assert_eq!(
    Value::try_from("$-1\r\n"),
    Ok(Nil)
);

// JSON: 10
assert_eq!(
    Value::try_from(":10\r\n"),
    Ok(Integer(10))
);

// JSON: "Nina Simone"
assert_eq!(
    Value::try_from("+Nina Simone\r\n"),
    Ok(String("Nina Simone".into()))
);

// JSON: "Lorem ipsum...\r\nDolor sit amet..."
assert_eq!(
    Value::try_from("$33\r\nLorem ipsum...\r\nDolor sit amet...\r\n"),
    Ok(String("Lorem ipsum...\r\nDolor sit amet...".into()))
);

// JavaScript: [null, 447, new Error("Oh oh!"), "Hourly", "Si vis pacem,\r\npara bellum"]
assert_eq!(
    Value::try_from("*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n"),
    Ok(Array(vec![
        Nil,
        Integer(447),
        Error("Oh oh!".into()),
        String("Hourly".into()),
        String("Si vis pacem,\r\npara bellum".into())
    ]))
);

// NOTE: Even recursive arrays - we leave that for you to try out.
```

## License

MIT

## Contributions

Should you find some [issues], please report on GitHub
project, or consider opening a [pull-request].

[RESP specs]: <https://redis.io/docs/reference/protocol-spec/>
[issues]: <https://github.com/SalathielGenese/resp/issues/>
[pull-request]: <https://github.com/SalathielGenese/resp/compare/>
