use crate::range;
use crate::syntax::text::{CharStream, Span};
use lsp_types::{Position, Range};
use std::iter::Peekable;
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

    pub fn range(&self) -> Range {
        self.span.range
    }

    pub fn start(&self) -> Position {
        self.span.start()
    }

    pub fn end(&self) -> Position {
        self.span.end()
    }

    pub fn text(&self) -> &str {
        &self.span.text
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
    pub fn range(&self) -> Range {
        match self {
            LatexNode::Environment(environment) => environment.range,
            LatexNode::Equation(equation) => equation.range,
            LatexNode::Group(group) => group.range,
            LatexNode::Command(command) => command.range,
            LatexNode::Text(text) => text.range,
        }
    }

    pub fn start(&self) -> Position {
        self.range().start
    }

    pub fn end(&self) -> Position {
        self.range().end
    }

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexCommandKind {
    Unknown,
    Include(LatexInclude),
    Section(LatexSection),
    Citation(LatexCitation),
    Label(LatexLabel),
}

pub const INCLUDE_COMMANDS: &'static [&'static str] = &[
    "\\include",
    "\\input",
    "\\bibliography",
    "\\addbibresource",
    "\\usepackage",
    "\\documentclass",
];

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

pub const SECTION_COMMANDS: &'static [&'static str] = &[
    "\\chapter",
    "\\chapter*",
    "\\section",
    "\\section*",
    "\\subsection",
    "\\subsection*",
    "\\subsubsection",
    "\\subsubsection*",
    "\\paragraph",
    "\\paragraph*",
    "\\subparagraph",
    "\\subparagraph*",
];

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

pub const CITATION_COMMANDS: &'static [&'static str] = &[
    "\\cite",
    "\\cite*",
    "\\Cite",
    "\\nocite",
    "\\citet",
    "\\citep",
    "\\citet*",
    "\\citep*",
    "\\citeauthor",
    "\\citeauthor*",
    "\\Citeauthor",
    "\\Citeauthor*",
    "\\citetitle",
    "\\citetitle*",
    "\\citeyear",
    "\\citeyear*",
    "\\citedate",
    "\\citedate*",
    "\\citeurl",
    "\\fullcite",
    "\\citeyearpar",
    "\\citealt",
    "\\citealp",
    "\\citetext",
    "\\parencite",
    "\\parencite*",
    "\\Parencite",
    "\\footcite",
    "\\footfullcite",
    "\\footcitetext",
    "\\textcite",
    "\\Textcite",
    "\\smartcite",
    "\\Smartcite",
    "\\supercite",
    "\\autocite",
    "\\Autocite",
    "\\autocite*",
    "\\Autocite*",
    "\\volcite",
    "\\Volcite",
    "\\pvolcite",
    "\\Pvolcite",
    "\\fvolcite",
    "\\ftvolcite",
    "\\svolcite",
    "\\Svolcite",
    "\\tvolcite",
    "\\Tvolcite",
    "\\avolcite",
    "\\Avolcite",
    "\\notecite",
    "\\notecite",
    "\\pnotecite",
    "\\Pnotecite",
    "\\fnotecite",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation {
    pub key: LatexToken,
}

impl LatexCitation {
    pub fn new(key: LatexToken) -> Self {
        LatexCitation { key }
    }
}

pub const LABEL_DEFINITION_COMMANDS: &'static [&'static str] = &["\\label"];

pub const LABEL_REFERENCE_COMMANDS: &'static [&'static str] = &["\\ref", "\\autoref", "\\eqref"];

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

struct LatexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> From<CharStream<'a>> for LatexLexer<'a> {
    fn from(stream: CharStream<'a>) -> Self {
        LatexLexer { stream }
    }
}

impl<'a> From<&'a str> for LatexLexer<'a> {
    fn from(text: &'a str) -> Self {
        let stream = CharStream::new(text);
        LatexLexer::from(stream)
    }
}

impl<'a> LatexLexer<'a> {
    fn single_char(&mut self, kind: LatexTokenKind) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        LatexToken::new(span, kind)
    }

    fn math(&mut self) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        if self.stream.satifies(|c| *c == '$') {
            self.stream.next();
        }
        let span = self.stream.end_span();
        LatexToken::new(span, LatexTokenKind::Math)
    }

    fn command(&mut self) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        let mut escape = true;
        while self.stream.satifies(|c| is_command_char(*c)) {
            self.stream.next();
            escape = false;
        }

        if let Some(c) = self.stream.peek() {
            if c != '\r' && c != '\n' && (escape || c == '*') {
                self.stream.next();
            }
        }

        let span = self.stream.end_span();
        LatexToken::new(span, LatexTokenKind::Command)
    }

    fn word(&mut self) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        while self.stream.satifies(|c| is_word_char(*c)) {
            self.stream.next();
        }

        let span = self.stream.end_span();
        LatexToken::new(span, LatexTokenKind::Word)
    }
}

impl<'a> Iterator for LatexLexer<'a> {
    type Item = LatexToken;

    fn next(&mut self) -> Option<LatexToken> {
        loop {
            match self.stream.peek() {
                Some('%') => {
                    self.stream.skip_rest_of_line();
                }
                Some('{') => {
                    return Some(self.single_char(LatexTokenKind::BeginGroup));
                }
                Some('}') => {
                    return Some(self.single_char(LatexTokenKind::EndGroup));
                }
                Some('[') => {
                    return Some(self.single_char(LatexTokenKind::BeginOptions));
                }
                Some(']') => {
                    return Some(self.single_char(LatexTokenKind::EndOptions));
                }
                Some('$') => {
                    return Some(self.math());
                }
                Some('\\') => {
                    return Some(self.command());
                }
                Some(c) => {
                    if c.is_whitespace() {
                        self.stream.next();
                    } else {
                        return Some(self.word());
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }
}

fn is_command_char(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '@'
}

fn is_word_char(c: char) -> bool {
    !c.is_whitespace()
        && c != '%'
        && c != '{'
        && c != '}'
        && c != '['
        && c != ']'
        && c != '\\'
        && c != '$'
}

struct LatexParser<I: Iterator<Item = LatexToken>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = LatexToken>> LatexParser<I> {
    fn new(tokens: I) -> Self {
        LatexParser {
            tokens: tokens.peekable(),
        }
    }

    fn root(&mut self) -> LatexRoot {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word
                | LatexTokenKind::BeginOptions
                | LatexTokenKind::EndOptions => {
                    children.push(LatexNode::Text(self.text(false)));
                }
                LatexTokenKind::Command => children.push(self.command_environment_equation()),
                LatexTokenKind::Math => children.push(LatexNode::Group(self.inline())),
                LatexTokenKind::BeginGroup => children.push(LatexNode::Group(self.group())),
                LatexTokenKind::EndGroup => {
                    self.tokens.next();
                }
            }
        }
        LatexRoot::new(children)
    }

    fn command_environment_equation(&mut self) -> LatexNode {
        let command = self.command();
        if let Some((name, name_range)) =
            test_environment_delimiter(&command, LatexEnvironmentDelimiterKind::Begin)
        {
            let left = LatexEnvironmentDelimiter::new(command, name, name_range);
            LatexNode::Environment(self.environment(left))
        } else if command.name.text() == "\\[" {
            LatexNode::Equation(self.equation(command))
        } else {
            LatexNode::Command(command)
        }
    }

    fn command(&mut self) -> Rc<LatexCommand> {
        let name = self.tokens.next().unwrap();
        let options = if self.next_of_kind(LatexTokenKind::BeginOptions) {
            Some(self.options())
        } else {
            None
        };

        let mut args = Vec::new();
        while self.next_of_kind(LatexTokenKind::BeginGroup) {
            args.push(self.group());
        }

        Rc::new(LatexCommand::new(name, options, args))
    }

    fn environment(&mut self, left: LatexEnvironmentDelimiter) -> Rc<LatexEnvironment> {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word
                | LatexTokenKind::BeginOptions
                | LatexTokenKind::EndOptions => {
                    children.push(LatexNode::Text(self.text(false)));
                }
                LatexTokenKind::Command => {
                    let node = self.command_environment_equation();
                    if let LatexNode::Command(command) = node {
                        if let Some((name, name_range)) =
                            test_environment_delimiter(&command, LatexEnvironmentDelimiterKind::End)
                        {
                            let right = LatexEnvironmentDelimiter::new(command, name, name_range);
                            return Rc::new(LatexEnvironment::new(left, children, Some(right)));
                        } else {
                            children.push(LatexNode::Command(command));
                        }
                    } else {
                        children.push(node);
                    }
                }
                LatexTokenKind::Math => children.push(LatexNode::Group(self.inline())),
                LatexTokenKind::BeginGroup => {
                    children.push(LatexNode::Group(self.group()));
                }
                LatexTokenKind::EndGroup => break,
            }
        }
        Rc::new(LatexEnvironment::new(left, children, None))
    }

    fn equation(&mut self, left: Rc<LatexCommand>) -> Rc<LatexEquation> {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word
                | LatexTokenKind::BeginOptions
                | LatexTokenKind::EndOptions => {
                    children.push(LatexNode::Text(self.text(false)));
                }
                LatexTokenKind::Command => {
                    let node = self.command_environment_equation();
                    if let LatexNode::Command(command) = node {
                        if command.name.text() == "\\]" {
                            return Rc::new(LatexEquation::new(left, children, Some(command)));
                        } else {
                            children.push(LatexNode::Command(command));
                        }
                    } else {
                        children.push(node);
                    }
                }
                LatexTokenKind::Math => children.push(LatexNode::Group(self.inline())),
                LatexTokenKind::BeginGroup => {
                    children.push(LatexNode::Group(self.group()));
                }
                LatexTokenKind::EndGroup => break,
            }
        }
        Rc::new(LatexEquation::new(left, children, None))
    }

    fn inline(&mut self) -> Rc<LatexGroup> {
        let left = self.tokens.next().unwrap();
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word
                | LatexTokenKind::BeginOptions
                | LatexTokenKind::EndOptions => {
                    children.push(LatexNode::Text(self.text(false)));
                }
                LatexTokenKind::Command => children.push(self.command_environment_equation()),
                LatexTokenKind::Math => break,
                LatexTokenKind::BeginGroup => {
                    children.push(LatexNode::Group(self.group()));
                }
                LatexTokenKind::EndGroup => break,
            }
        }

        let right = if self.next_of_kind(LatexTokenKind::Math) {
            self.tokens.next()
        } else {
            None
        };

        Rc::new(LatexGroup::new(
            left,
            children,
            right,
            LatexGroupKind::Inline,
        ))
    }

    fn group(&mut self) -> Rc<LatexGroup> {
        let left = self.tokens.next().unwrap();

        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word
                | LatexTokenKind::BeginOptions
                | LatexTokenKind::EndOptions => children.push(LatexNode::Text(self.text(false))),
                LatexTokenKind::Command => children.push(self.command_environment_equation()),
                LatexTokenKind::Math => children.push(LatexNode::Group(self.inline())),
                LatexTokenKind::BeginGroup => {
                    children.push(LatexNode::Group(self.group()));
                }
                LatexTokenKind::EndGroup => break,
            }
        }

        let right = if self.next_of_kind(LatexTokenKind::EndGroup) {
            self.tokens.next()
        } else {
            None
        };

        Rc::new(LatexGroup::new(
            left,
            children,
            right,
            LatexGroupKind::Group,
        ))
    }

    fn options(&mut self) -> Rc<LatexGroup> {
        let left = self.tokens.next().unwrap();

        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word | LatexTokenKind::BeginOptions => {
                    children.push(LatexNode::Text(self.text(true)));
                }
                LatexTokenKind::Command => children.push(self.command_environment_equation()),
                LatexTokenKind::Math => children.push(LatexNode::Group(self.inline())),
                LatexTokenKind::BeginGroup => children.push(LatexNode::Group(self.group())),
                LatexTokenKind::EndGroup | LatexTokenKind::EndOptions => break,
            }
        }

        let right = if self.next_of_kind(LatexTokenKind::EndOptions) {
            self.tokens.next()
        } else {
            None
        };

        Rc::new(LatexGroup::new(
            left,
            children,
            right,
            LatexGroupKind::Options,
        ))
    }

    fn text(&mut self, options: bool) -> Rc<LatexText> {
        let mut words = Vec::new();
        words.push(self.tokens.next().unwrap());
        while let Some(ref token) = self.tokens.peek() {
            if token.kind == LatexTokenKind::Word
                || (token.kind == LatexTokenKind::EndOptions && !options)
            {
                words.push(self.tokens.next().unwrap());
            } else {
                break;
            }
        }
        Rc::new(LatexText::new(words))
    }

    fn next_of_kind(&mut self, kind: LatexTokenKind) -> bool {
        if let Some(ref token) = self.tokens.peek() {
            token.kind == kind
        } else {
            false
        }
    }
}

fn test_environment_delimiter(
    command: &LatexCommand,
    kind: LatexEnvironmentDelimiterKind,
) -> Option<(String, Range)> {
    let name = if kind == LatexEnvironmentDelimiterKind::Begin {
        "\\begin"
    } else {
        "\\end"
    };

    if command.name.text() != name {
        return None;
    }

    if let Some(name) = command.extract_word(0) {
        Some((name.text().to_owned(), name.range()))
    } else if command.args[0].children.is_empty() {
        let name_position = command.args[0].left.end();
        let name_range = Range::new(name_position, name_position);
        Some((String::new(), name_range))
    } else {
        None
    }
}

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
