use super::ast::*;
use std::iter::Peekable;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LatexScope {
    Root,
    Group,
    Options,
}

pub struct LatexParser<I: Iterator<Item = LatexToken>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = LatexToken>> LatexParser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn root(&mut self) -> LatexRoot {
        let children = self.content(LatexScope::Root);
        LatexRoot::new(children)
    }

    fn content(&mut self, scope: LatexScope) -> Vec<LatexContent> {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word | LatexTokenKind::BeginOptions => {
                    children.push(LatexContent::Text(self.text(scope)));
                }
                LatexTokenKind::Command => {
                    children.push(LatexContent::Command(self.command()));
                }
                LatexTokenKind::Comma => {
                    let node = LatexComma::new(self.tokens.next().unwrap());
                    children.push(LatexContent::Comma(Arc::new(node)));
                }
                LatexTokenKind::Math => {
                    children.push(LatexContent::Math(self.math()));
                }
                LatexTokenKind::BeginGroup => {
                    children.push(LatexContent::Group(self.group(LatexGroupKind::Group)));
                }
                LatexTokenKind::EndGroup => {
                    if scope == LatexScope::Root {
                        self.tokens.next();
                    } else {
                        return children;
                    }
                }
                LatexTokenKind::EndOptions => {
                    if scope == LatexScope::Options {
                        return children;
                    } else {
                        children.push(LatexContent::Text(self.text(scope)));
                    }
                }
            }
        }
        children
    }

    fn command(&mut self) -> Arc<LatexCommand> {
        let name = self.tokens.next().unwrap();

        let mut options = Vec::new();
        let mut args = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::BeginGroup => {
                    args.push(self.group(LatexGroupKind::Group));
                }
                LatexTokenKind::BeginOptions => {
                    options.push(self.group(LatexGroupKind::Options));
                }
                _ => {
                    break;
                }
            }
        }
        Arc::new(LatexCommand::new(name, options, args))
    }

    fn group(&mut self, kind: LatexGroupKind) -> Arc<LatexGroup> {
        let left = self.tokens.next().unwrap();
        let scope = match kind {
            LatexGroupKind::Group => LatexScope::Group,
            LatexGroupKind::Options => LatexScope::Options,
        };
        let children = self.content(scope);
        let right_kind = match kind {
            LatexGroupKind::Group => LatexTokenKind::EndGroup,
            LatexGroupKind::Options => LatexTokenKind::EndOptions,
        };

        let right = if self.next_of_kind(right_kind) {
            self.tokens.next()
        } else {
            None
        };

        Arc::new(LatexGroup::new(left, children, right, kind))
    }

    fn text(&mut self, scope: LatexScope) -> Arc<LatexText> {
        let mut words = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            let kind = token.kind;
            let opts = kind == LatexTokenKind::EndOptions && scope != LatexScope::Options;
            if kind == LatexTokenKind::Word || kind == LatexTokenKind::BeginOptions || opts {
                words.push(self.tokens.next().unwrap());
            } else {
                break;
            }
        }
        Arc::new(LatexText::new(words))
    }

    fn math(&mut self) -> Arc<LatexMath> {
        let token = self.tokens.next().unwrap();
        Arc::new(LatexMath::new(token))
    }

    fn next_of_kind(&mut self, kind: LatexTokenKind) -> bool {
        if let Some(ref token) = self.tokens.peek() {
            token.kind == kind
        } else {
            false
        }
    }
}
