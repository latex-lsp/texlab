use cancellation::CancellationToken;
use cstree::{NodeOrToken, TextLen, TextRange};
use lsp_types::{DocumentFormattingParams, TextEdit};

use crate::{
    features::FeatureRequest,
    syntax::{
        bibtex::{self, HasType},
        CstNode,
    },
    LineIndex, LineIndexExt,
};

const LINE_LENGTH: usize = 80;

pub fn format_bibtex_internal(
    request: &FeatureRequest<DocumentFormattingParams>,
    _cancellation_token: &CancellationToken,
) -> Option<Vec<TextEdit>> {
    let mut indent = String::new();
    if request.params.options.insert_spaces {
        for _ in 0..request.params.options.tab_size {
            indent.push(' ');
        }
    } else {
        indent.push('\t');
    }

    let document = request.main_document();
    let data = document.data.as_bibtex()?;

    let mut formatter = Formatter::new(
        indent,
        request.params.options.tab_size,
        &document.line_index,
    );

    formatter.visit_node(&data.root);

    Some(vec![TextEdit {
        range: document
            .line_index
            .line_col_lsp_range(TextRange::new(0.into(), document.text.text_len())),
        new_text: formatter.output,
    }])
}

struct Formatter<'a> {
    indent: String,
    tab_size: u32,
    output: String,
    align: Vec<usize>,
    line_index: &'a LineIndex,
}

impl<'a> Formatter<'a> {
    fn new(indent: String, tab_size: u32, line_index: &'a LineIndex) -> Self {
        Self {
            indent,
            tab_size,
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

    fn visit_node(&mut self, node: &bibtex::SyntaxNode) {
        match node.kind() {
            bibtex::PREAMBLE => {
                let preamble = bibtex::Preamble::cast(node).unwrap();
                self.visit_token_lowercase(preamble.ty().unwrap());
                self.output.push('{');
                if preamble.syntax().arity() > 0 {
                    self.align.push(self.output.chars().count());
                    for node in preamble.syntax().children() {
                        self.visit_node(node);
                    }
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            bibtex::STRING => {
                let string = bibtex::String::cast(node).unwrap();
                self.visit_token_lowercase(string.ty().unwrap());
                self.output.push('{');
                if let Some(name) = string.name() {
                    self.output.push_str(name.text());
                    self.output.push_str(" = ");
                    if string.syntax().arity() > 0 {
                        self.align.push(self.output.chars().count());
                        for node in string.syntax().children() {
                            self.visit_node(node);
                        }
                        self.output.push('}');
                    }
                }
                self.output.push('\n');
            }
            bibtex::ENTRY => {
                let entry = bibtex::Entry::cast(node).unwrap();
                self.visit_token_lowercase(entry.ty().unwrap());
                self.output.push('{');
                if let Some(key) = entry.key() {
                    self.output.push_str(key.text());
                    self.output.push(',');
                    self.output.push('\n');
                    for field in node.children() {
                        self.visit_node(field);
                    }
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            bibtex::FIELD => {
                let field = bibtex::Field::cast(node).unwrap();
                self.output.push_str(&self.indent);
                let name = field.name().unwrap();
                self.output.push_str(name.text());
                self.output.push_str(" = ");
                if let Some(value) = field.value() {
                    let count = name.text().chars().count();
                    self.align.push(self.tab_size as usize + count + 3);
                    self.visit_node(value.syntax());
                    self.output.push(',');
                    self.output.push('\n');
                }
            }
            bibtex::VALUE => {
                let tokens: Vec<_> = node
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .filter(|token| token.kind() != bibtex::WHITESPACE)
                    .collect();

                self.output.push_str(tokens[0].text());

                let align = self.align.pop().unwrap_or_default();
                let mut length = align + tokens[0].text().chars().count();
                for i in 1..tokens.len() {
                    let previous = tokens[i - 1];
                    let current = tokens[i];
                    let current_length = current.text().chars().count();

                    let insert_space = self.should_insert_space(previous, current);
                    let space_length = if insert_space { 1 } else { 0 };

                    if length + current_length + space_length > LINE_LENGTH {
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
            bibtex::ROOT | bibtex::JUNK | bibtex::COMMENT => {
                for element in node.children_with_tokens() {
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

#[cfg(test)]
mod tests {
    use insta::assert_display_snapshot;

    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_wrap_long_lines() {
        let request = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}",
            )])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_multiple_entries() {
        let request = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}\n\n@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}",
            )])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_trailing_comma() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, bar = baz}")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_insert_braces() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, bar = baz,")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_command() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, bar = \"\\baz\",}")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_concatenation() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, bar = \"baz\" # \"qux\"}")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_parens() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article(foo,)")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_string() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@string{foo=\"bar\"}")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }

    #[test]
    fn test_preamble() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@preamble{\n\"foo bar baz\"}")])
            .main("main.bib")
            .build()
            .formatting();

        let edit = format_bibtex_internal(&request, CancellationToken::none())
            .unwrap()
            .pop()
            .unwrap();

        assert_display_snapshot!(edit.new_text);
    }
}
