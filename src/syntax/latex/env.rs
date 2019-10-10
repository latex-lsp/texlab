use super::ast::*;
use crate::syntax::language::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentDelimiter {
    pub command: Arc<LatexCommand>,
}

impl LatexEnvironmentDelimiter {
    pub fn name(&self) -> Option<&LatexToken> {
        self.command.extract_word(0)
    }

    pub fn is_math(&self) -> bool {
        if let Some(name) = self.name() {
            LANGUAGE_DATA
                .math_environments
                .iter()
                .any(|env| env == name.text())
        } else {
            false
        }
    }

    pub fn is_enum(&self) -> bool {
        if let Some(name) = self.name() {
            LANGUAGE_DATA
                .enum_environments
                .iter()
                .any(|env| env == name.text())
        } else {
            false
        }
    }
}

impl SyntaxNode for LatexEnvironmentDelimiter {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironment {
    pub left: LatexEnvironmentDelimiter,
    pub right: LatexEnvironmentDelimiter,
}

impl LatexEnvironment {
    pub fn is_root(&self) -> bool {
        self.left
            .name()
            .iter()
            .chain(self.right.name().iter())
            .any(|name| name.text() == "document")
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut stack = Vec::new();
        let mut environments = Vec::new();
        for command in commands {
            if let Some(delimiter) = Self::parse_delimiter(command) {
                if delimiter.command.name.text() == "\\begin" {
                    stack.push(delimiter);
                } else if let Some(begin) = stack.pop() {
                    environments.push(Self {
                        left: begin,
                        right: delimiter,
                    });
                }
            }
        }
        environments
    }

    fn parse_delimiter(command: &Arc<LatexCommand>) -> Option<LatexEnvironmentDelimiter> {
        if command.name.text() != "\\begin" && command.name.text() != "\\end" {
            return None;
        }

        if command.args.is_empty() {
            return None;
        }

        if command.has_word(0)
            || command.args[0].children.is_empty()
            || command.args[0].right.is_none()
        {
            let delimiter = LatexEnvironmentDelimiter {
                command: Arc::clone(&command),
            };
            Some(delimiter)
        } else {
            None
        }
    }
}

impl SyntaxNode for LatexEnvironment {
    fn range(&self) -> Range {
        Range::new(self.left.start(), self.right.end())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentInfo {
    pub environments: Vec<LatexEnvironment>,
    pub is_standalone: bool,
}

impl LatexEnvironmentInfo {
    pub fn parse(commands: &[Arc<LatexCommand>]) -> Self {
        let environments = LatexEnvironment::parse(commands);
        let is_standalone = environments.iter().any(LatexEnvironment::is_root);
        Self {
            environments,
            is_standalone,
        }
    }
}
