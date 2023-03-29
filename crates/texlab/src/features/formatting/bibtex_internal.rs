use lsp_types::{FormattingOptions, TextEdit};
use rowan::{ast::AstNode, NodeOrToken};
use syntax::bibtex::{self, HasName, HasType, HasValue};

use crate::{
    db::Document,
    util::{line_index::LineIndex, line_index_ext::LineIndexExt},
    Db,
};

pub fn format_bibtex_internal(
    db: &dyn Db,
    document: Document,
    options: &FormattingOptions,
) -> Option<Vec<TextEdit>> {
    let mut indent = String::new();

    if options.insert_spaces {
        for _ in 0..options.tab_size {
            indent.push(' ');
        }
    } else {
        indent.push('\t');
    }

    let line_length = db.config().formatting.line_length;

    let line_index = document.line_index(db);
    let data = document.parse(db).as_bib()?;
    let mut edits = Vec::new();

    for node in data.root(db).children().filter(|node| {
        matches!(
            node.kind(),
            bibtex::PREAMBLE | bibtex::STRING | bibtex::ENTRY
        )
    }) {
        let range = node.text_range();

        let mut formatter =
            Formatter::new(indent.clone(), options.tab_size, line_length, line_index);

        formatter.visit_node(node);
        edits.push(TextEdit {
            range: line_index.line_col_lsp_range(range),
            new_text: formatter.output,
        });
    }

    Some(edits)
}

struct Formatter<'a> {
    indent: String,
    tab_size: u32,
    line_length: usize,
    output: String,
    align: Vec<usize>,
    line_index: &'a LineIndex,
}

impl<'a> Formatter<'a> {
    fn new(indent: String, tab_size: u32, line_length: usize, line_index: &'a LineIndex) -> Self {
        Self {
            indent,
            tab_size,
            line_length,
            output: String::new(),
            align: Vec::new(),
            line_index,
        }
    }

    fn visit_token_lowercase(&mut self, token: &bibtex::SyntaxToken) {
        self.output.push_str(&token.text().to_lowercase());
    }

    fn should_insert_space(
        &self,
        previous: &bibtex::SyntaxToken,
        current: &bibtex::SyntaxToken,
    ) -> bool {
        let previous_range = self.line_index.line_col_lsp_range(previous.text_range());
        let current_range = self.line_index.line_col_lsp_range(current.text_range());
        previous_range.start.line != current_range.start.line
            || previous_range.end.character < current_range.start.character
    }

    fn base_align(&self) -> usize {
        self.output[self.output.rfind('\n').unwrap_or(0)..]
            .chars()
            .count()
    }

    fn visit_node(&mut self, parent: bibtex::SyntaxNode) {
        match parent.kind() {
            bibtex::PREAMBLE => {
                let preamble = bibtex::Preamble::cast(parent).unwrap();
                self.visit_token_lowercase(&preamble.type_token().unwrap());
                self.output.push('{');
                if preamble.syntax().children().next().is_some() {
                    self.align.push(self.base_align());
                    for node in preamble.syntax().children() {
                        self.visit_node(node);
                    }
                    self.output.push('}');
                }
            }
            bibtex::STRING => {
                let string = bibtex::StringDef::cast(parent).unwrap();
                self.visit_token_lowercase(&string.type_token().unwrap());
                self.output.push('{');
                if let Some(name) = string.name_token() {
                    self.output.push_str(name.text());
                    self.output.push_str(" = ");
                    if let Some(value) = string.value() {
                        self.align.push(self.base_align());
                        self.visit_node(value.syntax().clone());
                        self.output.push('}');
                    }
                }
            }
            bibtex::ENTRY => {
                let entry = bibtex::Entry::cast(parent).unwrap();
                self.visit_token_lowercase(&entry.type_token().unwrap());
                self.output.push('{');
                if let Some(key) = entry.name_token() {
                    self.output.push_str(&key.to_string());
                    self.output.push(',');
                    self.output.push('\n');
                    for field in entry.fields() {
                        self.visit_node(field.syntax().clone());
                    }
                    self.output.push('}');
                }
            }
            bibtex::FIELD => {
                let field = bibtex::Field::cast(parent).unwrap();
                self.output.push_str(&self.indent);
                let name = field.name_token().unwrap();
                self.output.push_str(name.text());
                self.output.push_str(" = ");
                if let Some(value) = field.value() {
                    let count = name.text().chars().count();
                    self.align.push(self.tab_size as usize + count + 3);
                    self.visit_node(value.syntax().clone());
                    self.output.push(',');
                    self.output.push('\n');
                }
            }
            kind if bibtex::Value::can_cast(kind) => {
                let tokens: Vec<_> = parent
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .filter(|token| token.kind() != bibtex::WHITESPACE)
                    .collect();

                self.output.push_str(tokens[0].text());

                let align = self.align.pop().unwrap_or_default();
                let mut length = align + tokens[0].text().chars().count();
                for i in 1..tokens.len() {
                    let previous = &tokens[i - 1];
                    let current = &tokens[i];
                    let current_length = current.text().chars().count();

                    let insert_space = self.should_insert_space(previous, current);
                    let space_length = if insert_space { 1 } else { 0 };

                    if length + current_length + space_length > self.line_length {
                        self.output.push('\n');
                        self.output.push_str(self.indent.as_ref());
                        for _ in 0..=align - self.tab_size as usize {
                            self.output.push(' ');
                        }
                        length = align;
                    } else if insert_space {
                        self.output.push(' ');
                        length += 1;
                    }
                    self.output.push_str(current.text());
                    length += current_length;
                }
            }
            bibtex::ROOT | bibtex::JUNK => {
                for element in parent.children_with_tokens() {
                    match element {
                        NodeOrToken::Token(token) => {
                            self.output.push_str(token.text());
                        }
                        NodeOrToken::Node(node) => {
                            self.visit_node(node);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
