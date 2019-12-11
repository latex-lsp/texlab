use super::ast::*;
use crate::syntax::text::*;
use texlab_protocol::Position;
use std::sync::Arc;

#[derive(Debug)]
pub struct LatexPrinter {
    pub output: String,
    position: Position,
}

impl LatexPrinter {
    pub fn new(start_position: Position) -> Self {
        Self {
            output: String::new(),
            position: start_position,
        }
    }

    fn synchronize(&mut self, position: Position) {
        while self.position.line < position.line {
            self.output.push('\n');
            self.position.line += 1;
        }

        while self.position.character < position.character {
            self.output.push(' ');
            self.position.character += 1;
        }
    }

    fn print_token(&mut self, token: &LatexToken) {
        self.synchronize(token.start());
        self.output.push_str(token.text());
        self.position.character += token.end().character - token.start().character;
        self.synchronize(token.end());
    }
}

impl LatexVisitor for LatexPrinter {
    fn visit_root(&mut self, root: Arc<LatexRoot>) {
        for child in &root.children {
            child.accept(self);
        }
    }

    fn visit_group(&mut self, group: Arc<LatexGroup>) {
        self.print_token(&group.left);
        for child in &group.children {
            child.accept(self);
        }
        if let Some(right) = &group.right {
            self.print_token(right);
        }
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        self.print_token(&command.name);
        for group in &command.groups {
            self.visit_group(Arc::clone(&group));
        }
    }

    fn visit_text(&mut self, text: Arc<LatexText>) {
        for word in &text.words {
            self.print_token(word);
        }
    }

    fn visit_comma(&mut self, comma: Arc<LatexComma>) {
        self.print_token(&comma.token);
    }

    fn visit_math(&mut self, math: Arc<LatexMath>) {
        self.print_token(&math.token)
    }
}
