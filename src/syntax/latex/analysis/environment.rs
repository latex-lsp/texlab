use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentDelimiter {
    pub command: Arc<LatexCommand>,
}

impl LatexEnvironmentDelimiter {
    pub fn new(command: Arc<LatexCommand>) -> Self {
        Self { command }
    }

    pub fn name(&self) -> Option<&LatexToken> {
        self.command.extract_word(0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironment {
    pub left: LatexEnvironmentDelimiter,
    pub right: LatexEnvironmentDelimiter,
}

impl SyntaxNode for LatexEnvironment {
    fn range(&self) -> Range {
        Range::new(self.left.command.start(), self.right.command.end())
    }
}

impl LatexEnvironment {
    pub fn new(left: LatexEnvironmentDelimiter, right: LatexEnvironmentDelimiter) -> Self {
        Self { left, right }
    }

    fn parse_delimiter(command: Arc<LatexCommand>) -> Option<LatexEnvironmentDelimiter> {
        if !ENVIRONMENT_COMMANDS.contains(&command.name.text()) {
            return None;
        }

        if command.has_word(0) {
            let delimiter = LatexEnvironmentDelimiter::new(Arc::clone(&command));
            return Some(delimiter);
        }

        if !command.args.is_empty() && command.args[0].children.is_empty() {
            let delimiter = LatexEnvironmentDelimiter::new(Arc::clone(&command));
            Some(delimiter)
        } else {
            None
        }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut stack = Vec::new();
        let mut environments = Vec::new();
        for command in commands {
            if let Some(delimiter) = Self::parse_delimiter(Arc::clone(&command)) {
                if delimiter.command.name.text() == ENVIRONMENT_COMMANDS[0] {
                    stack.push(delimiter);
                } else if let Some(begin) = stack.pop() {
                    environments.push(LatexEnvironment::new(begin, delimiter));
                }
            }
        }
        environments
    }
}

pub static ENVIRONMENT_COMMANDS: &[&str] = &["\\begin", "\\end"];
