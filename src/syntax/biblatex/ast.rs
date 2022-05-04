use derive_more::From;
use rowan::{ast::AstNode, NodeOrToken, TextRange};

use super::{
    Language,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};

pub fn small_range(node: &dyn AstNode<Language = Language>) -> TextRange {
    let full_range = node.syntax().text_range();
    let start = full_range.start();
    let mut token = node.syntax().last_token();
    while let Some(current) = token {
        if !matches!(current.kind(), WHITESPACE) {
            return TextRange::new(start, current.text_range().end());
        }

        token = current.prev_token();
    }

    TextRange::new(start, start)
}

macro_rules! ast_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Language = Language;

            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    $($kind => true,)+
                    _ => false,
                }
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

macro_rules! ast_node_enum {
    ($name:ident, $first:ident, $($other:ident),*) => {
        #[derive(Clone, From)]
        pub enum $name {
            $first($first),
            $($other($other),)*
        }

        impl AstNode for $name {
            type Language = Language;

            fn can_cast(kind: SyntaxKind) -> bool {
                $first::can_cast(kind) $(|| $other::can_cast(kind))*
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                $first::cast(node.clone()).map(Self::from) $(.or_else(|| $other::cast(node.clone()).map(Self::from)))*
            }

            fn syntax(&self) -> &SyntaxNode {
                match self {
                    Self::$first(node) => node.syntax(),
                    $(Self::$other(node) => node.syntax(),)*
                }
            }
        }
    };
}

pub trait HasType: AstNode<Language = Language> {
    fn type_(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == TYPE)
    }
}

pub trait HasDelimiters: AstNode<Language = Language> {
    fn left_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| matches!(token.kind(), L_CURLY | L_PAREN))
    }

    fn right_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| matches!(token.kind(), R_CURLY | R_PAREN))
    }
}

pub trait HasKey: AstNode<Language = Language> {
    fn key(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| matches!(token.kind(), KEY))
    }
}

pub trait HasEq: AstNode<Language = Language> {
    fn eq(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| matches!(token.kind(), EQ))
    }
}

pub trait HasComma: AstNode<Language = Language> {
    fn comma(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| matches!(token.kind(), COMMA))
    }
}

pub trait HasValue: AstNode<Language = Language> {
    fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

ast_node!(Root, ROOT);

ast_node!(Preamble, PREAMBLE);

impl HasType for Preamble {}

impl HasDelimiters for Preamble {}

impl HasValue for Preamble {}

ast_node!(StringDef, STRING);

impl HasType for StringDef {}

impl HasDelimiters for StringDef {}

impl HasKey for StringDef {}

impl HasEq for StringDef {}

impl HasValue for StringDef {}

ast_node!(Entry, ENTRY);

impl Entry {
    pub fn fields(&self) -> impl Iterator<Item = Field> {
        self.syntax().children().filter_map(Field::cast)
    }
}

impl HasType for Entry {}

impl HasDelimiters for Entry {}

impl HasKey for Entry {}

impl HasComma for Entry {}

ast_node!(Comment, COMMENT);

impl HasType for Comment {}

ast_node!(Field, FIELD);

impl HasKey for Field {}

impl HasEq for Field {}

impl HasComma for Field {}

impl HasValue for Field {}

ast_node_enum!(Value, Concat, CurlyGroup, QuoteGroup, Literal);

ast_node!(Concat, CONCAT);

impl Concat {
    pub fn values(&self) -> impl Iterator<Item = Value> {
        self.syntax().children().filter_map(Value::cast)
    }
}

ast_node!(CurlyGroup, CURLY_GROUP);

impl HasDelimiters for CurlyGroup {}

ast_node!(QuoteGroup, QUOTE_GROUP);

impl HasDelimiters for QuoteGroup {
    fn left_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .first_token()
            .filter(|token| matches!(token.kind(), QUOTE))
    }

    fn right_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .skip(1)
            .find(|token| matches!(token.kind(), QUOTE))
    }
}

ast_node!(Literal, LITERAL);
