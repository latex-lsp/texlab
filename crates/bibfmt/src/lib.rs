use line_index::LineIndex;
use rowan::{ast::AstNode, NodeOrToken};
use syntax::bibtex::{self, HasName, HasType, HasValue};

pub struct Options {
    pub insert_spaces: bool,
    pub tab_size: usize,
    pub line_length: usize,
}

impl Options {
    fn indent(&self) -> String {
        if self.insert_spaces {
            std::iter::repeat(' ').take(self.tab_size).collect()
        } else {
            String::from("\t")
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            insert_spaces: true,
            tab_size: 4,
            line_length: 80,
        }
    }
}

pub fn format(root: &bibtex::SyntaxNode, line_index: &LineIndex, options: &Options) -> String {
    let indent = options.indent();
    let mut output = String::new();
    for elem in root.children_with_tokens() {
        match elem {
            NodeOrToken::Token(token) => {
                output.push_str(token.text());
            }
            NodeOrToken::Node(node) => {
                let mut fmt = Formatter {
                    indent: &indent,
                    output: &mut output,
                    options,
                    align: Vec::new(),
                    line_index,
                };

                fmt.visit_node(node);
            }
        }
    }

    output
}

struct Formatter<'a> {
    output: &'a mut String,
    indent: &'a str,
    options: &'a Options,
    align: Vec<usize>,
    line_index: &'a LineIndex,
}

impl<'a> Formatter<'a> {
    fn visit_token_lowercase(&mut self, token: &bibtex::SyntaxToken) {
        self.output.push_str(&token.text().to_lowercase());
    }

    fn should_insert_space(
        &self,
        previous: &bibtex::SyntaxToken,
        current: &bibtex::SyntaxToken,
    ) -> bool {
        let previous = self.line_index.line_col(previous.text_range().end());
        let current = self.line_index.line_col(current.text_range().start());
        previous.line != current.line || previous.col < current.col
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
                    self.align.push(self.options.tab_size + count + 3);
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

                    if length + current_length + space_length > self.options.line_length {
                        self.output.push('\n');
                        self.output.push_str(self.indent.as_ref());
                        for _ in 0..=align - self.options.tab_size as usize {
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

#[cfg(test)]
mod tests;
