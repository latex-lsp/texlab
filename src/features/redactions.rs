use insta::internals::{Content, ContentPath};
use lsp_types::Url;

pub fn redact_uri(content: Content, _content_path: ContentPath) -> String {
    content.as_str().unwrap().replace(
        Url::from_directory_path(std::env::temp_dir())
            .unwrap()
            .as_str(),
        "[tmp]/",
    )
}
