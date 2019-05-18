use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentDelimiter {
    pub command: Arc<LatexCommand>,
}

impl LatexEnvironmentDelimiter {
    pub fn name(&self) -> Option<&LatexToken> {
        self.command.extract_word(0)
    }

    pub fn new(command: Arc<LatexCommand>) -> Self {
        LatexEnvironmentDelimiter { command }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironment {
    pub left: LatexEnvironmentDelimiter,
    pub right: LatexEnvironmentDelimiter,
}

impl LatexEnvironment {
    pub fn new(left: LatexEnvironmentDelimiter, right: LatexEnvironmentDelimiter) -> Self {
        LatexEnvironment { left, right }
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

    pub fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
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

pub const ENVIRONMENT_COMMANDS: &'static [&'static str] = &["\\begin", "\\end"];

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;

    #[test]
    fn test_nested() {
        let tree = LatexSyntaxTree::from("\\begin{foo}\\begin{bar}\\end{baz}\\end{qux}");
        let environments = tree.environments;
        assert_eq!(2, environments.len());
        assert_eq!("bar", environments[0].left.name().unwrap().text());
        assert_eq!("baz", environments[0].right.name().unwrap().text());
        assert_eq!("foo", environments[1].left.name().unwrap().text());
        assert_eq!("qux", environments[1].right.name().unwrap().text());
    }

    #[test]
    fn test_empty_name() {
        let tree = LatexSyntaxTree::from("\\begin{}\\end{}");
        let environments = tree.environments;
        assert_eq!(1, environments.len());
        assert_eq!(None, environments[0].left.name());
        assert_eq!(None, environments[0].right.name());
    }

    #[test]
    fn test_ummatched() {
        let tree = LatexSyntaxTree::from("\\end{foo} \\begin{bar}");
        assert_eq!(tree.environments, Vec::new());
    }
}
