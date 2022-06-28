//! # RESP parser and validator.
//!
//! A RESP (REdis Serialization Protocol) parser implementation,
//! written with edge performance in mind.
//!
//! If you are not familiar with RESP, consider starting here with
//! RESP specs[^resp_spec_link]. RESP is a binary safe serialization
//! protocol. Initially developed for the ReDiS project, it injection
//! safe (needs no escaping) and is fast forward as it requires no
//! look-back in parsing.
//!
//! This crate aims to parse and validate your RESP strings. Since the
//! protocol can be used beyond its initial scope, to a general-purpose
//! communication scheme.
//!
//! To do so, its reuses Rust [`TryInto`] trait to try and parse your [`&str`]
//! as a valid RESP. Implemented on a [`Value`] enum of RESP tokens, it
//! returns a Rust [`Result<Value, Error>`].
//!
//! Should you find some issues[^issues_link], please report on GitHub
//! project, or consider opening a pull-request[^pull_request_link].
//!
//! [^issues_link]: <https://github.com/SalathielGenese/resp/issues/>
//!
//! [^resp_spec_link]: <https://redis.io/docs/reference/protocol-spec/>
//!
//! [^pull_request_link]: <https://github.com/SalathielGenese/resp/compare/>

pub use error::Node;
pub use error::Error;
pub use value::Value;

pub mod value;
pub mod error;
