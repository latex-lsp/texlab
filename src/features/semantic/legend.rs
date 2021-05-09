use std::ops;

use lsp_types::{SemanticTokenModifier, SemanticTokenType};

macro_rules! define_semantic_token_types {
    ($(($ident:ident, $string:literal)),*$(,)?) => {
        $(pub const $ident: SemanticTokenType = SemanticTokenType::new($string);)*

        pub const SUPPORTED_TYPES: &[SemanticTokenType] = &[
            SemanticTokenType::COMMENT,
            SemanticTokenType::NUMBER,
            $($ident),*
        ];
    };
}

define_semantic_token_types![
    (JUNK, "junk"),
    (PREAMBLE_TYPE, "preambleType"),
    (STRING_TYPE, "stringType"),
    (COMMENT_TYPE, "commentType"),
    (ARTICLE_TYPE, "articleType"),
    (BOOK_TYPE, "bookType"),
    (COLLECTION_TYPE, "collectionType"),
    (PART_TYPE, "partType"),
    (THESIS_TYPE, "thesisType"),
    (MISC_TYPE, "miscType"),
    (UNKNOWN_TYPE, "unknownType"),
    (CURLY, "curly"),
    (BRACK, "brack"),
    (PAREN, "paren"),
    (COMMA, "comma"),
    (ENTRY_KEY, "entryKey"),
    (FIELD, "field"),
    (TEXT, "text"),
    (STRING_NAME, "string"),
    (EQUALITY_SIGN, "equalitySign"),
    (HASH, "hash"),
    (QUOTE, "quote"),
    (GENERIC_COMMAND, "genericCommand"),
];

macro_rules! define_semantic_token_modifiers {
    ($(($ident:ident, $string:literal)),*$(,)?) => {
        $(pub const $ident: SemanticTokenModifier = SemanticTokenModifier::new($string);)*

        pub const SUPPORTED_MODIFIERS: &[SemanticTokenModifier] = &[
            SemanticTokenModifier::DEFAULT_LIBRARY,
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            $($ident),*
        ];
    };
}

define_semantic_token_modifiers![(ITALIC, "italic"),];

pub fn type_index(ty: SemanticTokenType) -> u32 {
    SUPPORTED_TYPES.iter().position(|t| *t == ty).unwrap() as u32
}

#[derive(Default)]
pub struct ModifierSet(pub u32);

impl ops::BitOrAssign<SemanticTokenModifier> for ModifierSet {
    fn bitor_assign(&mut self, rhs: SemanticTokenModifier) {
        let index = SUPPORTED_MODIFIERS
            .iter()
            .position(|modifier| modifier == &rhs)
            .unwrap();
        self.0 |= 1 << index;
    }
}

impl ops::BitOr<SemanticTokenModifier> for ModifierSet {
    type Output = ModifierSet;

    fn bitor(mut self, rhs: SemanticTokenModifier) -> Self::Output {
        self |= rhs;
        self
    }
}
