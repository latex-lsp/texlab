pub mod capabilities;
pub mod chktex;
pub mod diagnostics;
pub mod line_index_ext;
pub mod lsp_enums;

use std::path::PathBuf;

use lsp_types::Url;
use once_cell::sync::Lazy;

pub static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(dirs::home_dir);

pub fn normalize_uri(uri: &mut Url) {
    if let Some(mut segments) = uri.path_segments() {
        if let Some(mut path) = segments.next().and_then(fix_drive_letter) {
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_fragment(None);
}

fn fix_drive_letter(text: &str) -> Option<String> {
    if !text.is_ascii() {
        return None;
    }

    match &text[1..] {
        ":" => Some(text.to_ascii_uppercase()),
        "%3A" | "%3a" => Some(format!("{}:", text[0..1].to_ascii_uppercase())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::Url;

    use super::normalize_uri;

    #[test]
    fn test_lowercase_drive_letter() {
        let mut uri = Url::parse("file://c:/foo/bar.txt").unwrap();
        normalize_uri(&mut uri);
        assert_eq!(uri.as_str(), "file:///C:/foo/bar.txt");
    }

    #[test]
    fn test_uppercase_drive_letter() {
        let mut uri = Url::parse("file://C:/foo/bar.txt").unwrap();
        normalize_uri(&mut uri);
        assert_eq!(uri.as_str(), "file:///C:/foo/bar.txt");
    }

    #[test]
    fn test_fragment() {
        let mut uri = Url::parse("foo:///bar/baz.txt#qux").unwrap();
        normalize_uri(&mut uri);
        assert_eq!(uri.as_str(), "foo:///bar/baz.txt");
    }
}
