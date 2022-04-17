use itertools::{EitherOrBoth, Itertools};
use rowan::{ast::AstNode, TextRange};

use super::{
    BibtexLanguage,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};

pub fn small_range(node: &dyn AstNode<Language = BibtexLanguage>) -> TextRange {
    let full_range = node.syntax().text_range();
    let start = full_range.start();
    let mut token = node.syntax().last_token();
    while let Some(current) = token {
        if !matches!(current.kind(), WHITESPACE | JUNK) {
            return TextRange::new(start, current.text_range().end());
        }
        token = current.prev_token();
    }
    TextRange::new(start, start)
}

macro_rules! cst_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Language = BibtexLanguage;

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

pub trait HasCurly: AstNode<Language = BibtexLanguage> {
    fn left_curly(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_CURLY)
    }

    fn right_curly(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_CURLY)
    }
}

pub trait HasQuotes: AstNode<Language = BibtexLanguage> {
    fn left_quote(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == QUOTE)
    }

    fn right_quote(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == QUOTE)
            .nth(1)
    }
}

pub trait HasDelimiters: AstNode<Language = BibtexLanguage> {
    fn left_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), L_CURLY | L_PAREN))
    }

    fn right_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), R_CURLY | R_PAREN))
    }
}

pub trait HasType: AstNode<Language = BibtexLanguage> {
    fn ty(&self) -> Option<SyntaxToken> {
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

impl HasType for Comment {}

cst_node!(Preamble, PREAMBLE);

impl HasType for Preamble {}

impl HasDelimiters for Preamble {}

impl Preamble {
    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(String, STRING);

impl HasType for String {}

impl HasDelimiters for String {}

impl String {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn equality_sign(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == EQUALITY_SIGN)
    }

    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Entry, ENTRY);

impl HasType for Entry {}

impl HasDelimiters for Entry {}

impl Entry {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }

    pub fn fields(&self) -> impl Iterator<Item = Field> {
        self.syntax().children().filter_map(Field::cast)
    }
}

cst_node!(Key, KEY);

impl Key {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.words()
            .zip_longest(other.words())
            .all(|result| match result {
                EitherOrBoth::Both(left, right) => left.text() == right.text(),
                EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => false,
            })
    }
}

impl Eq for Key {}

impl ToString for Key {
    fn to_string(&self) -> std::string::String {
        let mut buf = std::string::String::new();
        for word in self.words() {
            buf.push_str(word.text());
            buf.push(' ');
        }

        buf.pop().unwrap();
        buf
    }
}

cst_node!(Field, FIELD);

impl Field {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn equality_sign(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == EQUALITY_SIGN)
    }

    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Value, VALUE);

impl Value {
    pub fn tokens(&self) -> impl Iterator<Item = Token> {
        self.syntax().children().filter_map(Token::cast)
    }
}

cst_node!(Token, TOKEN);

cst_node!(BraceGroup, BRACE_GROUP);

impl HasCurly for BraceGroup {}

cst_node!(QuoteGroup, QUOTE_GROUP);

impl HasQuotes for QuoteGroup {}
