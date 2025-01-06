use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum PathParseError {
    EmptyPath,
    PathShouldBeAbsolute,
    PathShouldEndWithFilename,
}
