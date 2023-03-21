mod citations;
mod label;
mod math_delimiter;

use bitflags::bitflags;
use lsp_types::{
    Position, Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend, Url,
};
use rowan::{TextLen, TextRange};

use crate::{
    db::{Document, Workspace},
    util::{line_index::LineIndex, line_index_ext::LineIndexExt},
    Db,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[repr(u32)]
enum TokenKind {
    GenericLabel = 0,
    SectionLabel,
    FloatLabel,
    TheoremLabel,
    EquationLabel,
    EnumItemLabel,
    Citation,
    MathDelimiter,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
    struct TokenModifiers: u32 {
        const NONE = 0;
        const UNDEFINED = 1;
        const UNUSED = 2;
        const DEPRECATED = 4;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Token {
    range: TextRange,
    kind: TokenKind,
    modifiers: TokenModifiers,
}

#[derive(Debug, Default)]
struct TokenBuilder {
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
                let delta_start = range.start.character - last_pos.character;
                data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset,
                });
            }

            last_pos = range.start;
        }

        SemanticTokens {
            result_id: None,
            data,
        }
    }
}

#[derive(Clone, Copy)]
struct Context<'db> {
    db: &'db dyn Db,
    document: Document,
    viewport: TextRange,
}

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::new("genericLabel"),
            SemanticTokenType::new("sectionLabel"),
            SemanticTokenType::new("floatLabel"),
            SemanticTokenType::new("theoremLabel"),
            SemanticTokenType::new("equationLabel"),
            SemanticTokenType::new("enumItemLabel"),
            SemanticTokenType::new("citation"),
            SemanticTokenType::new("mathDelimiter"),
        ],
        token_modifiers: vec![
            SemanticTokenModifier::new("undefined"),
            SemanticTokenModifier::new("unused"),
            SemanticTokenModifier::new("deprecated"),
        ],
    }
}

pub fn find_all(db: &dyn Db, uri: &Url, viewport: Option<Range>) -> Option<SemanticTokens> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;
    let viewport = viewport.map_or_else(
        || TextRange::new(0.into(), document.text(db).text_len()),
        |range| document.line_index(db).offset_lsp_range(range),
    );

    let context = Context {
        db,
        document,
        viewport,
    };

    let mut builder = TokenBuilder::default();

    label::find(context, &mut builder);
    citations::find(context, &mut builder);
    math_delimiter::find(context, &mut builder);

    Some(builder.finish(document.line_index(db)))
}
