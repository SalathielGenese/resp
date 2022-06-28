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
returns a Rust `Result<Value, String>`.

Whilst the error is a simple string for now, it will evolve into its own
enum, which will be more descriptive of the reason behind the validation
error(s) encountered.

## Usage

Add dependency to your project:
```editorconfig
; Cargo.toml
[dependecies]
squall_dot_io_resp = "0.1.2"
```

Here are example with code:
```rust
use crate::squall_dot_io_resp::Value;

// JSON: null
with_resp("$-1\r\n".try_into());

// JSON: 10
with_resp(":10\r\n".try_into());

// JSON: "Nina Simone"
with_resp("+Nina Simone\r\n".try_into());

// JSON: "Lorem ipsum...\r\nDolor sit amet..."
with_resp("$33\r\nLorem ipsum...\r\nDolor sit amet...\r\n".try_into());

// JavaScript: [null, 447, new Error("Oh oh!"), "Hourly", "Si vis pacem,\r\npara bellum"]
with_resp("*5\r\n$-1\r\n:447\r\n-Oh oh!\r\n+Hourly\r\n$26\r\nSi vis pacem,\r\npara bellum\r\n"
          .try_into());

// NOTE: Even recursive arrays - we leave that for you to try out.

fn with_resp(input: Result<Value, String>) {
    println!("{:?}", input);
}
```

## License

MIT

## Contributions

Should you find some [issues], please report on GitHub
project, or consider opening a [pull-request].

[RESP specs]: <https://redis.io/docs/reference/protocol-spec/>
[issues]: <https://github.com/SalathielGenese/resp/issues/>
[pull-request]: <https://github.com/SalathielGenese/resp/compare/>
