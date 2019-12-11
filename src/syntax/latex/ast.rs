use texlab_protocol::RangeExt;
use crate::syntax::text::{Span, SyntaxNode};
use itertools::Itertools;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexTokenKind {
    Word,
    Command,
    Math,
    Comma,
    BeginGroup,
    EndGroup,
    BeginOptions,
    EndOptions,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexToken {
    pub span: Span,
    pub kind: LatexTokenKind,
}

impl LatexToken {
    pub fn new(span: Span, kind: LatexTokenKind) -> Self {
        Self { span, kind }
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

impl SyntaxNode for LatexToken {
    fn range(&self) -> Range {
        self.span.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexRoot {
    pub children: Vec<LatexContent>,
}

impl LatexRoot {
    pub fn new(children: Vec<LatexContent>) -> Self {
        Self { children }
    }
}

impl SyntaxNode for LatexRoot {
    fn range(&self) -> Range {
        if self.children.is_empty() {
            Range::new_simple(0, 0, 0, 0)
        } else {
            Range::new(
                self.children[0].start(),
                self.children[self.children.len() - 1].end(),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexContent {
    Group(Arc<LatexGroup>),
    Command(Arc<LatexCommand>),
    Text(Arc<LatexText>),
    Comma(Arc<LatexComma>),
    Math(Arc<LatexMath>),
}

impl LatexContent {
    pub fn accept<T: LatexVisitor>(&self, visitor: &mut T) {
        match self {
            LatexContent::Group(group) => visitor.visit_group(Arc::clone(&group)),
            LatexContent::Command(command) => visitor.visit_command(Arc::clone(&command)),
            LatexContent::Text(text) => visitor.visit_text(Arc::clone(&text)),
            LatexContent::Comma(comma) => visitor.visit_comma(Arc::clone(&comma)),
            LatexContent::Math(math) => visitor.visit_math(Arc::clone(&math)),
        }
    }
}

impl SyntaxNode for LatexContent {
    fn range(&self) -> Range {
        match self {
            LatexContent::Group(group) => group.range(),
            LatexContent::Command(command) => command.range(),
            LatexContent::Text(text) => text.range(),
            LatexContent::Comma(comma) => comma.range(),
            LatexContent::Math(math) => math.range(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexGroupKind {
    Group,
    Options,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexGroup {
    pub range: Range,
    pub left: LatexToken,
    pub children: Vec<LatexContent>,
    pub right: Option<LatexToken>,
    pub kind: LatexGroupKind,
}

impl LatexGroup {
    pub fn new(
        left: LatexToken,
        children: Vec<LatexContent>,
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

        Self {
            range: Range::new(left.start(), end),
            left,
            children,
            right,
            kind,
        }
    }
}

impl SyntaxNode for LatexGroup {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommand {
    pub range: Range,
    pub name: LatexToken,
    pub options: Vec<Arc<LatexGroup>>,
    pub args: Vec<Arc<LatexGroup>>,
    pub groups: Vec<Arc<LatexGroup>>,
}

impl LatexCommand {
    pub fn new(
        name: LatexToken,
        options: Vec<Arc<LatexGroup>>,
        args: Vec<Arc<LatexGroup>>,
    ) -> Self {
        let groups: Vec<Arc<LatexGroup>> = args
            .iter()
            .chain(options.iter())
            .sorted_by_key(|group| group.range.start)
            .map(Arc::clone)
            .collect();

        let end = if let Some(group) = groups.last() {
            group.end()
        } else {
            name.end()
        };

        Self {
            range: Range::new(name.start(), end),
            name,
            options,
            args,
            groups,
        }
    }

    pub fn short_name_range(&self) -> Range {
        Range::new_simple(
            self.name.start().line,
            self.name.start().character + 1,
            self.name.end().line,
            self.name.end().character,
        )
    }

    pub fn extract_text(&self, index: usize) -> Option<&LatexText> {
        if self.args.len() > index && self.args[index].children.len() == 1 {
            if let LatexContent::Text(ref text) = self.args[index].children[0] {
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

    pub fn has_word(&self, index: usize) -> bool {
        self.extract_word(index).is_some()
    }

    pub fn extract_comma_separated_words(&self, index: usize) -> Vec<&LatexToken> {
        let mut words = Vec::new();
        for child in &self.args[index].children {
            if let LatexContent::Text(text) = child {
                for word in &text.words {
                    words.push(word);
                }
            }
        }
        words
    }

    pub fn has_comma_separated_words(&self, index: usize) -> bool {
        if self.args.len() <= index {
            return false;
        }

        for node in &self.args[index].children {
            match node {
                LatexContent::Text(_) | LatexContent::Comma(_) => (),
                LatexContent::Command(_) | LatexContent::Group(_) | LatexContent::Math(_) => {
                    return false;
                }
            }
        }
        true
    }
}

impl SyntaxNode for LatexCommand {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexText {
    pub range: Range,
    pub words: Vec<LatexToken>,
}

impl LatexText {
    pub fn new(words: Vec<LatexToken>) -> Self {
        Self {
            range: Range::new(words[0].start(), words[words.len() - 1].end()),
            words,
        }
    }
}

impl SyntaxNode for LatexText {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexComma {
    pub token: LatexToken,
}

impl LatexComma {
    pub fn new(token: LatexToken) -> Self {
        Self { token }
    }
}

impl SyntaxNode for LatexComma {
    fn range(&self) -> Range {
        self.token.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexMath {
    pub token: LatexToken,
}

impl LatexMath {
    pub fn new(token: LatexToken) -> Self {
        Self { token }
    }
}

impl SyntaxNode for LatexMath {
    fn range(&self) -> Range {
        self.token.range()
    }
}

pub trait LatexVisitor {
    fn visit_root(&mut self, root: Arc<LatexRoot>);

    fn visit_group(&mut self, group: Arc<LatexGroup>);

    fn visit_command(&mut self, command: Arc<LatexCommand>);

    fn visit_text(&mut self, text: Arc<LatexText>);

    fn visit_comma(&mut self, comma: Arc<LatexComma>);

    fn visit_math(&mut self, math: Arc<LatexMath>);
}

pub struct LatexWalker;

impl LatexWalker {
    pub fn walk_root<T: LatexVisitor>(visitor: &mut T, root: Arc<LatexRoot>) {
        for child in &root.children {
            child.accept(visitor);
        }
    }

    pub fn walk_group<T: LatexVisitor>(visitor: &mut T, group: Arc<LatexGroup>) {
        for child in &group.children {
            child.accept(visitor);
        }
    }

    pub fn walk_command<T: LatexVisitor>(visitor: &mut T, command: Arc<LatexCommand>) {
        for arg in &command.groups {
            visitor.visit_group(Arc::clone(&arg));
        }
    }

    pub fn walk_text<T: LatexVisitor>(_visitor: &mut T, _text: Arc<LatexText>) {}

    pub fn walk_comma<T: LatexVisitor>(_visitor: &mut T, _comma: Arc<LatexComma>) {}

    pub fn walk_math<T: LatexVisitor>(_visitor: &mut T, _math: Arc<LatexMath>) {}
}
