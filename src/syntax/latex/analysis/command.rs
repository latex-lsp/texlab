use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexCommandAnalyzer {
    pub commands: Vec<Arc<LatexCommand>>,
}

impl LatexCommandAnalyzer {
    pub fn find(root: Arc<LatexRoot>) -> Vec<Arc<LatexCommand>> {
        let mut analyzer = LatexCommandAnalyzer::default();
        analyzer.visit_root(root);
        analyzer.commands
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
