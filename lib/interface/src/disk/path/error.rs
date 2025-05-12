use std::fmt::{Debug};

#[derive(Debug)]
pub enum PathParseError {
    EmptyPath,
    PathShouldBeAbsolute,
    PathShouldEndWithFilename,
}
