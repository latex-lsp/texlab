use std::path::PathBuf;

use base_db::{util::LineCol, Owner, Workspace};
use rowan::{TextRange, TextSize};
use url::Url;

#[derive(Debug)]
pub struct Fixture {
    pub workspace: Workspace,
    pub documents: Vec<DocumentSpec>,
}

impl Fixture {
    pub fn parse(input: &str) -> Fixture {
        let mut documents = Vec::new();

        let mut start = 0;
        for end in input
            .match_indices("%!")
            .skip(1)
            .map(|(i, _)| i)
            .chain(std::iter::once(input.len()))
        {
            documents.push(DocumentSpec::parse(&input[start..end]));
            start = end;
        }

        let mut workspace = Workspace::default();
        for document in &documents {
            let path = PathBuf::from(document.uri.path());
            let language = distro::Language::from_path(&path).unwrap_or(distro::Language::Tex);

            workspace.open(
                document.uri.clone(),
                document.text.clone(),
                language,
                Owner::Client,
                LineCol { line: 0, col: 0 },
            );
        }

        Self {
            workspace,
            documents,
        }
    }
}

#[derive(Debug)]
pub struct DocumentSpec {
    pub uri: Url,
    pub text: String,
    pub cursor: Option<TextSize>,
    pub ranges: Vec<TextRange>,
}

impl DocumentSpec {
    pub fn parse(input: &str) -> Self {
        let (uri_str, input) = input
            .trim()
            .strip_prefix("%! ")
            .map(|input| input.split_once('\n').unwrap_or((input, "")))
            .unwrap();

        let uri = Url::parse(&format!("file:///texlab/{uri_str}")).unwrap();

        let mut ranges = Vec::new();
        let mut cursor = None;

        let mut text = String::new();
        for line in input.lines().map(|line| line.trim_end()) {
            if line.chars().all(|c| matches!(c, ' ' | '^' | '|' | '!')) && !line.is_empty() {
                cursor = cursor.or_else(|| {
                    let offset = line.find('|')?;
                    Some(TextSize::from((text.len() + offset) as u32))
                });

                if let Some(start) = line.find('!') {
                    let position = TextSize::from((text.len() + start) as u32);
                    ranges.push(TextRange::new(position, position));
                }

                if let Some(start) = line.find('^') {
                    let end = line.rfind('^').unwrap() + 1;
                    ranges.push(TextRange::new(
                        TextSize::from((text.len() + start) as u32),
                        TextSize::from((text.len() + end) as u32),
                    ));
                }
            } else {
                text.push_str(line);
                text.push('\n');
            }
        }

        Self {
            uri,
            text,
            cursor,
            ranges,
        }
    }
}
