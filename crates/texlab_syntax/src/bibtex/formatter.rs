use super::ast::*;
use crate::text::SyntaxNode;
use petgraph::graph::NodeIndex;
use std::{i32, string::String as StdString};
use texlab_protocol::BibtexFormattingOptions;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FormattingParams<'a> {
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub options: &'a BibtexFormattingOptions,
}

impl<'a> FormattingParams<'a> {
    fn line_length(self) -> i32 {
        let line_length = self.options.line_length.unwrap_or(120);
        if line_length <= 0 {
            i32::MAX
        } else {
            line_length
        }
    }

    fn indent(self) -> StdString {
        if self.insert_spaces {
            let mut buffer = StdString::new();
            for _ in 0..self.tab_size {
                buffer.push(' ');
            }
            buffer
        } else {
            "\t".into()
        }
    }
}

#[derive(Debug, Clone)]
struct Formatter<'a> {
    params: FormattingParams<'a>,
    indent: StdString,
    output: StdString,
    align: Vec<usize>,
}

impl<'a> Formatter<'a> {
    fn new(params: FormattingParams<'a>) -> Self {
        Self {
            params,
            indent: params.indent(),
            output: StdString::new(),
            align: Vec::new(),
        }
    }

    fn visit_token_lowercase(&mut self, token: &Token) {
        self.output.push_str(token.text().to_lowercase().as_ref());
    }

    fn should_insert_space(previous: &Token, current: &Token) -> bool {
        previous.start().line != current.start().line
            || previous.end().character < current.start().character
    }
}

impl<'a, 'b> Visitor<'b> for Formatter<'a> {
    fn visit(&mut self, tree: &'b Tree, node: NodeIndex) {
        match &tree.graph[node] {
            Node::Root(_) => tree.walk(self, node),
            Node::Comment(comment) => self.output.push_str(comment.token.text()),
            Node::Preamble(preamble) => {
                self.visit_token_lowercase(&preamble.ty);
                self.output.push('{');
                if tree.has_children(node) {
                    self.align.push(self.output.chars().count());
                    tree.walk(self, node);
                    self.output.push('}');
                }
            }
            Node::String(string) => {
                self.visit_token_lowercase(&string.ty);
                self.output.push('{');
                if let Some(name) = &string.name {
                    self.output.push_str(name.text());
                    self.output.push_str(" = ");
                    if tree.has_children(node) {
                        self.align.push(self.output.chars().count());
                        tree.walk(self, node);
                        self.output.push('}');
                    }
                }
            }
            Node::Entry(entry) => {
                self.visit_token_lowercase(&entry.ty);
                self.output.push('{');
                if let Some(key) = &entry.key {
                    self.output.push_str(key.text());
                    self.output.push(',');
                    self.output.push('\n');
                    tree.walk(self, node);
                    self.output.push('}');
                }
            }
            Node::Field(field) => {
                self.output.push_str(&self.indent);
                self.visit_token_lowercase(&field.name);
                self.output.push_str(" = ");
                if tree.has_children(node) {
                    let count = field.name.text().chars().count();
                    self.align.push(self.params.tab_size as usize + count + 3);
                    tree.walk(self, node);
                    self.output.push(',');
                    self.output.push('\n');
                }
            }
            Node::Word(_)
            | Node::Command(_)
            | Node::BracedContent(_)
            | Node::QuotedContent(_)
            | Node::Concat(_) => {
                let mut analyzer = ContentAnalyzer::default();
                analyzer.visit(tree, node);
                let tokens = analyzer.tokens;
                self.output.push_str(tokens[0].text());

                let align = self.align.pop().unwrap_or_default();
                let mut length = align + tokens[0].text().chars().count();
                for i in 1..tokens.len() {
                    let previous = tokens[i - 1];
                    let current = tokens[i];
                    let current_length = current.text().chars().count();

                    let insert_space = Self::should_insert_space(previous, current);
                    let space_length = if insert_space { 1 } else { 0 };

                    if length + current_length + space_length > self.params.line_length() as usize {
                        self.output.push('\n');
                        self.output.push_str(self.indent.as_ref());
                        for _ in 0..=align - self.params.tab_size {
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
        }
    }
}

#[derive(Debug, Default)]
struct ContentAnalyzer<'a> {
    tokens: Vec<&'a Token>,
}

impl<'a> Visitor<'a> for ContentAnalyzer<'a> {
    fn visit(&mut self, tree: &'a Tree, node: NodeIndex) {
        match &tree.graph[node] {
            Node::Root(_)
            | Node::Comment(_)
            | Node::Preamble(_)
            | Node::String(_)
            | Node::Entry(_)
            | Node::Field(_) => tree.walk(self, node),
            Node::Word(word) => self.tokens.push(&word.token),
            Node::Command(cmd) => self.tokens.push(&cmd.token),
            Node::QuotedContent(content) => {
                self.tokens.push(&content.left);
                tree.walk(self, node);
                if let Some(right) = &content.right {
                    self.tokens.push(right);
                }
            }
            Node::BracedContent(content) => {
                self.tokens.push(&content.left);
                tree.walk(self, node);
                if let Some(right) = &content.right {
                    self.tokens.push(right);
                }
            }
            Node::Concat(concat) => {
                let mut children = tree.children(node);
                let left = children.next().unwrap();
                self.visit(tree, left);
                self.tokens.push(&concat.operator);
                children.for_each(|right| self.visit(tree, right));
            }
        }
    }
}

pub fn format(tree: &Tree, node: NodeIndex, params: FormattingParams) -> StdString {
    let mut formatter = Formatter::new(params);
    formatter.visit(tree, node);
    formatter.output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bibtex;
    use indoc::indoc;

    fn verify(source: &str, expected: &str, line_length: i32) {
        let tree = bibtex::open(source);
        let options = BibtexFormattingOptions {
            line_length: Some(line_length),
            formatter: None,
        };

        let mut children = tree.children(tree.root);
        let declaration = children.next().unwrap();
        assert_eq!(children.next(), None);

        let actual = format(
            &tree,
            declaration,
            FormattingParams {
                tab_size: 4,
                insert_spaces: true,
                options: &options,
            },
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn wrap_long_lines() {
        let source =
            "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}";
        let expected = indoc!(
            "
            @article{foo,
                bar = {Lorem ipsum dolor
                       sit amet,
                       consectetur
                       adipiscing elit.},
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn line_length_zero() {
        let source =
            "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}";
        let expected = indoc!(
            "
            @article{foo,
                bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},
            }"
        );
        verify(source, expected, 0);
    }

    #[test]
    fn trailing_commas() {
        let source = "@article{foo, bar = baz}";
        let expected = indoc!(
            "
            @article{foo,
                bar = baz,
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn insert_braces() {
        let source = "@article{foo, bar = baz,";
        let expected = indoc!(
            "
            @article{foo,
                bar = baz,
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn commands() {
        let source = "@article{foo, bar = \"\\baz\",}";
        let expected = indoc!(
            "@article{foo,
                bar = \"\\baz\",
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn concatenation() {
        let source = "@article{foo, bar = \"baz\" # \"qux\"}";
        let expected = indoc!(
            "
            @article{foo,
                bar = \"baz\" # \"qux\",
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn parentheses() {
        let source = "@article(foo,)";
        let expected = indoc!(
            "
            @article{foo,
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn string() {
        let source = "@string{foo=\"bar\"}";
        let expected = "@string{foo = \"bar\"}";
        verify(source, expected, 30);
    }

    #[test]
    fn preamble() {
        let source = "@preamble{\n\"foo bar baz\"}";
        let expected = "@preamble{\"foo bar baz\"}";
        verify(source, expected, 30);
    }
}
