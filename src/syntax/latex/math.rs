use super::ast::*;
use crate::syntax::language::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation {
    pub left: Arc<LatexCommand>,
    pub right: Arc<LatexCommand>,
}

impl LatexEquation {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut equations = Vec::new();
        let mut left = None;
        for command in commands {
            let name = command.name.text();
            if name == "\\[" || name == "\\(" {
                left = Some(command);
            } else if name == "\\]" || name == "\\)" {
                if let Some(begin) = left {
                    equations.push(Self {
                        left: Arc::clone(&begin),
                        right: Arc::clone(&command),
                    });
                    left = None;
                }
            }
        }
        equations
    }
}

impl SyntaxNode for LatexEquation {
    fn range(&self) -> Range {
        Range::new(self.left.start(), self.right.end())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInline {
    pub left: Arc<LatexMath>,
    pub right: Arc<LatexMath>,
}

impl LatexInline {
    fn parse(root: Arc<LatexRoot>) -> Vec<Self> {
        let mut analyzer = LatexInlineAnalyzer::default();
        analyzer.visit_root(root);
        analyzer.inlines
    }
}

impl SyntaxNode for LatexInline {
    fn range(&self) -> Range {
        Range::new(self.left.start(), self.right.end())
    }
}

#[derive(Debug, Default)]
struct LatexInlineAnalyzer {
    inlines: Vec<LatexInline>,
    left: Option<Arc<LatexMath>>,
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
            let inline = LatexInline {
                left: Arc::clone(&left),
                right: Arc::clone(&math),
            };
            self.inlines.push(inline);
            self.left = None;
        } else {
            self.left = Some(Arc::clone(&math));
        }
        LatexWalker::walk_math(self, math);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexMathOperator {
    pub command: Arc<LatexCommand>,
    pub definition: Arc<LatexCommand>,
    pub definition_index: usize,
    pub implementation_index: usize,
}

impl LatexMathOperator {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut operators = Vec::new();
        for command in commands {
            for LatexMathOperatorCommand {
                name,
                definition_index,
                implementation_index,
            } in &LANGUAGE_DATA.math_operator_commands
            {
                if command.name.text() == name
                    && command.args.len() > *definition_index
                    && command.args.len() > *implementation_index
                {
                    let definition = command.args[0].children.iter().next();
                    if let Some(LatexContent::Command(definition)) = definition {
                        operators.push(Self {
                            command: Arc::clone(command),
                            definition: Arc::clone(definition),
                            definition_index: *definition_index,
                            implementation_index: *implementation_index,
                        })
                    }
                }
            }
        }
        operators
    }
}

impl SyntaxNode for LatexMathOperator {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexTheoremDefinition {
    pub command: Arc<LatexCommand>,
    pub index: usize,
}

impl LatexTheoremDefinition {
    pub fn name(&self) -> &LatexToken {
        self.command.extract_word(self.index).unwrap()
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut definitions = Vec::new();
        for command in commands {
            for LatexTheoremDefinitionCommand { name, index } in
                &LANGUAGE_DATA.theorem_definition_commands
            {
                if command.name.text() == name && command.has_word(*index) {
                    definitions.push(Self {
                        command: Arc::clone(&command),
                        index: *index,
                    });
                }
            }
        }
        definitions
    }
}

impl SyntaxNode for LatexTheoremDefinition {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexMathInfo {
    pub equations: Vec<LatexEquation>,
    pub inlines: Vec<LatexInline>,
    pub operators: Vec<LatexMathOperator>,
    pub theorem_definitions: Vec<LatexTheoremDefinition>,
}

impl LatexMathInfo {
    pub fn parse(root: Arc<LatexRoot>, commands: &[Arc<LatexCommand>]) -> Self {
        Self {
            equations: LatexEquation::parse(commands),
            inlines: LatexInline::parse(root),
            operators: LatexMathOperator::parse(commands),
            theorem_definitions: LatexTheoremDefinition::parse(commands),
        }
    }
}
