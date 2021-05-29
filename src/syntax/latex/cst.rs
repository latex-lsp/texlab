use cstree::TextRange;
use itertools::{EitherOrBoth, Itertools};

use crate::syntax::CstNode;

use super::{Language, SyntaxKind::*, SyntaxNode, SyntaxToken};

macro_rules! cst_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone, Copy)]
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

            fn small_range(&self) -> TextRange {
                let full_range = self.syntax().text_range();
                let start = full_range.start();
                let mut token = self.syntax().last_token();
                while let Some(current) = token {
                    if !matches!(current.kind(), WHITESPACE | COMMENT) {
                        return TextRange::new(start, current.text_range().end());
                    }
                    token = current.prev_token();
                }
                TextRange::new(start, start)
            }
        }
    };
}

cst_node!(Text, TEXT);

impl<'a> Text<'a> {
    pub fn words(&self) -> impl Iterator<Item = &'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD)
    }
}

pub trait HasCurly<'a>: CstNode<'a, Lang = Language> {
    fn left_curly(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_CURLY)
    }

    fn right_curly(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_CURLY)
    }

    fn content_text(&self) -> Option<String> {
        self.left_curly()?;
        self.right_curly()?;
        let mut text = String::new();
        for child in self
            .syntax()
            .descendants_with_tokens()
            .filter_map(|child| child.into_token())
            .filter(|token| !matches!(token.kind(), COMMENT))
        {
            text.push_str(child.text());
        }
        let text = text.trim_end();
        let text = text[1..text.len() - 1].trim().to_string();

        Some(text)
    }
}

cst_node!(CurlyGroup, CURLY_GROUP);

impl<'a> HasCurly<'a> for CurlyGroup<'a> {}

impl<'a> CurlyGroup<'a> {}

pub trait HasBrack<'a>: CstNode<'a, Lang = Language> {
    fn left_brack(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACK)
    }

    fn right_brack(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_BRACK)
    }

    fn content_text(&self) -> Option<String> {
        self.left_brack()?;
        self.right_brack()?;
        let mut text = String::new();
        for child in self
            .syntax()
            .descendants_with_tokens()
            .filter_map(|child| child.into_token())
            .filter(|token| !matches!(token.kind(), COMMENT))
        {
            text.push_str(child.text());
        }
        let text = text.trim_end();
        let text = text[1..text.len() - 1].trim().to_string();

        Some(text)
    }
}

cst_node!(BrackGroup, BRACK_GROUP);

impl<'a> BrackGroup<'a> {}

impl<'a> HasBrack<'a> for BrackGroup<'a> {}

pub trait HasParen<'a>: CstNode<'a, Lang = Language> {
    fn left_paren(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_PAREN)
    }

    fn right_paren(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_PAREN)
    }
}

cst_node!(ParenGroup, PAREN_GROUP);

impl<'a> HasParen<'a> for ParenGroup<'a> {}

cst_node!(MixedGroup, MIXED_GROUP);

impl<'a> MixedGroup<'a> {
    pub fn left_delim(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind().into(), L_BRACK | L_PAREN))
    }

    pub fn right_delim(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind().into(), R_BRACK | R_PAREN))
    }
}

cst_node!(CurlyGroupWord, CURLY_GROUP_WORD);

impl<'a> HasCurly<'a> for CurlyGroupWord<'a> {}

impl<'a> CurlyGroupWord<'a> {
    pub fn key(&self) -> Option<Key<'a>> {
        self.syntax().children().find_map(Key::cast)
    }
}

cst_node!(BrackGroupWord, BRACK_GROUP_WORD);

impl<'a> HasBrack<'a> for BrackGroupWord<'a> {}

impl<'a> BrackGroupWord<'a> {
    pub fn key(&self) -> Option<Key<'a>> {
        self.syntax().children().find_map(Key::cast)
    }
}

cst_node!(CurlyGroupWordList, CURLY_GROUP_WORD_LIST);

impl<'a> HasCurly<'a> for CurlyGroupWordList<'a> {}

impl<'a> CurlyGroupWordList<'a> {
    pub fn keys(&self) -> impl Iterator<Item = Key<'a>> {
        self.syntax().children().filter_map(Key::cast)
    }
}

cst_node!(CurlyGroupCommand, CURLY_GROUP_COMMAND);

impl<'a> HasCurly<'a> for CurlyGroupCommand<'a> {}

impl<'a> CurlyGroupCommand<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == GENERIC_COMMAND_NAME)
    }
}

cst_node!(Key, KEY);

impl<'a> Key<'a> {
    pub fn words(&self) -> impl Iterator<Item = &'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD)
    }
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.words()
            .zip_longest(other.words())
            .all(|result| match result {
                EitherOrBoth::Both(left, right) => left.text() == right.text(),
                EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => false,
            })
    }
}

impl<'a> Eq for Key<'a> {}

impl<'a> ToString for Key<'a> {
    fn to_string(&self) -> String {
        self.words().map(|word| word.text()).join(" ")
    }
}

cst_node!(Value, VALUE);

cst_node!(KeyValuePair, KEY_VALUE_PAIR);

impl<'a> KeyValuePair<'a> {
    pub fn key(&self) -> Option<Key<'a>> {
        self.syntax().children().find_map(Key::cast)
    }

    pub fn value(&self) -> Option<Value<'a>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(KeyValueBody, KEY_VALUE_BODY);

impl<'a> KeyValueBody<'a> {
    pub fn pairs(&self) -> impl Iterator<Item = KeyValuePair<'a>> {
        self.syntax().children().filter_map(KeyValuePair::cast)
    }
}

pub trait HasKeyValueBody<'a>: CstNode<'a, Lang = Language> {
    fn body(&self) -> Option<KeyValueBody<'a>> {
        self.syntax().children().find_map(KeyValueBody::cast)
    }
}

cst_node!(CurlyGroupKeyValue, CURLY_GROUP_KEY_VALUE);

impl<'a> HasCurly<'a> for CurlyGroupKeyValue<'a> {}

impl<'a> HasKeyValueBody<'a> for CurlyGroupKeyValue<'a> {}

cst_node!(BrackGroupKeyValue, BRACK_GROUP_KEY_VALUE);

impl<'a> HasBrack<'a> for BrackGroupKeyValue<'a> {}

impl<'a> HasKeyValueBody<'a> for BrackGroupKeyValue<'a> {}

cst_node!(Formula, FORMULA);

cst_node!(GenericCommand, GENERIC_COMMAND);

impl<'a> GenericCommand<'a> {
    pub fn name(&self) -> Option<&'a SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == GENERIC_COMMAND_NAME)
    }
}

cst_node!(Equation, EQUATION);

cst_node!(Begin, BEGIN);

impl<'a> Begin<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn options(&self) -> Option<BrackGroup<'a>> {
        self.syntax().children().find_map(BrackGroup::cast)
    }
}

cst_node!(End, END);

impl<'a> End<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(Environment, ENVIRONMENT);

impl<'a> Environment<'a> {
    pub fn begin(&self) -> Option<Begin<'a>> {
        self.syntax().children().find_map(Begin::cast)
    }

    pub fn end(&self) -> Option<End<'a>> {
        self.syntax().children().find_map(End::cast)
    }
}

cst_node!(
    Section,
    PART,
    CHAPTER,
    SECTION,
    SUBSECTION,
    SUBSUBSECTION,
    PARAGRAPH,
    SUBPARAGRAPH
);

impl<'a> Section<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(EnumItem, ENUM_ITEM);

impl<'a> EnumItem<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn label(&self) -> Option<BrackGroup<'a>> {
        self.syntax().children().find_map(BrackGroup::cast)
    }
}

cst_node!(Caption, CAPTION);

impl<'a> Caption<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn short(&self) -> Option<BrackGroup<'a>> {
        self.syntax().children().find_map(BrackGroup::cast)
    }

    pub fn long(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(Citation, CITATION);

impl<'a> Citation<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn prenote(&self) -> Option<BrackGroup<'a>> {
        self.syntax().children().find_map(BrackGroup::cast)
    }

    pub fn postnote(&self) -> Option<BrackGroup<'a>> {
        self.syntax().children().filter_map(BrackGroup::cast).nth(1)
    }

    pub fn key_list(&self) -> Option<CurlyGroupWordList<'a>> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(
    Include,
    PACKAGE_INCLUDE,
    CLASS_INCLUDE,
    LATEX_INCLUDE,
    BIBLATEX_INCLUDE,
    BIBTEX_INCLUDE,
    GRAPHICS_INCLUDE,
    SVG_INCLUDE,
    INKSCAPE_INCLUDE,
    VERBATIM_INCLUDE
);

impl<'a> Include<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn path_list(&self) -> Option<CurlyGroupWordList<'a>> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(Import, IMPORT);

impl<'a> Import<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn directory(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn file(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }
}

cst_node!(LabelDefinition, LABEL_DEFINITION);

impl<'a> LabelDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(LabelReference, LABEL_REFERENCE);

impl<'a> LabelReference<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name_list(&self) -> Option<CurlyGroupWordList<'a>> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(LabelReferenceRange, LABEL_REFERENCE_RANGE);

impl<'a> LabelReferenceRange<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn from(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn to(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }
}

cst_node!(LabelNumber, LABEL_NUMBER);

impl<'a> LabelNumber<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn text(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(TheoremDefinition, THEOREM_DEFINITION);

impl<'a> TheoremDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn description(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(CommandDefinition, COMMAND_DEFINITION, MATH_OPERATOR);

impl<'a> CommandDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupCommand<'a>> {
        self.syntax().children().find_map(CurlyGroupCommand::cast)
    }

    pub fn implementation(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(AcronymReference, ACRONYM_REFERENCE);

impl<'a> AcronymReference<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }
}

cst_node!(AcronymDefinition, ACRONYM_DEFINITION);

impl<'a> AcronymDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(AcronymDeclaration, ACRONYM_DECLARATION);

impl<'a> AcronymDeclaration<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(ColorDefinition, COLOR_DEFINITION);

impl<'a> ColorDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn model(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }

    pub fn spec(&self) -> Option<CurlyGroup<'a>> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(ColorSetDefinition, COLOR_SET_DEFINITION);

impl<'a> ColorSetDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn model_list(&self) -> Option<CurlyGroupWordList<'a>> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(ColorReference, COLOR_REFERENCE);

impl<'a> ColorReference<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(GlossaryEntryReference, GLOSSARY_ENTRY_REFERENCE);

impl<'a> GlossaryEntryReference<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(GlossaryEntryDefinition, GLOSSARY_ENTRY_DEFINITION);

impl<'a> GlossaryEntryDefinition<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord<'a>> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(TikzLibraryImport, TIKZ_LIBRARY_IMPORT);

impl<'a> TikzLibraryImport<'a> {
    pub fn command(&self) -> Option<&'a SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name_list(&self) -> Option<CurlyGroupWordList<'a>> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}
