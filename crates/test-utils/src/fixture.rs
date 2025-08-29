use std::path::PathBuf;

use base_db::{DocumentLocation, FeatureParams, Owner, Workspace};
use line_index::{LineCol, LineIndex};
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

    pub fn make_params(&'_ self) -> Option<(FeatureParams<'_>, TextSize)> {
        let spec = self
            .documents
            .iter()
            .find(|spec| spec.cursor.is_some())
            .or_else(|| self.documents.first())?;

        let document = self.workspace.lookup(&spec.uri)?;
        let params = FeatureParams::new(&self.workspace, document);
        let cursor = spec.cursor.unwrap_or_default();
        Some((params, cursor))
    }

    pub fn locations(&'_ self) -> impl Iterator<Item = DocumentLocation<'_>> {
        self.documents.iter().flat_map(|spec| {
            let document = self.workspace.lookup(&spec.uri).unwrap();
            spec.ranges
                .iter()
                .map(|range| DocumentLocation::new(document, *range))
        })
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
        let mut line_nbr = 0;
        for line in input.lines().map(|line| line.trim_end()) {
            if line.chars().all(|c| matches!(c, ' ' | '^' | '|' | '!')) && !line.is_empty() {
                cursor = cursor.or_else(|| {
                    let offset = line.find('|')?;
                    Some(CharacterPosition::new(line_nbr, offset))
                });

                if let Some(start) = line.find('!') {
                    let position = CharacterPosition::new(line_nbr, start);
                    ranges.push(CharacterRange::new(position, position));
                }

                if let Some(start) = line.find('^') {
                    let end = line.rfind('^').unwrap() + 1;
                    let start = CharacterPosition::new(line_nbr, start);
                    let end = CharacterPosition::new(line_nbr, end);
                    ranges.push(CharacterRange::new(start, end));
                }
            } else {
                text.push_str(line);
                text.push('\n');
                line_nbr += 1;
            }
        }

        let line_index = LineIndex::new(&text);

        let cursor = cursor.and_then(|cursor| cursor.to_offset(&text, &line_index));
        let ranges = ranges
            .into_iter()
            .filter_map(|range| range.to_offset(&text, &line_index))
            .collect();

        Self {
            uri,
            text,
            cursor,
            ranges,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct CharacterPosition {
    line: usize,
    col: usize,
}

impl CharacterPosition {
    fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    fn to_offset(self, text: &str, line_index: &LineIndex) -> Option<TextSize> {
        let start = line_index.offset(LineCol {
            line: (self.line - 1) as u32,
            col: 0,
        })?;

        let slice = &text[start.into()..];
        let len = slice
            .char_indices()
            .nth(self.col)
            .map_or_else(|| slice.len(), |(i, _)| i);

        Some(start + TextSize::try_from(len).ok()?)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct CharacterRange {
    start: CharacterPosition,
    end: CharacterPosition,
}

impl CharacterRange {
    fn new(start: CharacterPosition, end: CharacterPosition) -> Self {
        Self { start, end }
    }

    fn to_offset(self, text: &str, line_index: &LineIndex) -> Option<TextRange> {
        let start = self.start.to_offset(text, line_index)?;
        let end = self.end.to_offset(text, line_index)?;
        Some(TextRange::new(start, end))
    }
}
