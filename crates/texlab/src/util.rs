mod client_flags;
pub mod from_proto;
pub mod line_index_ext;
pub mod lsp_enums;
pub mod to_proto;

pub use self::client_flags::ClientFlags;

pub fn normalize_uri(uri: &mut lsp_types::Url) {
    if let Some(mut segments) = uri.path_segments() {
        if let Some(mut path) = segments.next().and_then(fix_drive_letter) {
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_path(&normalize_percent_encoding(uri.path()));
    uri.set_fragment(None);
}

fn normalize_percent_encoding(input: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    for c in input.chars() {
        if c == '%' && i == 0 {
            output.push(c);
            i = 2;
        } else if i > 0 {
            output.push(c.to_ascii_uppercase());
            i -= 1;
        } else {
            output.push(c);
        }
    }

    output
}

fn fix_drive_letter(text: &str) -> Option<String> {
    if !text.is_ascii() || text.is_empty() {
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

    #[test]
    #[cfg(unix)]
    fn test_lowercase_percent_encoding_unix() {
        let mut uri1 = Url::parse("file:///foo/%c3%a4.tex").unwrap();
        let uri2 = Url::from_file_path("/foo/ä.tex").unwrap();

        normalize_uri(&mut uri1);
        assert_eq!(uri1, uri2);
    }

    #[test]
    #[cfg(unix)]
    fn test_uppercase_percent_encoding_unix() {
        let mut uri1 = Url::parse("file:///foo/%C3%A4.tex").unwrap();
        let uri2 = Url::from_file_path("/foo/ä.tex").unwrap();

        normalize_uri(&mut uri1);
        assert_eq!(uri1, uri2);
    }

    #[test]
    #[cfg(windows)]
    fn test_lowercase_percent_encoding_windows() {
        let mut uri1 = Url::parse("file:///c%3a/foo/%c3%a4.tex").unwrap();
        let uri2 = Url::from_file_path("C:/foo/ä.tex").unwrap();

        normalize_uri(&mut uri1);
        assert_eq!(uri1, uri2);
    }

    #[test]
    #[cfg(windows)]
    fn test_uppercase_percent_encoding_windows() {
        let mut uri1 = Url::parse("file:///c%3A/foo/%C3%A4.tex").unwrap();
        let uri2 = Url::from_file_path("C:/foo/ä.tex").unwrap();

        normalize_uri(&mut uri1);
        assert_eq!(uri1, uri2);
    }
}
