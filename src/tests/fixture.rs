use std::collections::BTreeMap;

use lsp_types::{Position, Range, TextDocumentIdentifier, TextDocumentPositionParams};
use rustc_hash::FxHashMap;

use super::client::Client;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum Line<'a> {
    File(&'a str, &'a str),
    Plain(&'a str),
    Range(u32, u32, std::ops::Range<usize>),
    Cursor(usize),
    Empty,
}

fn parse_line(line: &str) -> Line {
    if let Some(name) = line.strip_prefix("%TEX ") {
        Line::File(name, "latex")
    } else if let Some(name) = line.strip_prefix("%BIB ") {
        Line::File(name, "bibtex")
    } else if let Some(name) = line.strip_prefix("%LOG ") {
        Line::File(name, "log")
    } else if let Some(text) = line.strip_prefix("%SRC ") {
        Line::Plain(text)
    } else if let Some(text) = line.strip_prefix("%CUR ") {
        let position = text.find('^').unwrap();
        Line::Cursor(position)
    } else if line.is_empty() {
        Line::Empty
    } else {
        let key1 = line[1..2].parse().unwrap();
        let key2 = line[3..4].parse().unwrap();
        let line = &line[5..];
        let range = line
            .find('^')
            .map_or(0..0, |start| start..(line.rfind('^').unwrap() + 1));
        Line::Range(key1, key2, range)
    }
}

#[derive(Debug, Default)]
pub struct FileRange<'a> {
    pub name: &'a str,
    pub range: Range,
}

#[derive(Debug, Default)]
pub struct File<'a> {
    pub name: &'a str,
    pub lang: &'a str,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FileCursor<'a> {
    pub name: &'a str,
    pub position: Position,
}

impl<'a> FileCursor<'a> {
    pub fn into_params(self, server: &Client) -> TextDocumentPositionParams {
        let text_document = TextDocumentIdentifier::new(server.uri(self.name));
        TextDocumentPositionParams {
            text_document,
            position: self.position,
        }
    }
}

#[derive(Debug, Default)]
pub struct Fixture<'a> {
    pub files: Vec<File<'a>>,
    pub cursor: Option<FileCursor<'a>>,
    pub ranges: BTreeMap<u32, FxHashMap<u32, FileRange<'a>>>,
}

pub fn parse(input: &str) -> Fixture {
    let mut fixture = Fixture::default();
    let mut file = File::default();
    let mut file_line_index = 0;
    for line in input.lines().map(parse_line) {
        match line {
            Line::File(name, lang) => {
                if !file.name.is_empty() {
                    fixture.files.push(file);
                    file = File::default();
                }

                file.name = name;
                file.lang = lang;
                file_line_index = 0;
            }
            Line::Plain(line) => {
                file.text.push_str(line);
                file.text.push('\n');
                file_line_index += 1;
            }
            Line::Range(key1, key2, range) => {
                let line = file_line_index - 1;
                let start = Position::new(line, range.start.try_into().unwrap());
                let end = Position::new(line, range.end.try_into().unwrap());
                let range = Range::new(start, end);
                fixture.ranges.entry(key1).or_default().insert(
                    key2,
                    FileRange {
                        name: file.name,
                        range,
                    },
                );
            }
            Line::Cursor(position) => {
                fixture.cursor = Some(FileCursor {
                    name: file.name,
                    position: Position::new(file_line_index - 1, position.try_into().unwrap()),
                });
            }
            Line::Empty => {}
        };
    }

    fixture.files.push(file);
    fixture
}
