mod error;

use std::str::Split;
use crate::path::error::PathParseError;
use crate::path::error::PathParseError::{EmptyPath, PathShouldBeAbsolute};

pub struct Path {
    pub(crate) absolute_directories: Vec<String>,
    pub(crate)filename: String,
}

impl TryFrom<String> for Path {
    type Error = PathParseError;

    fn try_from(raw_path: String) -> Result<Self, Self::Error> {
        if raw_path.is_empty() {
            return Err(EmptyPath);
        }

        if !raw_path.starts_with("/") {
            return Err(PathShouldBeAbsolute);
        }

        let parts: Split<&str> = raw_path.split("/");

        /* Skip first empty string. As path is absolute, first element of string will be empty string. */
        let path: Vec<String> = parts.skip(1).collect();
        

        todo!()
    }
}
