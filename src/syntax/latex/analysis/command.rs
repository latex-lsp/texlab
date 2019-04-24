use crate::syntax::latex::ast::*;

pub struct LatexCommandAnalyzer<'a> {
    pub commands: Vec<&'a LatexCommand>,
}

impl<'a> LatexCommandAnalyzer<'a> {
    pub fn new() -> Self {
        LatexCommandAnalyzer {
            commands: Vec::new(),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexCommandAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        self.commands.push(command);
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    #[test]
    fn test() {
        let tree = LatexSyntaxTree::from("\\a[\\b]{\\c}{d}");
        let mut analyzer = LatexCommandAnalyzer::new();
        analyzer.visit_root(&tree.root);
        let commands: Vec<&str> = analyzer
            .commands
            .iter()
            .map(|command| command.name.text())
            .collect();
        assert_eq!(vec!["\\a", "\\b", "\\c"], commands);
    }
}
