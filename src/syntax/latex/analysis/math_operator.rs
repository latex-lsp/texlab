use crate::syntax::latex::LatexCommand;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexMathOperator {
    pub command: Arc<LatexCommand>,
}

impl SyntaxNode for LatexMathOperator {
    fn range(&self) -> Range {
        self.command.range()
    }
}

impl LatexMathOperator {
    pub fn new(command: Arc<LatexCommand>) -> Self {
        Self { command }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        commands
            .iter()
            .filter(|cmd| MATH_OPERATOR_COMMANDS.contains(&cmd.name.text()))
            .filter(|cmd| cmd.args.len() >= 2)
            .map(|cmd| Arc::clone(&cmd))
            .map(LatexMathOperator::new)
            .collect()
    }
}

pub static MATH_OPERATOR_COMMANDS: &[&str] = &["\\DeclareMathOperator", "\\DeclareMathOperator*"];
