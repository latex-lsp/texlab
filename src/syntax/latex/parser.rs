use crate::syntax::latex::ast::*;
use crate::syntax::text::{Node, Span};
use lsp_types::Range;
use std::iter::Peekable;
use std::rc::Rc;

pub struct LatexParser<I: Iterator<Item = LatexToken>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = LatexToken>> LatexParser<I> {
    pub fn new(tokens: I) -> Self {
        LatexParser {
            tokens: tokens.peekable(),
        }
    }

    pub fn root(&mut self) -> LatexRoot {
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
        if let Some(name) =
            test_environment_delimiter(&command, LatexEnvironmentDelimiterKind::Begin)
        {
            let left = LatexEnvironmentDelimiter::new(command, name.text, name.range);
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
                        if let Some(name) =
                            test_environment_delimiter(&command, LatexEnvironmentDelimiterKind::End)
                        {
                            let right =
                                LatexEnvironmentDelimiter::new(command, name.text, name.range);
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
) -> Option<Span> {
    let name = if kind == LatexEnvironmentDelimiterKind::Begin {
        "\\begin"
    } else {
        "\\end"
    };

    if command.name.text() != name {
        return None;
    }

    if let Some(name) = command.extract_word(0) {
        Some(Span::new(name.range(), name.text().to_owned()))
    } else if command.args[0].children.is_empty() {
        let name_position = command.args[0].left.end();
        let name_range = Range::new(name_position, name_position);
        Some(Span::new(name_range, String::new()))
    } else {
        None
    }
}
