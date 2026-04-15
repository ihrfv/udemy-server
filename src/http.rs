#![allow(dead_code)]
#![allow(unused_imports)]

pub use method::Method;
pub use request::{ParseError, Request};

mod method;
mod query_string;
mod request;
