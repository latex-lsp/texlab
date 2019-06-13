use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexInlineAnalyzer {
    inlines: Vec<Arc<LatexGroup>>,
}

impl LatexInlineAnalyzer {
    pub fn find(root: Arc<LatexRoot>) -> Vec<Arc<LatexGroup>> {
        let mut analyzer = Self::default();
        analyzer.visit_root(root);
        analyzer.inlines
    }
}

impl LatexVisitor for LatexInlineAnalyzer {
    fn visit_root(&mut self, root: Arc<LatexRoot>) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: Arc<LatexGroup>) {
        if group.kind == LatexGroupKind::Math {
            self.inlines.push(Arc::clone(&group));
        }
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, _text: Arc<LatexText>) {}
}
