use crate::syntax::latex::constants::*;
use crate::syntax::text::{Node, Span};
use lsp_types::Range;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexTokenKind {
    Word,
    Command,
    Math,
    BeginOptions,
    EndOptions,
    BeginGroup,
    EndGroup,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexToken {
    pub span: Span,
    pub kind: LatexTokenKind,
}

impl LatexToken {
    pub fn new(span: Span, kind: LatexTokenKind) -> Self {
        LatexToken { span, kind }
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

impl Node for LatexToken {
    fn range(&self) -> Range {
        self.span.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexRoot {
    pub children: Vec<LatexNode>,
}

impl LatexRoot {
    pub fn new(children: Vec<LatexNode>) -> Self {
        LatexRoot { children }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexNodeKind {
    Environment,
    Equation,
    Group(LatexGroupKind),
    Command,
    Text,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexNode {
    Environment(Rc<LatexEnvironment>),
    Equation(Rc<LatexEquation>),
    Group(Rc<LatexGroup>),
    Command(Rc<LatexCommand>),
    Text(Rc<LatexText>),
}

impl LatexNode {
    pub fn accept<T>(&self, visitor: &mut LatexVisitor<T>) -> T {
        match self {
            LatexNode::Environment(environment) => visitor.visit_environment(environment.clone()),
            LatexNode::Equation(equation) => visitor.visit_equation(equation.clone()),
            LatexNode::Group(group) => visitor.visit_group(group.clone()),
            LatexNode::Command(command) => visitor.visit_command(command.clone()),
            LatexNode::Text(text) => visitor.visit_text(text.clone()),
        }
    }

    pub fn kind(&self) -> LatexNodeKind {
        match self {
            LatexNode::Environment(_) => LatexNodeKind::Environment,
            LatexNode::Equation(_) => LatexNodeKind::Equation,
            LatexNode::Group(group) => LatexNodeKind::Group(group.kind),
            LatexNode::Command(_) => LatexNodeKind::Command,
            LatexNode::Text(_) => LatexNodeKind::Text,
        }
    }
}

impl Node for LatexNode {
    fn range(&self) -> Range {
        match self {
            LatexNode::Environment(environment) => environment.range,
            LatexNode::Equation(equation) => equation.range,
            LatexNode::Group(group) => group.range,
            LatexNode::Command(command) => command.range,
            LatexNode::Text(text) => text.range,
        }
    }
}

pub trait LatexVisitor<T> {
    fn visit_environment(&mut self, environment: Rc<LatexEnvironment>) -> T;

    fn visit_equation(&mut self, equation: Rc<LatexEquation>) -> T;

    fn visit_group(&mut self, group: Rc<LatexGroup>) -> T;

    fn visit_command(&mut self, command: Rc<LatexCommand>) -> T;

    fn visit_text(&mut self, text: Rc<LatexText>) -> T;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironment {
    pub range: Range,
    pub left: LatexEnvironmentDelimiter,
    pub children: Vec<LatexNode>,
    pub right: Option<LatexEnvironmentDelimiter>,
}

impl LatexEnvironment {
    pub fn new(
        left: LatexEnvironmentDelimiter,
        children: Vec<LatexNode>,
        right: Option<LatexEnvironmentDelimiter>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.range.end
        } else if !children.is_empty() {
            children[children.len() - 1].end()
        } else {
            left.range.end
        };
        LatexEnvironment {
            range: Range::new(left.range.start, end),
            left,
            children,
            right,
        }
    }
}

impl Node for LatexEnvironment {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentDelimiter {
    pub range: Range,
    pub command: Rc<LatexCommand>,
    pub name: String,
    pub name_range: Range,
}

impl LatexEnvironmentDelimiter {
    pub fn new(command: Rc<LatexCommand>, name: String, name_range: Range) -> Self {
        LatexEnvironmentDelimiter {
            range: command.range,
            command,
            name,
            name_range,
        }
    }
}

impl Node for LatexEnvironmentDelimiter {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexEnvironmentDelimiterKind {
    Begin,
    End,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation {
    pub range: Range,
    pub left: Rc<LatexCommand>,
    pub children: Vec<LatexNode>,
    pub right: Option<Rc<LatexCommand>>,
}

impl LatexEquation {
    pub fn new(
        left: Rc<LatexCommand>,
        children: Vec<LatexNode>,
        right: Option<Rc<LatexCommand>>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.range.end
        } else if !children.is_empty() {
            children[children.len() - 1].end()
        } else {
            left.range.end
        };
        LatexEquation {
            range: Range::new(left.range.start, end),
            left,
            children,
            right,
        }
    }
}

impl Node for LatexEquation {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexGroup {
    pub range: Range,
    pub left: LatexToken,
    pub children: Vec<LatexNode>,
    pub right: Option<LatexToken>,
    pub kind: LatexGroupKind,
}

impl LatexGroup {
    pub fn new(
        left: LatexToken,
        children: Vec<LatexNode>,
        right: Option<LatexToken>,
        kind: LatexGroupKind,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if !children.is_empty() {
            children[children.len() - 1].end()
        } else {
            left.end()
        };
        LatexGroup {
            range: Range::new(left.start(), end),
            left,
            children,
            right,
            kind,
        }
    }
}

impl Node for LatexGroup {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexGroupKind {
    Group,
    Options,
    Inline,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommand {
    pub range: Range,
    pub name: LatexToken,
    pub options: Option<Rc<LatexGroup>>,
    pub args: Vec<Rc<LatexGroup>>,
    pub kind: LatexCommandKind,
}

impl LatexCommand {
    pub fn new(
        name: LatexToken,
        options: Option<Rc<LatexGroup>>,
        args: Vec<Rc<LatexGroup>>,
    ) -> Self {
        let end = if !args.is_empty() {
            args[args.len() - 1].range.end
        } else if let Some(ref options) = options {
            options.range.end
        } else {
            name.end()
        };
        let mut command = LatexCommand {
            range: Range::new(name.start(), end),
            name,
            options,
            args,
            kind: LatexCommandKind::Unknown,
        };
        command.analyze_kind();
        command
    }

    pub fn extract_text(&self, index: usize) -> Option<&LatexText> {
        if self.args.len() > index && self.args[index].children.len() == 1 {
            if let LatexNode::Text(ref text) = self.args[index].children[0] {
                Some(text)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn extract_word(&self, index: usize) -> Option<&LatexToken> {
        let text = self.extract_text(index)?;
        if text.words.len() == 1 {
            Some(&text.words[0])
        } else {
            None
        }
    }

    pub fn extract_content(&self, index: usize) -> Option<String> {
        let mut words = Vec::new();
        let text = self.extract_text(index)?;
        for word in &text.words {
            words.push(word.text().to_owned());
        }
        Some(words.join(" "))
    }

    fn analyze_kind(&mut self) {
        let name = self.name.text();
        if INCLUDE_COMMANDS.contains(&name) {
            if let Some(path) = self.extract_content(0) {
                let kind = if self.name.text() == "\\usepackage" {
                    LatexIncludeKind::Package
                } else if self.name.text() == "\\documentclass" {
                    LatexIncludeKind::Class
                } else {
                    LatexIncludeKind::File
                };
                self.kind = LatexCommandKind::Include(LatexInclude::new(path, kind));
            }
        } else if SECTION_COMMANDS.contains(&name) {
            if let Some(text) = self.extract_content(0) {
                let level = SECTION_COMMANDS.binary_search(&name).unwrap() / 2;
                self.kind = LatexCommandKind::Section(LatexSection::new(text, level));
            }
        } else if CITATION_COMMANDS.contains(&name) {
            if let Some(key) = self.extract_word(0) {
                self.kind = LatexCommandKind::Citation(LatexCitation::new(key.clone()));
            }
        } else if LABEL_DEFINITION_COMMANDS.contains(&name) {
            if let Some(name) = self.extract_word(0) {
                self.kind = LatexCommandKind::Label(LatexLabel::new(
                    name.clone(),
                    LatexLabelKind::Definition,
                ));
            }
        } else if LABEL_REFERENCE_COMMANDS.contains(&name) {
            if let Some(name) = self.extract_word(0) {
                self.kind = LatexCommandKind::Label(LatexLabel::new(
                    name.clone(),
                    LatexLabelKind::Reference,
                ));
            }
        }
    }
}

impl Node for LatexCommand {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexCommandKind {
    Unknown,
    Include(LatexInclude),
    Section(LatexSection),
    Citation(LatexCitation),
    Label(LatexLabel),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInclude {
    pub path: String,
    pub kind: LatexIncludeKind,
}

impl LatexInclude {
    pub fn new(path: String, kind: LatexIncludeKind) -> Self {
        LatexInclude { path, kind }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexIncludeKind {
    Package,
    Class,
    File,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSection {
    pub text: String,
    pub level: usize,
}

impl LatexSection {
    pub fn new(text: String, level: usize) -> Self {
        LatexSection { text, level }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation {
    pub key: LatexToken,
}

impl LatexCitation {
    pub fn new(key: LatexToken) -> Self {
        LatexCitation { key }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabel {
    pub name: LatexToken,
    pub kind: LatexLabelKind,
}

impl LatexLabel {
    pub fn new(name: LatexToken, kind: LatexLabelKind) -> Self {
        LatexLabel { name, kind }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexLabelKind {
    Definition,
    Reference,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexText {
    pub range: Range,
    pub words: Vec<LatexToken>,
}

impl LatexText {
    pub fn new(words: Vec<LatexToken>) -> Self {
        LatexText {
            range: Range::new(words[0].start(), words[words.len() - 1].end()),
            words,
        }
    }
}

impl Node for LatexText {
    fn range(&self) -> Range {
        self.range
    }
}
