use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
};

use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[derive(Eq, Clone, Serialize, Deserialize)]
pub struct Uri(Url);

impl Uri {
    pub fn with_extension(&self, extension: &str) -> Option<Self> {
        let file_name = self.path_segments()?.last()?;
        let file_stem = match file_name.rfind('.') {
            Some(index) => &file_name[..index],
            None => file_name,
        };
        self.join(&format!("{}.{}", file_stem, extension))
            .ok()
            .map(Into::into)
    }

    pub fn parse(input: &str) -> Result<Self, ParseError> {
        Url::parse(input).map(|url| url.into())
    }

    #[allow(clippy::result_unit_err)]
    pub fn from_directory_path<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        Url::from_directory_path(path).map(|url| url.into())
    }

    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        Url::from_file_path(path).map(|url| url.into())
    }
}

impl PartialEq for Uri {
    fn eq(&self, other: &Self) -> bool {
        if cfg!(windows) {
            self.as_str().to_lowercase() == other.as_str().to_lowercase()
        } else {
            self.as_str() == other.as_str()
        }
    }
}

impl Hash for Uri {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().to_lowercase().hash(state);
    }
}

impl Deref for Uri {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Url> for Uri {
    fn from(url: Url) -> Self {
        Uri(url)
    }
}

impl From<Uri> for Url {
    fn from(uri: Uri) -> Self {
        uri.0
    }
}

impl fmt::Debug for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
