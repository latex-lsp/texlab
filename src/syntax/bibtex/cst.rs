use crate::syntax::CstNode;

use super::{Language, SyntaxKind::*, SyntaxNode, SyntaxToken};

macro_rules! cst_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        pub struct $name<'a>(&'a SyntaxNode);

        impl<'a> CstNode<'a> for $name<'a> {
            type Lang = Language;

            fn cast(node: &'a cstree::ResolvedNode<Self::Lang>) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &'a cstree::ResolvedNode<Self::Lang> {
                &self.0
            }
        }
    };
}

pub trait HasCurly<'a>: CstNode<'a, Lang = Language> {
    fn left_curly(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_CURLY.into())
    }

    fn right_curly(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_CURLY.into())
    }
}

pub trait HasQuotes<'a>: CstNode<'a, Lang = Language> {
    fn left_quote(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == QUOTE.into())
    }

    fn right_quote(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == QUOTE.into())
            .skip(1)
            .next()
    }
}

pub trait HasDelimiters<'a>: CstNode<'a, Lang = Language> {
    fn left_delimiter(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), L_CURLY | L_PAREN))
    }

    fn right_delimiter(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), R_CURLY | R_PAREN))
    }
}

pub trait HasType<'a>: CstNode<'a, Lang = Language> {
    fn ty(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| {
                matches!(
                    node.kind(),
                    PREAMBLE_TYPE | STRING_TYPE | COMMENT_TYPE | ENTRY_TYPE
                )
            })
    }
}

cst_node!(Root, ROOT);

cst_node!(Junk, JUNK);

cst_node!(Comment, COMMENT);

impl<'a> HasType<'a> for Comment<'a> {}

cst_node!(Preamble, PREAMBLE);

impl<'a> HasType<'a> for Preamble<'a> {}

impl<'a> HasDelimiters<'a> for Preamble<'a> {}

impl<'a> Preamble<'a> {
    pub fn value(&self) -> Option<Value<'a>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(String, STRING);

impl<'a> HasType<'a> for String<'a> {}

impl<'a> HasDelimiters<'a> for String<'a> {}

impl<'a> String<'a> {
    pub fn value(&self) -> Option<Value<'a>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Entry, ENTRY);

impl<'a> HasType<'a> for Entry<'a> {}

impl<'a> HasDelimiters<'a> for Entry<'a> {}

impl<'a> Entry<'a> {
    pub fn key(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn fields(&self) -> impl Iterator<Item = Field<'a>> {
        self.syntax().children().filter_map(Field::cast)
    }
}

cst_node!(Field, FIELD);

impl<'a> Field<'a> {
    pub fn name(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn value(&self) -> Option<Value<'a>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Value, VALUE);

impl<'a> Value<'a> {
    pub fn tokens(&self) -> impl Iterator<Item = Token<'a>> {
        self.syntax().children().filter_map(Token::cast)
    }
}

cst_node!(Token, TOKEN);

cst_node!(BraceGroup, BRACE_GROUP);

impl<'a> HasCurly<'a> for BraceGroup<'a> {}

cst_node!(QuoteGroup, QUOTE_GROUP);

impl<'a> HasQuotes<'a> for QuoteGroup<'a> {}
