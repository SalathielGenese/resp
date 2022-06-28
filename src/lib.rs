//! # Squall.IO RESP (REdis Serialization Protocol) parser and validator.
//!
//! A RESP parser implementation, written with edge performance in mind.
//! If you are not familiar with RESP, consider starting here with
//! [RESP specs](https://redis.io/docs/reference/protocol-spec/).
//!
//! RESP is a binary safe serialization protocol. Initially developed for
//! the ReDiS project, it injection safe (needs no escaping) and is fast
//! forward as it requires no look-back in parsing.
//!
//! Should you find some issues, please report on GitHub project, or consider opening a pull-request.

pub mod value;

pub use value::{Value};
