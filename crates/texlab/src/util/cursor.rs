use base_db::{Document, DocumentData, Project, Workspace};
use lsp_types::{Position, Url};
use rowan::{ast::AstNode, TextRange, TextSize};
use syntax::{bibtex, latex};

use super::line_index_ext::LineIndexExt;

#[derive(Debug)]
pub enum Cursor {
    Tex(latex::SyntaxToken),
    Bib(bibtex::SyntaxToken),
    Nothing,
}

impl Cursor {
    pub fn new_tex(
        left: Option<latex::SyntaxToken>,
        right: Option<latex::SyntaxToken>,
    ) -> Option<Self> {
        let left = left?;
        let right = right?;

        if left.kind() == latex::COMMAND_NAME {
            return Some(Self::Tex(left));
        }

        if right.kind() == latex::WORD {
            return Some(Self::Tex(right));
        }

        if left.kind() == latex::WORD {
            return Some(Self::Tex(left));
        }

        if right.kind() == latex::COMMAND_NAME {
            return Some(Self::Tex(right));
        }

        if left.kind() == latex::WHITESPACE && left.parent()?.kind() == latex::KEY {
            return Some(Self::Tex(left));
        }

        if matches!(right.kind(), latex::WHITESPACE | latex::LINE_BREAK)
            && right.parent()?.kind() == latex::KEY
        {
            return Some(Self::Tex(right));
        }

        Some(Self::Tex(right))
    }

    pub fn new_bib(
        left: Option<bibtex::SyntaxToken>,
        right: Option<bibtex::SyntaxToken>,
    ) -> Option<Self> {
        let left = left?;
        let right = right?;

        if right.kind() == bibtex::TYPE {
            return Some(Self::Bib(right));
        }

        if left.kind() == bibtex::TYPE {
            return Some(Self::Bib(left));
        }

        if matches!(left.kind(), bibtex::COMMAND_NAME | bibtex::ACCENT_NAME) {
            return Some(Self::Bib(left));
        }

        if matches!(right.kind(), bibtex::WORD | bibtex::NAME) {
            return Some(Self::Bib(right));
        }

        if matches!(left.kind(), bibtex::WORD | bibtex::NAME) {
            return Some(Self::Bib(left));
        }

        if matches!(right.kind(), bibtex::COMMAND_NAME | bibtex::ACCENT_NAME) {
            return Some(Self::Bib(right));
        }

        Some(Self::Bib(right))
    }

    pub fn as_tex(&self) -> Option<&latex::SyntaxToken> {
        if let Self::Tex(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_bib(&self) -> Option<&bibtex::SyntaxToken> {
        if let Self::Bib(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

pub struct CursorContext<'a, T = ()> {
    pub workspace: &'a Workspace,
    pub document: &'a Document,
    pub project: Project<'a>,
    pub cursor: Cursor,
    pub offset: TextSize,
    pub params: T,
}

impl<'a, T> CursorContext<'a, T> {
    pub fn new(workspace: &'a Workspace, uri: &Url, position: Position, params: T) -> Option<Self> {
        let document = workspace.lookup(uri)?;
        let offset = document.line_index.offset_lsp(position);

        let cursor = match &document.data {
            DocumentData::Tex(data) => {
                let root = data.root_node();
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_tex(left, right)
            }
            DocumentData::Bib(data) => {
                let root = data.root_node();
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_bib(left, right)
            }
            DocumentData::Aux(_)
            | DocumentData::Log(_)
            | DocumentData::Root
            | DocumentData::Tectonic => None,
        };

        Some(Self {
            workspace,
            document,
            project: workspace.project(document),
            cursor: cursor.unwrap_or(Cursor::Nothing),
            offset,
            params,
        })
    }

    pub fn find_label_name_key(&self) -> Option<(String, TextRange)> {
        let name = self
            .cursor
            .as_tex()
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
}
