use crate::syntax::bibtex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::DocumentFormattingParams;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexFormattingOptions {
    tab_size: usize,
    insert_spaces: bool,
    line_length: usize,
}

impl BibtexFormattingOptions {
    pub fn new(tab_size: usize, insert_spaces: bool, line_length: usize) -> Self {
        let line_length = if line_length <= 0 {
            std::usize::MAX
        } else {
            line_length
        };

        BibtexFormattingOptions {
            tab_size,
            insert_spaces,
            line_length,
        }
    }
}

impl Default for BibtexFormattingOptions {
    fn default() -> Self {
        BibtexFormattingOptions::new(4, true, 120)
    }
}

pub struct BibtexFormatter {
    pub options: BibtexFormattingOptions,
    indent: String,
    output: String,
}

impl BibtexFormatter {
    pub fn new(options: BibtexFormattingOptions) -> Self {
        let indent = if options.insert_spaces {
            let mut buffer = String::new();
            for _ in 0..options.tab_size {
                buffer.push(' ');
            }
            buffer
        } else {
            String::from("\t")
        };

        BibtexFormatter {
            options,
            indent,
            output: String::new(),
        }
    }

    pub fn format_declaration(&mut self, declaration: &BibtexDeclaration) {
        match declaration {
            BibtexDeclaration::Comment(comment) => {
                let text = comment.token.text();
                self.output.push_str(text);
            }
            BibtexDeclaration::Preamble(preamble) => {
                self.format_token(&preamble.kind);
                self.output.push('{');
                if let Some(ref content) = preamble.content {
                    self.format_content(content, self.output.chars().count());
                    self.output.push('}');
                }
            }
            BibtexDeclaration::String(string) => {
                self.format_token(&string.kind);
                self.output.push('{');
                if let Some(ref name) = string.name {
                    self.output.push_str(name.text());
                    self.output.push_str(" = ");
                    if let Some(ref value) = string.value {
                        self.format_content(value, self.output.chars().count());
                        self.output.push('}');
                    }
                }
            }
            BibtexDeclaration::Entry(entry) => {
                self.format_token(&entry.kind);
                self.output.push('{');
                if let Some(ref key) = entry.key {
                    self.output.push_str(key.text());
                    self.output.push(',');
                    self.output.push('\n');
                    for field in &entry.fields {
                        self.format_field(field);
                    }
                    self.output.push('}');
                }
            }
        }
    }

    fn format_field(&mut self, field: &BibtexField) {
        self.output.push_str(self.indent.as_ref());
        self.format_token(&field.name);
        self.output.push_str(" = ");
        let count = field.name.text().chars().count();
        let align = self.options.tab_size as usize + count + 3;
        if let Some(ref content) = field.content {
            self.format_content(content, align);
            self.output.push(',');
            self.output.push('\n');
        }
    }

    fn format_content(&mut self, content: &BibtexContent, align: usize) {
        let mut analyzer = BibtexContentAnalyzer::new();
        content.accept(&mut analyzer);
        let tokens = analyzer.tokens;
        self.output.push_str(tokens[0].text());

        let mut length = align + tokens[0].text().chars().count();
        for i in 1..tokens.len() {
            let previous = tokens[i - 1];
            let current = tokens[i];
            let current_length = current.text().chars().count();

            let insert_space = should_insert_space(previous, current);
            let space_length = if insert_space { 1 } else { 0 };

            if length + current_length + space_length > self.options.line_length {
                self.output.push('\n');
                self.output.push_str(self.indent.as_ref());
                for j in 0..align - self.options.tab_size + 1 {
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

    fn format_token(&mut self, token: &BibtexToken) {
        self.output.push_str(token.text().to_lowercase().as_ref());
    }
}

fn should_insert_space(previous: &BibtexToken, current: &BibtexToken) -> bool {
    previous.start().line != current.start().line
        || previous.end().character < current.start().character
}

struct BibtexContentAnalyzer<'a> {
    pub tokens: Vec<&'a BibtexToken>,
}

impl<'a> BibtexContentAnalyzer<'a> {
    pub fn new() -> Self {
        BibtexContentAnalyzer { tokens: Vec::new() }
    }
}

impl<'a> BibtexVisitor<'a> for BibtexContentAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a BibtexRoot) {}

    fn visit_comment(&mut self, comment: &'a BibtexComment) {}

    fn visit_preamble(&mut self, preamble: &'a BibtexPreamble) {}

    fn visit_string(&mut self, string: &'a BibtexString) {}

    fn visit_entry(&mut self, entry: &'a BibtexEntry) {}

    fn visit_field(&mut self, field: &'a BibtexField) {}

    fn visit_word(&mut self, word: &'a BibtexWord) {
        self.tokens.push(&word.token);
    }

    fn visit_command(&mut self, command: &'a BibtexCommand) {
        self.tokens.push(&command.token);
    }

    fn visit_quoted_content(&mut self, content: &'a BibtexQuotedContent) {
        self.tokens.push(&content.left);
        BibtexWalker::walk_quoted_content(self, content);
        if let Some(ref right) = content.right {
            self.tokens.push(right);
        }
    }

    fn visit_braced_content(&mut self, content: &'a BibtexBracedContent) {
        self.tokens.push(&content.left);
        BibtexWalker::walk_braced_content(self, content);
        if let Some(ref right) = content.right {
            self.tokens.push(right);
        }
    }

    fn visit_concat(&mut self, concat: &'a BibtexConcat) {
        concat.left.accept(self);
        self.tokens.push(&concat.operator);
        if let Some(ref right) = concat.right {
            right.accept(self);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::bibtex::BibtexSyntaxTree;
    use indoc::indoc;

    fn verify(source: &str, expected: &str, line_length: usize) {
        let tree = BibtexSyntaxTree::from(source);
        let options = BibtexFormattingOptions::new(4, true, line_length);
        let mut formatter = BibtexFormatter::new(options);
        formatter.format_declaration(&tree.root.children[0]);
        assert_eq!(expected, formatter.output);
    }

    #[test]
    fn test_wrap_long_lines() {
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
    fn test_line_length_zero() {
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
    fn test_trailing_commas() {
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
    fn test_insert_braces() {
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
    fn test_commands() {
        let source = "@article{foo, bar = \"\\baz\",}";
        let expected = indoc!(
            "@article{foo,
                bar = \"\\baz\",
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn test_concatenation() {
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
    fn test_parentheses() {
        let source = "@article(foo,)";
        let expected = indoc!(
            "
            @article{foo,
            }"
        );
        verify(source, expected, 30);
    }

    #[test]
    fn test_string() {
        let source = "@string{foo=\"bar\"}";
        let expected = "@string{foo = \"bar\"}";
        verify(source, expected, 30);
    }

    #[test]
    fn test_preamble() {
        let source = "@preamble{\n\"foo bar baz\"}";
        let expected = "@preamble{\"foo bar baz\"}";
        verify(source, expected, 30);
    }
}
