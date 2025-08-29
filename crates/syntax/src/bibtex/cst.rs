use rowan::{ast::AstNode, NodeOrToken};

use super::{
    BibtexLanguage,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};

macro_rules! cst_node {
    (name: $name:ident, kinds: [$($kind:pat),+], traits: [$($trait: ident),*]) => {
        #[derive(Clone)]
        pub struct $name {
            node: SyntaxNode,
        }

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
                    $($kind => Some(Self { node}),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.node
            }
        }

        $(
            impl $trait for $name { }
        )*
    };
}

macro_rules! cst_node_enum {
    (name: $name:ident, variants: [$($variant:ident),+]) => {
        #[derive(Clone)]
        pub enum $name {
            $($variant($variant),)*
        }

        impl AstNode for $name {
            type Language = BibtexLanguage;

            fn can_cast(kind: SyntaxKind) -> bool {
                false $(|| $variant::can_cast(kind))+
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                None $(.or_else(|| $variant::cast(node.clone()).map(Self::$variant)))*
            }

            fn syntax(&self) -> &SyntaxNode {
                match self {
                    $(Self::$variant(node) => node.syntax(),)*
                }
            }
        }

        $(
            impl From<$variant> for $name {
                fn from(node: $variant) -> Self {
                    Self::$variant(node)
                }
            }
        )*
    };
}

pub trait HasType: AstNode<Language = BibtexLanguage> {
    fn type_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == TYPE)
    }
}

pub trait HasDelims: AstNode<Language = BibtexLanguage> {
    fn left_delim_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == L_DELIM)
    }

    fn right_delim_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == R_DELIM)
    }
}

pub trait HasName: AstNode<Language = BibtexLanguage> {
    fn name_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == NAME)
    }
}

pub trait HasEq: AstNode<Language = BibtexLanguage> {
    fn eq_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == EQ)
    }
}

pub trait HasComma: AstNode<Language = BibtexLanguage> {
    fn comma_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == COMMA)
    }
}

pub trait HasPound: AstNode<Language = BibtexLanguage> {
    fn pound_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == POUND)
    }
}

pub trait HasInteger: AstNode<Language = BibtexLanguage> {
    fn integer_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == INTEGER)
    }
}

pub trait HasCommandName: AstNode<Language = BibtexLanguage> {
    fn command_name_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == COMMAND_NAME)
    }
}

pub trait HasAccentName: AstNode<Language = BibtexLanguage> {
    fn accent_name_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == ACCENT_NAME)
    }
}

pub trait HasWord: AstNode<Language = BibtexLanguage> {
    fn word_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == WORD)
    }
}

pub trait HasValue: AstNode<Language = BibtexLanguage> {
    fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(name: Root, kinds: [ROOT], traits: []);

impl Root {
    pub fn strings(&self) -> impl Iterator<Item = StringDef> + use<> {
        self.syntax().children().filter_map(StringDef::cast)
    }

    pub fn entries(&self) -> impl Iterator<Item = Entry> + use<> {
        self.syntax().children().filter_map(Entry::cast)
    }

    pub fn find_entry(&self, name: &str) -> Option<Entry> {
        self.entries()
            .find(|entry| entry.name_token().is_some_and(|token| token.text() == name))
    }
}

cst_node!(name: Preamble, kinds: [PREAMBLE], traits: [HasType, HasDelims, HasValue]);

cst_node!(name: StringDef, kinds: [STRING], traits: [HasType, HasDelims, HasName, HasEq, HasValue]);

cst_node!(name: Entry, kinds: [ENTRY], traits: [HasType, HasDelims, HasName, HasComma]);

impl Entry {
    pub fn fields(&self) -> impl Iterator<Item = Field> + use<> {
        self.syntax().children().filter_map(Field::cast)
    }
}

cst_node!(name: Field, kinds: [FIELD], traits: [HasName, HasEq, HasValue, HasComma]);

cst_node_enum!(name: Value, variants: [Literal,  CurlyGroup, QuoteGroup, Join, Accent, Command]);

cst_node!(name: Literal, kinds: [LITERAL], traits: [HasName, HasInteger]);

cst_node!(name: CurlyGroup, kinds: [CURLY_GROUP], traits: []);

cst_node!(name: QuoteGroup, kinds: [QUOTE_GROUP], traits: []);

cst_node!(name: Join, kinds: [JOIN], traits: [HasPound]);

impl Join {
    pub fn left_value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }

    pub fn right_value(&self) -> Option<Value> {
        self.syntax().children().filter_map(Value::cast).nth(1)
    }
}

cst_node!(name: Accent, kinds: [ACCENT], traits: [HasAccentName, HasWord]);

cst_node!(name: Command, kinds: [COMMAND], traits: [HasCommandName]);
