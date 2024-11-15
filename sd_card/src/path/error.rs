use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum PathParseError {
    EmptyPath,
    PathShouldBeAbsolute,
    PathShouldEndWithFilename,
}

impl Display for PathParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PathParseError {}
