//! # Squall.IO RESP (REdis Serialization Protocol) parser
//!
//! A RESP parser implementation, written with edge performance in mind.
//!
//! If you are not familiar with RESP, consider starting here with
//! [RESP specs](https://redis.io/docs/reference/protocol-spec/).
//!
//! Should you find some issues, please report on GitHub project, or consider opening a pull-request.

pub mod value;

pub use value::{Value};
