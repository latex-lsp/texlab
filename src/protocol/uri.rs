use lsp_types::{TextDocumentIdentifier, TextDocumentPositionParams};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
};
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

impl Into<Url> for Uri {
    fn into(self) -> Url {
        self.0
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

pub trait AsUri {
    fn as_uri(&self) -> Uri;
}

impl AsUri for TextDocumentIdentifier {
    fn as_uri(&self) -> Uri {
        self.uri.clone().into()
    }
}

impl AsUri for TextDocumentPositionParams {
    fn as_uri(&self) -> Uri {
        self.text_document.as_uri()
    }
}
