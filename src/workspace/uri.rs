use lsp_types::*;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::Path;

#[derive(Debug, Clone, Eq)]
pub struct Uri(Url);

impl Uri {
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
