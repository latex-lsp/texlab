use crate::range;
use crate::syntax::text::{Span, SyntaxNode};
use lsp_types::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexTokenKind {
    Word,
    Command,
    Math,
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
        LatexToken { span, kind }
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexRoot {
    pub children: Vec<LatexContent>,
}

impl LatexRoot {
    pub fn new(children: Vec<LatexContent>) -> Self {
        LatexRoot { children }
    }
}

impl SyntaxNode for LatexRoot {
    fn range(&self) -> Range {
        if self.children.is_empty() {
            range::create(0, 0, 0, 0)
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
    Group(LatexGroup),
    Command(LatexCommand),
    Text(LatexText),
}

impl LatexContent {
    pub fn accept<'a>(&'a self, visitor: &mut LatexVisitor<'a>) {
        match self {
            LatexContent::Group(group) => visitor.visit_group(group),
            LatexContent::Command(command) => visitor.visit_command(command),
            LatexContent::Text(text) => visitor.visit_text(text),
        }
    }
}

impl SyntaxNode for LatexContent {
    fn range(&self) -> Range {
        match self {
            LatexContent::Group(group) => group.range(),
            LatexContent::Command(command) => command.range(),
            LatexContent::Text(text) => text.range(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexGroupKind {
    Group,
    Options,
    Math,
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

        LatexGroup {
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
    pub options: Option<LatexGroup>,
    pub args: Vec<LatexGroup>,
}

impl LatexCommand {
    pub fn new(name: LatexToken, options: Option<LatexGroup>, args: Vec<LatexGroup>) -> Self {
        let end = if !args.is_empty() {
            args[args.len() - 1].end()
        } else if let Some(ref options) = options {
            options.end()
        } else {
            name.end()
        };

        LatexCommand {
            range: Range::new(name.start(), end),
            name,
            options,
            args,
        }
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

    pub fn extract_content(&self, index: usize) -> Option<String> {
        let mut words = Vec::new();
        let text = self.extract_text(index)?;
        for word in &text.words {
            words.push(word.text().to_owned());
        }
        Some(words.join(" "))
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
        LatexText {
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

pub trait LatexVisitor<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot);

    fn visit_group(&mut self, group: &'a LatexGroup);

    fn visit_command(&mut self, command: &'a LatexCommand);

    fn visit_text(&mut self, text: &'a LatexText);
}

pub struct LatexWalker;

impl LatexWalker {
    pub fn walk_root<'a>(visitor: &mut LatexVisitor<'a>, root: &'a LatexRoot) {
        for child in &root.children {
            child.accept(visitor);
        }
    }

    pub fn walk_group<'a>(visitor: &mut LatexVisitor<'a>, group: &'a LatexGroup) {
        for child in &group.children {
            child.accept(visitor);
        }
    }

    pub fn walk_command<'a>(visitor: &mut LatexVisitor<'a>, command: &'a LatexCommand) {
        if let Some(ref options) = command.options {
            visitor.visit_group(options);
        }

        for arg in &command.args {
            visitor.visit_group(arg);
        }
    }
}
