use crate::syntax::latex::ast::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentDelimiter<'a> {
    pub command: &'a LatexCommand,
    pub name: Option<&'a LatexToken>,
}

impl<'a> LatexEnvironmentDelimiter<'a> {
    pub fn new(command: &'a LatexCommand, name: Option<&'a LatexToken>) -> Self {
        LatexEnvironmentDelimiter { command, name }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironment<'a> {
    pub left: LatexEnvironmentDelimiter<'a>,
    pub right: LatexEnvironmentDelimiter<'a>,
}

impl<'a> LatexEnvironment<'a> {
    pub fn new(left: LatexEnvironmentDelimiter<'a>, right: LatexEnvironmentDelimiter<'a>) -> Self {
        LatexEnvironment { left, right }
    }
}

pub struct LatexEnvironmentAnalyzer<'a> {
    pub environments: Vec<LatexEnvironment<'a>>,
    stack: Vec<LatexEnvironmentDelimiter<'a>>,
}

impl<'a> LatexEnvironmentAnalyzer<'a> {
    pub fn new() -> Self {
        LatexEnvironmentAnalyzer {
            environments: Vec::new(),
            stack: Vec::new(),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexEnvironmentAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if let Some(delimiter) = parse_delimiter(command) {
            if delimiter.command.name.text() == ENVIRONMENT_COMMANDS[0] {
                self.stack.push(delimiter);
            } else if let Some(begin) = self.stack.pop() {
                self.environments
                    .push(LatexEnvironment::new(begin, delimiter));
            }
        }

        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

fn parse_delimiter(command: &LatexCommand) -> Option<LatexEnvironmentDelimiter> {
    if !ENVIRONMENT_COMMANDS.contains(&command.name.text()) {
        return None;
    }

    if let Some(name) = command.extract_word(0) {
        let delimiter = LatexEnvironmentDelimiter::new(command, Some(name));
        return Some(delimiter);
    }

    if !command.args.is_empty() && command.args[0].children.is_empty() {
        let delimiter = LatexEnvironmentDelimiter::new(command, None);
        Some(delimiter)
    } else {
        None
    }
}

pub const ENVIRONMENT_COMMANDS: &'static [&'static str] = &["\\begin", "\\end"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    fn analyze(tree: &LatexSyntaxTree) -> Vec<LatexEnvironment> {
        let mut analyzer = LatexEnvironmentAnalyzer::new();
        analyzer.visit_root(&tree.root);
        analyzer.environments
    }

    #[test]
    fn test_nested() {
        let tree = LatexSyntaxTree::from("\\begin{foo}\\begin{bar}\\end{baz}\\end{qux}");
        let environments = analyze(&tree);
        assert_eq!(2, environments.len());
        assert_eq!("bar", environments[0].left.name.unwrap().text());
        assert_eq!("baz", environments[0].right.name.unwrap().text());
        assert_eq!("foo", environments[1].left.name.unwrap().text());
        assert_eq!("qux", environments[1].right.name.unwrap().text());
    }

    #[test]
    fn test_empty_name() {
        let tree = LatexSyntaxTree::from("\\begin{}\\end{}");
        let environments = analyze(&tree);
        assert_eq!(1, environments.len());
        assert_eq!(None, environments[0].left.name);
        assert_eq!(None, environments[0].right.name);
    }

    #[test]
    fn test_ummatched() {
        let tree = LatexSyntaxTree::from("\\end{foo} \\begin{bar}");
        assert_eq!(analyze(&tree), Vec::new());
    }
}
