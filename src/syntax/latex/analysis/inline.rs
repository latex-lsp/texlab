use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInline {
    pub left: Arc<LatexMath>,
    pub right: Arc<LatexMath>,
}

impl SyntaxNode for LatexInline {
    fn range(&self) -> Range {
        Range::new(self.left.start(), self.right.end())
    }
}

impl LatexInline {
    pub fn new(left: Arc<LatexMath>, right: Arc<LatexMath>) -> Self {
        Self { left, right }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexInlineAnalyzer {
    inlines: Vec<LatexInline>,
    left: Option<Arc<LatexMath>>,
}

impl LatexInlineAnalyzer {
    pub fn parse_all(root: Arc<LatexRoot>) -> Vec<LatexInline> {
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
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: Arc<LatexText>) {
        LatexWalker::walk_text(self, text);
    }

    fn visit_comma(&mut self, comma: Arc<LatexComma>) {
        LatexWalker::walk_comma(self, comma);
    }

    fn visit_math(&mut self, math: Arc<LatexMath>) {
        if let Some(left) = &self.left {
            let inline = LatexInline::new(Arc::clone(&left), Arc::clone(&math));
            self.inlines.push(inline);
            self.left = None;
        } else {
            self.left = Some(Arc::clone(&math));
        }
        LatexWalker::walk_math(self, math);
    }
}
