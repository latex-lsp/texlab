use itertools::{EitherOrBoth, Itertools};
use rowan::{ast::AstNode, TextRange};

use super::{
    LatexLanguage,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};

pub fn small_range(node: &dyn AstNode<Language = LatexLanguage>) -> TextRange {
    let full_range = node.syntax().text_range();
    let start = full_range.start();
    let mut token = node.syntax().last_token();
    while let Some(current) = token {
        if !matches!(current.kind(), WHITESPACE | COMMENT) {
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
            type Language = LatexLanguage;

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

cst_node!(Text, TEXT);

impl Text {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD)
    }
}

pub trait HasCurly: AstNode<Language = LatexLanguage> {
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

impl HasCurly for CurlyGroup {}

impl CurlyGroup {}

pub trait HasBrack: AstNode<Language = LatexLanguage> {
    fn left_brack(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACK)
    }

    fn right_brack(&self) -> Option<SyntaxToken> {
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

impl BrackGroup {}

impl HasBrack for BrackGroup {}

pub trait HasParen: AstNode<Language = LatexLanguage> {
    fn left_paren(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_PAREN)
    }

    fn right_paren(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_PAREN)
    }
}

cst_node!(ParenGroup, PAREN_GROUP);

impl HasParen for ParenGroup {}

cst_node!(MixedGroup, MIXED_GROUP);

impl MixedGroup {
    pub fn left_delim(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), L_BRACK | L_PAREN))
    }

    pub fn right_delim(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), R_BRACK | R_PAREN))
    }
}

cst_node!(CurlyGroupWord, CURLY_GROUP_WORD);

impl HasCurly for CurlyGroupWord {}

impl CurlyGroupWord {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }
}

cst_node!(BrackGroupWord, BRACK_GROUP_WORD);

impl HasBrack for BrackGroupWord {}

impl BrackGroupWord {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }
}

cst_node!(CurlyGroupWordList, CURLY_GROUP_WORD_LIST);

impl HasCurly for CurlyGroupWordList {}

impl CurlyGroupWordList {
    pub fn keys(&self) -> impl Iterator<Item = Key> {
        self.syntax().descendants().filter_map(Key::cast)
    }
}

cst_node!(CurlyGroupCommand, CURLY_GROUP_COMMAND);

impl HasCurly for CurlyGroupCommand {}

impl CurlyGroupCommand {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == COMMAND_NAME)
    }
}

cst_node!(Key, KEY);

impl Key {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| !matches!(node.kind(), WHITESPACE | COMMENT))
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
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for token in self
            .syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
        {
            if matches!(token.kind(), WHITESPACE | COMMENT) {
                buf.push(' ');
            } else {
                buf.push_str(token.text());
            }
        }

        buf = String::from(buf.trim());
        buf
    }
}

cst_node!(Value, VALUE);

impl Value {
    pub fn text(&self) -> Option<String> {
        match CurlyGroup::cast(self.syntax().clone()) {
            Some(group) => group.content_text(),
            None => Some(self.syntax().text().to_string()),
        }
    }
}

cst_node!(KeyValuePair, KEY_VALUE_PAIR);

impl KeyValuePair {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }

    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(KeyValueBody, KEY_VALUE_BODY);

impl KeyValueBody {
    pub fn pairs(&self) -> impl Iterator<Item = KeyValuePair> {
        self.syntax().children().filter_map(KeyValuePair::cast)
    }
}

pub trait HasKeyValueBody: AstNode<Language = LatexLanguage> {
    fn body(&self) -> Option<KeyValueBody> {
        self.syntax().children().find_map(KeyValueBody::cast)
    }
}

cst_node!(CurlyGroupKeyValue, CURLY_GROUP_KEY_VALUE);

impl HasCurly for CurlyGroupKeyValue {}

impl HasKeyValueBody for CurlyGroupKeyValue {}

cst_node!(BrackGroupKeyValue, BRACK_GROUP_KEY_VALUE);

impl HasBrack for BrackGroupKeyValue {}

impl HasKeyValueBody for BrackGroupKeyValue {}

cst_node!(Formula, FORMULA);

cst_node!(GenericCommand, GENERIC_COMMAND);

impl GenericCommand {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == COMMAND_NAME)
    }
}

cst_node!(Equation, EQUATION);

cst_node!(Begin, BEGIN);

impl Begin {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn options(&self) -> Option<BrackGroup> {
        self.syntax().children().find_map(BrackGroup::cast)
    }
}

cst_node!(End, END);

impl End {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(Environment, ENVIRONMENT);

impl Environment {
    pub fn begin(&self) -> Option<Begin> {
        self.syntax().children().find_map(Begin::cast)
    }

    pub fn end(&self) -> Option<End> {
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

impl Section {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(EnumItem, ENUM_ITEM);

impl EnumItem {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn label(&self) -> Option<BrackGroup> {
        self.syntax().children().find_map(BrackGroup::cast)
    }
}

cst_node!(Caption, CAPTION);

impl Caption {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn short(&self) -> Option<BrackGroup> {
        self.syntax().children().find_map(BrackGroup::cast)
    }

    pub fn long(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(Citation, CITATION);

impl Citation {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn prenote(&self) -> Option<BrackGroup> {
        self.syntax().children().find_map(BrackGroup::cast)
    }

    pub fn postnote(&self) -> Option<BrackGroup> {
        self.syntax().children().filter_map(BrackGroup::cast).nth(1)
    }

    pub fn key_list(&self) -> Option<CurlyGroupWordList> {
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

impl Include {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn path_list(&self) -> Option<CurlyGroupWordList> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(Import, IMPORT);

impl Import {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn directory(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn file(&self) -> Option<CurlyGroupWord> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }
}

cst_node!(LabelDefinition, LABEL_DEFINITION);

impl LabelDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(LabelReference, LABEL_REFERENCE);

impl LabelReference {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name_list(&self) -> Option<CurlyGroupWordList> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(LabelReferenceRange, LABEL_REFERENCE_RANGE);

impl LabelReferenceRange {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn from(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn to(&self) -> Option<CurlyGroupWord> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }
}

cst_node!(LabelNumber, LABEL_NUMBER);

impl LabelNumber {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn text(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(
    TheoremDefinition,
    THEOREM_DEFINITION_AMSTHM,
    THEOREM_DEFINITION_THMTOOLS
);

impl TheoremDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn names(&self) -> impl Iterator<Item = Key> {
        self.syntax()
            .children()
            .find_map(CurlyGroupWordList::cast)
            .into_iter()
            .flat_map(|group| group.keys())
            .chain(
                self.syntax()
                    .children()
                    .find_map(CurlyGroupWord::cast)
                    .into_iter()
                    .filter_map(|group| group.key()),
            )
    }

    pub fn heading(&self) -> Option<String> {
        if self.0.kind() == THEOREM_DEFINITION_THMTOOLS {
            let options = self
                .syntax()
                .children()
                .find_map(BrackGroupKeyValue::cast)
                .and_then(|group| group.body())?;

            options
                .pairs()
                .find(|pair| pair.key().map_or(false, |key| key.to_string() == "name"))
                .and_then(|pair| pair.value())
                .and_then(|name| name.text())
        } else {
            self.syntax()
                .children()
                .find_map(CurlyGroup::cast)
                .and_then(|group| group.content_text())
        }
    }
}

cst_node!(OldCommandDefinition, OLD_COMMAND_DEFINITION);

impl OldCommandDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .skip(1)
            .filter_map(|elem| elem.into_token())
            .find(|token| token.kind() == COMMAND_NAME)
    }
}

cst_node!(NewCommandDefinition, NEW_COMMAND_DEFINITION, MATH_OPERATOR);

impl NewCommandDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupCommand> {
        self.syntax().children().find_map(CurlyGroupCommand::cast)
    }

    pub fn implementation(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(AcronymReference, ACRONYM_REFERENCE);

impl AcronymReference {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }
}

cst_node!(AcronymDefinition, ACRONYM_DEFINITION);

impl AcronymDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(AcronymDeclaration, ACRONYM_DECLARATION);

impl AcronymDeclaration {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(ColorDefinition, COLOR_DEFINITION);

impl ColorDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn model(&self) -> Option<CurlyGroupWord> {
        self.syntax()
            .children()
            .filter_map(CurlyGroupWord::cast)
            .nth(1)
    }

    pub fn spec(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}

cst_node!(ColorSetDefinition, COLOR_SET_DEFINITION);

impl ColorSetDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn model_list(&self) -> Option<CurlyGroupWordList> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(ColorReference, COLOR_REFERENCE);

impl ColorReference {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(GlossaryEntryReference, GLOSSARY_ENTRY_REFERENCE);

impl GlossaryEntryReference {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(GlossaryEntryDefinition, GLOSSARY_ENTRY_DEFINITION);

impl GlossaryEntryDefinition {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(TikzLibraryImport, TIKZ_LIBRARY_IMPORT);

impl TikzLibraryImport {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name_list(&self) -> Option<CurlyGroupWordList> {
        self.syntax().children().find_map(CurlyGroupWordList::cast)
    }
}

cst_node!(GraphicsPath, GRAPHICS_PATH);

impl GraphicsPath {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn path_list(&self) -> impl Iterator<Item = CurlyGroupWord> {
        self.syntax().descendants().filter_map(CurlyGroupWord::cast)
    }
}

cst_node!(BibItem, BIBITEM);

impl BibItem {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(TocContentsLine, TOC_CONTENTS_LINE);

impl TocContentsLine {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn unit(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }

    pub fn line(&self) -> Option<CurlyGroup> {
        self.syntax().children().filter_map(CurlyGroup::cast).nth(1)
    }
}

cst_node!(TocNumberLine, TOC_NUMBER_LINE);

impl TocNumberLine {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn number(&self) -> Option<CurlyGroup> {
        self.syntax().children().find_map(CurlyGroup::cast)
    }
}
