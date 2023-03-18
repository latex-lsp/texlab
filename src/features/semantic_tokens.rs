mod label;

use bitflags::bitflags;
use lsp_types::{
    Position, Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend, Url,
};
use rowan::TextRange;

use crate::{
    db::Workspace,
    util::{line_index::LineIndex, line_index_ext::LineIndexExt},
    Db,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[repr(u32)]
pub enum TokenKind {
    Label = 0,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
    pub struct TokenModifiers: u32 {
        const NONE = 0;
        const UNDEFINED = 1;
        const UNUSED = 2;
    }
}

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![SemanticTokenType::new("label")],
        token_modifiers: vec![
            SemanticTokenModifier::new("undefined"),
            SemanticTokenModifier::new("unused"),
        ],
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Token {
    pub range: TextRange,
    pub kind: TokenKind,
    pub modifiers: TokenModifiers,
}

#[derive(Debug, Default)]
pub struct TokenBuilder {
    tokens: Vec<Token>,
}

impl TokenBuilder {
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn finish(mut self, line_index: &LineIndex) -> SemanticTokens {
        let mut data = Vec::new();

        self.tokens.sort_by_key(|token| token.range.start());

        let mut last_pos = Position::new(0, 0);
        for token in self.tokens {
            let range = line_index.line_col_lsp_range(token.range);
            let length = range.end.character - range.start.character;
            let token_type = token.kind as u32;
            let token_modifiers_bitset = token.modifiers.bits();

            if range.start.line > last_pos.line {
                let delta_line = range.start.line - last_pos.line;
                let delta_start = range.start.character;
                data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset,
                });
            } else {
                let delta_line = 0;
                let delta_start = last_pos.character - range.start.character;
                data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset,
                });
            }

            last_pos = range.end;
        }

        SemanticTokens {
            result_id: None,
            data,
        }
    }
}

pub fn find_all(db: &dyn Db, uri: &Url, viewport: Range) -> Option<SemanticTokens> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;
    let viewport = document.line_index(db).offset_lsp_range(viewport);
    let mut builder = TokenBuilder::default();
    label::find(db, document, viewport, &mut builder);
    Some(builder.finish(document.line_index(db)))
}
