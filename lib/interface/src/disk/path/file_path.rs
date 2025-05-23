use crate::PathParseError;
use crate::PathParseError::{EmptyPath, PathShouldBeAbsolute, PathShouldEndWithFilename};

#[derive(Debug)]
pub struct FilePath {
    pub directories_path: Vec<String>,
    pub filename: String,
}

impl TryFrom<&str> for FilePath {
    type Error = PathParseError;

    fn try_from(raw_path: &str) -> Result<Self, Self::Error> {
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
                directories_path: directories_path
                    .split("/")
                    .map(str::to_string)
                    .collect(),
                filename: filename.to_string()
            })
        } else {
            Err(EmptyPath)
        }
    }
}

impl From<(&[&str], &str)> for FilePath {
    fn from(raw_path: (&[&str], &str)) -> Self {
        Self {
            directories_path: raw_path.0.iter()
                .map(ToString::to_string)
                .collect(),
            filename: raw_path.1.to_string()
        }
    }
}