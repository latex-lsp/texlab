use lsp_types::{Position, Url};
use rowan::{ast::AstNode, TextRange, TextSize};

use crate::{
    db::{parse::DocumentData, Document, Workspace},
    syntax::{bibtex, latex},
    Db,
};

use super::{line_index::LineIndex, line_index_ext::LineIndexExt};

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

    pub fn command_range(&self, offset: TextSize) -> Option<TextRange> {
        self.as_tex()
            .filter(|token| token.kind() == latex::COMMAND_NAME)
            .filter(|token| token.text_range().start() != offset)
            .map(|token| token.text_range())
            .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))
            .or_else(|| {
                self.as_bib()
                    .filter(|token| {
                        matches!(token.kind(), bibtex::COMMAND_NAME | bibtex::ACCENT_NAME)
                    })
                    .filter(|token| token.text_range().start() != offset)
                    .map(|token| token.text_range())
                    .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))
            })
    }
}

pub struct CursorContext<'db, T = ()> {
    pub db: &'db dyn Db,
    pub document: Document,
    pub line_index: &'db LineIndex,
    pub workspace: Workspace,
    pub cursor: Cursor,
    pub offset: TextSize,
    pub params: T,
}

impl<'db, T> CursorContext<'db, T> {
    pub fn new(db: &'db dyn Db, uri: &Url, position: Position, params: T) -> Option<Self> {
        let workspace = Workspace::get(db);
        let document = workspace.lookup_uri(db, uri)?;
        let line_index = document.line_index(db);
        let offset = line_index.offset_lsp(position);

        let cursor = match document.parse(db) {
            DocumentData::Tex(data) => {
                let root = data.root(db);
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_tex(left, right)
            }
            DocumentData::Bib(data) => {
                let root = data.root(db);
                let left = root.token_at_offset(offset).left_biased();
                let right = root.token_at_offset(offset).right_biased();
                Cursor::new_bib(left, right)
            }
            DocumentData::Log(_) | DocumentData::TexlabRoot(_) | DocumentData::Tectonic(_) => None,
        };

        Some(Self {
            db,
            document,
            line_index,
            workspace,
            cursor: cursor.unwrap_or(Cursor::Nothing),
            offset,
            params,
        })
    }

    pub fn related(&self) -> impl Iterator<Item = Document> + '_ {
        self.workspace
            .related(self.db, self.document)
            .iter()
            .copied()
    }

    pub fn is_inside_latex_curly(&self, group: &impl latex::HasCurly) -> bool {
        latex::small_range(group).contains(self.offset) || group.right_curly().is_none()
    }

    pub fn find_citation_key_word(&self) -> Option<(String, TextRange)> {
        let word = self
            .cursor
            .as_tex()
            .filter(|token| token.kind() == latex::WORD)?;

        let key = latex::Key::cast(word.parent()?)?;

        let group = latex::CurlyGroupWordList::cast(key.syntax().parent()?)?;
        latex::Citation::cast(group.syntax().parent()?)?;
        Some((key.to_string(), latex::small_range(&key)))
    }

    pub fn find_citation_key_command(&self) -> Option<(String, TextRange)> {
        let command = self.cursor.as_tex()?;

        let citation = latex::Citation::cast(command.parent()?)?;
        let key = citation.key_list()?.keys().next()?;
        Some((key.to_string(), latex::small_range(&key)))
    }

    pub fn find_entry_key(&self) -> Option<(String, TextRange)> {
        let key = self
            .cursor
            .as_bib()
            .filter(|token| token.kind() == bibtex::NAME)?;

        bibtex::Entry::cast(key.parent()?)?;
        Some((key.to_string(), key.text_range()))
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

    pub fn find_label_name_command(&self) -> Option<(String, TextRange)> {
        let node = self.cursor.as_tex()?.parent()?;
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

    pub fn find_environment(&self) -> Option<(latex::Key, latex::Key)> {
        let token = self.cursor.as_tex()?;

        for env in token.parent_ancestors()
            .filter_map(latex::Environment::cast) {

            let beg = env.begin()?
                .name()?
                .key()?;
            let end = env.end()?
                .name()?
                .key()?;

            return Some((beg,end));
        }

        None
    }

    pub fn find_curly_group_word(&self) -> Option<(String, TextRange, latex::CurlyGroupWord)> {
        let token = self.cursor.as_tex()?;
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
        let token = self.cursor.as_tex()?;
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
                .map_or(false, |tok| tok.kind() != latex::R_CURLY)
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
