use crate::syntax::latex::ast::*;
use std::sync::Arc;

pub struct LatexCommandAnalyzer {
    pub commands: Vec<Arc<LatexCommand>>,
}

impl LatexCommandAnalyzer {
    pub fn new() -> Self {
        LatexCommandAnalyzer {
            commands: Vec::new(),
        }
    }
}

impl LatexVisitor for LatexCommandAnalyzer {
    fn visit_root(&mut self, root: Arc<LatexRoot>) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: Arc<LatexGroup>) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        self.commands.push(Arc::clone(&command));
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, _text: Arc<LatexText>) {}
}

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;

    #[test]
    fn test() {
        let tree = LatexSyntaxTree::from("\\a[\\b]{\\c}{d}");
        let commands: Vec<&str> = tree
            .commands
            .iter()
            .map(|command| command.name.text())
            .collect();
        assert_eq!(vec!["\\a", "\\b", "\\c"], commands);
    }
}
