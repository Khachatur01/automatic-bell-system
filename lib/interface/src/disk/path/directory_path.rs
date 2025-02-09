use std::fmt::{Display, Formatter};
use crate::PathParseError;
use crate::PathParseError::{EmptyPath, PathShouldBeAbsolute};

pub struct DirectoryPath {
    pub directories_path: Vec<String>,
}

impl TryFrom<&str> for DirectoryPath {
    type Error = PathParseError;

    fn try_from(raw_path: &str) -> Result<Self, Self::Error> {
        if raw_path.is_empty() {
            return Err(EmptyPath);
        }

        if !raw_path.starts_with("/") {
            return Err(PathShouldBeAbsolute);
        }

        Ok(Self {
            directories_path: raw_path
                .split("/")
                .map(str::to_string)
                .collect()
        })
    }
}

impl From<&[&str]> for DirectoryPath {
    fn from(directories_list: &[&str]) -> Self {
        Self {
            directories_path: directories_list.iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

impl Display for DirectoryPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.directories_path.join("/"))
    }
}
