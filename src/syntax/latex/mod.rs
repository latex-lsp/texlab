pub mod ast;
pub mod constants;
pub mod lexer;
pub mod parser;

use crate::range;
use crate::syntax::latex::ast::*;
use crate::syntax::latex::lexer::LatexLexer;
use crate::syntax::latex::parser::LatexParser;
use crate::syntax::text::Node;
use lsp_types::Position;
use std::rc::Rc;

struct LatexFinder {
    pub position: Option<Position>,
    pub results: Vec<LatexNode>,
}

impl LatexFinder {
    pub fn new(position: Option<Position>) -> Self {
        LatexFinder {
            position,
            results: Vec::new(),
        }
    }

    fn check_range(&self, node: &LatexNode) -> bool {
        if let Some(position) = self.position {
            range::contains(node.range(), position)
        } else {
            true
        }
    }
}

impl LatexVisitor<()> for LatexFinder {
    fn visit_environment(&mut self, environment: Rc<LatexEnvironment>) {
        let node = LatexNode::Environment(Rc::clone(&environment));
        if self.check_range(&node) {
            self.results.push(node);
            self.visit_command(Rc::clone(&environment.left.command));

            for child in &environment.children {
                child.accept(self);
            }

            if let Some(ref right) = environment.right {
                self.visit_command(Rc::clone(&right.command));
            }
        }
    }

    fn visit_equation(&mut self, equation: Rc<LatexEquation>) {
        let node = LatexNode::Equation(Rc::clone(&equation));
        if self.check_range(&node) {
            self.results.push(node);
            self.visit_command(Rc::clone(&equation.left));

            for child in &equation.children {
                child.accept(self);
            }

            if let Some(ref right) = equation.right {
                self.visit_command(Rc::clone(&right));
            }
        }
    }

    fn visit_group(&mut self, group: Rc<LatexGroup>) {
        let node = LatexNode::Group(Rc::clone(&group));
        if self.check_range(&node) {
            self.results.push(node);

            for child in &group.children {
                child.accept(self);
            }
        }
    }

    fn visit_command(&mut self, command: Rc<LatexCommand>) {
        let node = LatexNode::Command(Rc::clone(&command));
        if self.check_range(&node) {
            self.results.push(node);

            if let Some(ref options) = command.options {
                self.visit_group(Rc::clone(&options));
            }

            for arg in &command.args {
                self.visit_group(Rc::clone(&arg));
            }
        }
    }

    fn visit_text(&mut self, text: Rc<LatexText>) {
        let node = LatexNode::Text(Rc::clone(&text));
        if self.check_range(&node) {
            self.results.push(node);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSyntaxTree {
    pub root: LatexRoot,
    pub descendants: Vec<LatexNode>,
}

impl From<LatexRoot> for LatexSyntaxTree {
    fn from(root: LatexRoot) -> Self {
        let mut finder = LatexFinder::new(None);
        for child in &root.children {
            child.accept(&mut finder);
        }
        LatexSyntaxTree {
            root,
            descendants: finder.results,
        }
    }
}

impl From<&str> for LatexSyntaxTree {
    fn from(text: &str) -> Self {
        let tokens = LatexLexer::from(text);
        let mut parser = LatexParser::new(tokens);
        let root = parser.root();
        LatexSyntaxTree::from(root)
    }
}

impl LatexSyntaxTree {
    fn find(&self, position: Position) -> Vec<LatexNode> {
        let mut finder = LatexFinder::new(Some(position));
        for child in &self.root.children {
            child.accept(&mut finder);
        }
        finder.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify(text: &str, expected: Vec<LatexNodeKind>) {
        let actual: Vec<LatexNodeKind> = LatexSyntaxTree::from(text)
            .descendants
            .iter()
            .map(|node| node.kind())
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_empty() {
        verify("", Vec::new());
    }

    #[test]
    fn test_command() {
        verify("\\foo", vec![LatexNodeKind::Command]);
        verify("\\foo@bar*", vec![LatexNodeKind::Command]);
        verify("\\**", vec![LatexNodeKind::Command, LatexNodeKind::Text]);
        verify("\\%", vec![LatexNodeKind::Command]);
        verify(
            "\\foo[bar]",
            vec![
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Options),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "\\foo[bar]{baz}{qux}",
            vec![
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Options),
                LatexNodeKind::Text,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
            ],
        );
    }

    #[test]
    fn test_inline() {
        verify(
            "$ x $",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Inline),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "$x$ $$y$$",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Inline),
                LatexNodeKind::Text,
                LatexNodeKind::Group(LatexGroupKind::Inline),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "${\\foo}$",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Inline),
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Command,
            ],
        );
        verify(
            "$}$",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Inline),
                LatexNodeKind::Group(LatexGroupKind::Inline),
            ],
        )
    }

    #[test]
    fn test_equation() {
        verify(
            "\\[foo\\]",
            vec![
                LatexNodeKind::Equation,
                LatexNodeKind::Command,
                LatexNodeKind::Text,
                LatexNodeKind::Command,
            ],
        );
        verify(
            "\\[}foo\\]",
            vec![
                LatexNodeKind::Equation,
                LatexNodeKind::Command,
                LatexNodeKind::Text,
                LatexNodeKind::Command,
            ],
        );
        verify(
            "\\[\\foo\\]",
            vec![
                LatexNodeKind::Equation,
                LatexNodeKind::Command,
                LatexNodeKind::Command,
                LatexNodeKind::Command,
            ],
        );
    }

    #[test]
    fn test_group() {
        verify("}", Vec::new());
        verify(
            "{{foo}}",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "{foo",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
            ],
        );
    }

    #[test]
    fn test_environment() {
        verify(
            "\\begin{a}foo\\end{b}",
            vec![
                LatexNodeKind::Environment,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
                LatexNodeKind::Text,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "\\begin{a}foo",
            vec![
                LatexNodeKind::Environment,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
                LatexNodeKind::Text,
            ],
        );
        verify(
            "\\begin{}foo\\end{}",
            vec![
                LatexNodeKind::Environment,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
            ],
        );
        verify(
            "\\end{a}",
            vec![
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
            ],
        );
        verify(
            "{\\begin{a}foo}bar",
            vec![
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Environment,
                LatexNodeKind::Command,
                LatexNodeKind::Group(LatexGroupKind::Group),
                LatexNodeKind::Text,
                LatexNodeKind::Text,
                LatexNodeKind::Text,
            ],
        );
    }
}
