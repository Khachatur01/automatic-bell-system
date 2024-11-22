mod error;

use crate::path::error::PathParseError;
use crate::path::error::PathParseError::{EmptyPath, PathShouldBeAbsolute, PathShouldEndWithFilename};

pub struct Path {
    pub(crate) directories_path: String,
    pub(crate) filename: String,
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

        if raw_path.ends_with("/") {
            return Err(PathShouldEndWithFilename);
        }

        if let Some((directories_path, filename)) = raw_path.rsplit_once("/") {
            Ok(Self {
                directories_path: directories_path.to_string(),
                filename: filename.to_string()
            })
        } else {
            Err(EmptyPath)
        }
    }
}
