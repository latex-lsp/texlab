use cstree::{TextRange, TextSize};
use lsp_types::{
    CompletionParams, Position, ReferenceParams, RenameParams, TextDocumentPositionParams,
};

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
                let left = data.root.token_at_offset(offset).left_biased();
                let right = data.root.token_at_offset(offset).right_biased();
                Cursor::new_latex(left, right)
            }
            DocumentData::Bibtex(data) => {
                let left = data.root.token_at_offset(offset).left_biased();
                let right = data.root.token_at_offset(offset).right_biased();
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

    pub fn is_inside_latex_curly<'a>(&self, group: &impl latex::HasCurly<'a>) -> bool {
        group.small_range().contains(self.offset) || group.right_curly().is_none()
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
