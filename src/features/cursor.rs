use lsp_types::{
    CompletionParams, DocumentHighlightParams, GotoDefinitionParams, HoverParams, Position,
    ReferenceParams, RenameParams, TextDocumentPositionParams,
};
use rowan::{ast::AstNode, TextRange, TextSize};

use crate::{
    syntax::{bibtex, latex},
    DocumentData, LineIndexExt,
};

use super::FeatureRequest;

#[derive(Debug)]
pub enum Cursor {
    Latex(latex::SyntaxToken),
    Bibtex(bibtex::SyntaxToken),
    Nothing,
}

impl Cursor {
    pub fn new_latex(
        left: Option<latex::SyntaxToken>,
        right: Option<latex::SyntaxToken>,
    ) -> Option<Self> {
        let left = left?;
        let right = right?;

        if left.kind().is_command_name() {
            return Some(Self::Latex(left));
        }

        if right.kind() == latex::WORD {
            return Some(Self::Latex(right));
        }

        if left.kind() == latex::WORD {
            return Some(Self::Latex(left));
        }

        if right.kind().is_command_name() {
            return Some(Self::Latex(right));
        }

        if left.kind() == latex::WHITESPACE && left.parent()?.kind() == latex::KEY {
            return Some(Self::Latex(left));
        }

        if matches!(right.kind(), latex::WHITESPACE | latex::LINE_BREAK)
            && right.parent()?.kind() == latex::KEY
        {
            return Some(Self::Latex(right));
        }

        Some(Self::Latex(right))
    }

    pub fn new_bibtex(
        left: Option<bibtex::SyntaxToken>,
        right: Option<bibtex::SyntaxToken>,
    ) -> Option<Self> {
        let left = left?;
        let right = right?;

        if right.kind().is_type() {
            return Some(Self::Bibtex(right));
        }

        if left.kind().is_type() {
            return Some(Self::Bibtex(left));
        }

        if left.kind() == bibtex::COMMAND_NAME {
            return Some(Self::Bibtex(left));
        }

        if right.kind() == bibtex::WORD {
            return Some(Self::Bibtex(right));
        }

        if left.kind() == bibtex::WORD {
            return Some(Self::Bibtex(left));
        }

        if right.kind() == bibtex::COMMAND_NAME {
            return Some(Self::Bibtex(right));
        }

        Some(Self::Bibtex(right))
    }

    pub fn as_latex(&self) -> Option<&latex::SyntaxToken> {
        if let Self::Latex(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_bibtex(&self) -> Option<&bibtex::SyntaxToken> {
        if let Self::Bibtex(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn command_range(&self, offset: TextSize) -> Option<TextRange> {
        self.as_latex()
            .filter(|token| token.kind().is_command_name())
            .filter(|token| token.text_range().start() != offset)
            .map(|token| token.text_range())
            .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))
            .or_else(|| {
                self.as_bibtex()
                    .filter(|token| token.kind() == bibtex::COMMAND_NAME)
                    .filter(|token| token.text_range().start() != offset)
                    .map(|token| token.text_range())
                    .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))
            })
    }
}

pub struct CursorContext<P> {
    pub request: FeatureRequest<P>,
    pub cursor: Cursor,
    pub offset: TextSize,
}

impl<P: HasPosition> CursorContext<P> {
    pub fn new(request: FeatureRequest<P>) -> Self {
        let main_document = request.main_document();
        let offset = main_document
            .line_index
            .offset_lsp(request.params.position());

        let cursor = match &main_document.data {
            DocumentData::Latex(data) => {
                let root = latex::SyntaxNode::new_root(data.green.clone());
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_latex(left, right)
            }
            DocumentData::Bibtex(data) => {
                let root = bibtex::SyntaxNode::new_root(data.green.clone());
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_bibtex(left, right)
            }
            DocumentData::BuildLog(_) => None,
        };

        Self {
            request,
            cursor: cursor.unwrap_or(Cursor::Nothing),
            offset,
        }
    }

    pub fn is_inside_latex_curly(&self, group: &impl latex::HasCurly) -> bool {
        latex::small_range(group).contains(self.offset) || group.right_curly().is_none()
    }

    pub fn find_citation_key_word(&self) -> Option<(String, TextRange)> {
        let word = self
            .cursor
            .as_latex()
            .filter(|token| token.kind() == latex::WORD)?;

        let key = latex::Key::cast(word.parent()?)?;

        let group = latex::CurlyGroupWordList::cast(key.syntax().parent()?)?;
        latex::Citation::cast(group.syntax().parent()?)?;
        Some((key.to_string(), latex::small_range(&key)))
    }

    pub fn find_citation_key_command(&self) -> Option<(String, TextRange)> {
        let command = self.cursor.as_latex()?;

        let citation = latex::Citation::cast(command.parent()?)?;
        let key = citation.key_list()?.keys().next()?;
        Some((key.to_string(), latex::small_range(&key)))
    }

    pub fn find_entry_key(&self) -> Option<(String, TextRange)> {
        let word = self
            .cursor
            .as_bibtex()
            .filter(|token| token.kind() == bibtex::WORD)?;

        let key = bibtex::Key::cast(word.parent()?)?;

        bibtex::Entry::cast(key.syntax().parent()?)?;
        Some((key.to_string(), bibtex::small_range(&key)))
    }

    pub fn find_label_name_key(&self) -> Option<(String, TextRange)> {
        let name = self
            .cursor
            .as_latex()
            .filter(|token| token.kind() == latex::WORD)?;

        let key = latex::Key::cast(name.parent()?)?;

        if matches!(
            key.syntax().parent()?.parent()?.kind(),
            latex::LABEL_DEFINITION | latex::LABEL_REFERENCE | latex::LABEL_REFERENCE_RANGE
        ) {
            Some((key.to_string(), latex::small_range(&key)))
        } else {
            None
        }
    }

    pub fn find_label_name_command(&self) -> Option<(String, TextRange)> {
        let node = self.cursor.as_latex()?.parent()?;
        if let Some(label) = latex::LabelDefinition::cast(node.clone()) {
            let name = label.name()?.key()?;
            Some((name.to_string(), latex::small_range(&name)))
        } else if let Some(label) = latex::LabelReference::cast(node.clone()) {
            let name = label.name_list()?.keys().next()?;
            Some((name.to_string(), latex::small_range(&name)))
        } else if let Some(label) = latex::LabelReferenceRange::cast(node) {
            let name = label.from()?.key()?;
            Some((name.to_string(), latex::small_range(&name)))
        } else {
            None
        }
    }

    pub fn find_environment_name(&self) -> Option<(String, TextRange)> {
        let (name, range, group) = self.find_curly_group_word()?;

        if !matches!(group.syntax().parent()?.kind(), latex::BEGIN | latex::END) {
            return None;
        }

        Some((name, range))
    }

    pub fn find_curly_group_word(&self) -> Option<(String, TextRange, latex::CurlyGroupWord)> {
        let token = self.cursor.as_latex()?;
        let key = latex::Key::cast(token.parent()?);

        let group = key
            .as_ref()
            .and_then(|key| key.syntax().parent())
            .unwrap_or(token.parent()?);

        let group =
            latex::CurlyGroupWord::cast(group).filter(|group| self.is_inside_latex_curly(group))?;

        key.map(|key| (key.to_string(), latex::small_range(&key), group.clone()))
            .or_else(|| Some((String::new(), TextRange::empty(self.offset), group)))
    }

    pub fn find_curly_group_word_list(
        &self,
    ) -> Option<(String, TextRange, latex::CurlyGroupWordList)> {
        let token = self.cursor.as_latex()?;
        let key = latex::Key::cast(token.parent()?);

        let group = key
            .as_ref()
            .and_then(|key| key.syntax().parent())
            .unwrap_or(token.parent()?);

        let group = latex::CurlyGroupWordList::cast(group)
            .filter(|group| self.is_inside_latex_curly(group))?;

        key.map(|key| {
            let range = if group
                .syntax()
                .last_token()
                .filter(|tok| tok.kind() == latex::MISSING)
                .is_some()
            {
                TextRange::new(latex::small_range(&key).start(), token.text_range().end())
            } else {
                latex::small_range(&key)
            };

            (key.to_string(), range, group.clone())
        })
        .or_else(|| Some((String::new(), TextRange::empty(self.offset), group)))
    }
}

pub trait HasPosition {
    fn position(&self) -> Position;
}

impl HasPosition for CompletionParams {
    fn position(&self) -> Position {
        self.text_document_position.position
    }
}

impl HasPosition for TextDocumentPositionParams {
    fn position(&self) -> Position {
        self.position
    }
}

impl HasPosition for RenameParams {
    fn position(&self) -> Position {
        self.text_document_position.position
    }
}

impl HasPosition for ReferenceParams {
    fn position(&self) -> Position {
        self.text_document_position.position
    }
}

impl HasPosition for HoverParams {
    fn position(&self) -> Position {
        self.text_document_position_params.position
    }
}

impl HasPosition for GotoDefinitionParams {
    fn position(&self) -> Position {
        self.text_document_position_params.position
    }
}

impl HasPosition for DocumentHighlightParams {
    fn position(&self) -> Position {
        self.text_document_position_params.position
    }
}
