use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum PathParseError {
    EmptyPath,
    PathShouldBeAbsolute,
    PathShouldEndWithFilename,
}
