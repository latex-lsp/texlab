use std::path::PathBuf;

use insta::internals::{Content, ContentPath};
use lsp_types::Url;

pub fn redact_uri(directory: PathBuf) -> impl Fn(Content, ContentPath) -> String {
    move |content, _content_path| {
        content.as_str().unwrap().replace(
            Url::from_directory_path(&directory).unwrap().as_str(),
            "[tmp]/",
        )
    }
}
