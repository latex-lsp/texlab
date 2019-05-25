use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation {
    pub left: Arc<LatexCommand>,
    pub right: Arc<LatexCommand>,
}

impl LatexEquation {
    pub fn new(left: Arc<LatexCommand>, right: Arc<LatexCommand>) -> Self {
        LatexEquation { left, right }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut equations = Vec::new();
        let mut left = None;
        for command in commands {
            if command.name.text() == EQUATION_COMMANDS[0] {
                left = Some(command);
            } else if let Some(begin) = left {
                equations.push(LatexEquation::new(Arc::clone(&begin), Arc::clone(&command)));
                left = None;
            }
        }
        equations
    }
}

pub static EQUATION_COMMANDS: &[&str] = &["\\[", "\\]"];
