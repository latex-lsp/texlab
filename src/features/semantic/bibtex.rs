use cancellation::CancellationToken;
use lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensRangeParams,
};
use rowan::{NodeOrToken, TextRange};

use crate::{
    features::FeatureRequest,
    syntax::{bibtex, CstNode},
    BibtexEntryTypeCategory, LineIndex, LineIndexExt, LANGUAGE_DATA,
};

use super::legend::*;

struct Context<'a> {
    line_index: &'a LineIndex,
    previous_line: u32,
    previous_character: u32,
    tokens: &'a mut Vec<SemanticToken>,
    cancellation_token: &'a CancellationToken,
}

impl<'a> Context<'a> {
    pub fn insert(
        &mut self,
        range: TextRange,
        token_type: SemanticTokenType,
        token_modifiers_bitset: ModifierSet,
    ) {
        let range = self.line_index.line_col_lsp_range(range);

        let mut delta_line = range.start.line;
        let mut delta_start = range.start.character;
        if !self.tokens.is_empty() {
            delta_line -= self.previous_line;
            if delta_line == 0 {
                delta_start -= self.previous_character;
            }
        }
        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: range.end.character - range.start.character,
            token_type: type_index(token_type),
            token_modifiers_bitset: token_modifiers_bitset.0,
        });

        self.previous_line = range.start.line;
        self.previous_character = range.start.character;
    }
}

pub fn find_bibtex_semantic_tokens_range(
    request: &FeatureRequest<SemanticTokensRangeParams>,
    tokens: &mut Vec<SemanticToken>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    let document = request.main_document();
    let data = document.data.as_bibtex()?;

    let mut context = Context {
        line_index: &document.line_index,
        previous_line: 0,
        previous_character: 0,
        tokens,
        cancellation_token,
    };

    let range = document.line_index.offset_lsp_range(request.params.range);

    for node in data
        .root
        .children()
        .filter(|node| node.text_range().intersect(range).is_some())
    {
        cancellation_token.result().ok()?;
        visit_junk(&mut context, node)
            .or_else(|| visit_preamble(&mut context, node))
            .or_else(|| visit_string(&mut context, node))
            .or_else(|| visit_entry(&mut context, node));
    }

    Some(())
}

fn visit_junk(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    let junk = bibtex::Junk::cast(node)?;
    for token in junk
        .syntax()
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() != bibtex::WHITESPACE)
    {
        context.insert(token.text_range(), JUNK, ModifierSet::default());
    }
    Some(())
}

fn visit_preamble(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    let preamble = bibtex::Preamble::cast(node)?;
    for element in preamble.syntax().children_with_tokens() {
        match element {
            NodeOrToken::Node(value) => {
                visit_value(context, value);
            }
            NodeOrToken::Token(token) => match token.kind() {
                bibtex::PREAMBLE_TYPE => {
                    context.insert(token.text_range(), PREAMBLE_TYPE, ModifierSet::default());
                }
                bibtex::L_CURLY | bibtex::R_CURLY => {
                    context.insert(token.text_range(), CURLY, ModifierSet::default());
                }
                bibtex::L_PAREN | bibtex::R_PAREN => {
                    context.insert(token.text_range(), PAREN, ModifierSet::default());
                }
                _ => {}
            },
        }
    }

    Some(())
}

fn visit_string(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    let string = bibtex::String::cast(node)?;
    for element in string.syntax().children_with_tokens() {
        match element {
            NodeOrToken::Node(value) => {
                visit_value(context, value);
            }
            NodeOrToken::Token(token) => match token.kind() {
                bibtex::STRING_TYPE => {
                    context.insert(token.text_range(), STRING_TYPE, ModifierSet::default());
                }
                bibtex::L_CURLY | bibtex::R_CURLY => {
                    context.insert(token.text_range(), CURLY, ModifierSet::default());
                }
                bibtex::L_PAREN | bibtex::R_PAREN => {
                    context.insert(token.text_range(), PAREN, ModifierSet::default());
                }
                bibtex::WORD => {
                    context.insert(
                        token.text_range(),
                        STRING_NAME,
                        ModifierSet::default() | SemanticTokenModifier::DEFINITION,
                    );
                }
                bibtex::EQUALITY_SIGN => {
                    context.insert(token.text_range(), EQUALITY_SIGN, ModifierSet::default());
                }
                _ => {}
            },
        }
    }

    Some(())
}

fn visit_entry(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    let entry = bibtex::Entry::cast(node)?;

    for element in entry.syntax().children_with_tokens() {
        match element {
            NodeOrToken::Node(field) => {
                visit_field(context, field);
            }
            NodeOrToken::Token(token) => match token.kind() {
                bibtex::ENTRY_TYPE => {
                    context.insert(
                        token.text_range(),
                        match LANGUAGE_DATA.find_entry_type(&token.text()[1..]) {
                            Some(ty) => match ty.category {
                                BibtexEntryTypeCategory::Article => ARTICLE_TYPE,
                                BibtexEntryTypeCategory::Misc => MISC_TYPE,
                                BibtexEntryTypeCategory::String => STRING_TYPE,
                                BibtexEntryTypeCategory::Book => BOOK_TYPE,
                                BibtexEntryTypeCategory::Collection => COLLECTION_TYPE,
                                BibtexEntryTypeCategory::Part => PART_TYPE,
                                BibtexEntryTypeCategory::Thesis => THESIS_TYPE,
                            },
                            None => UNKNOWN_TYPE,
                        },
                        ModifierSet::default(),
                    );
                }
                bibtex::L_CURLY | bibtex::R_CURLY => {
                    context.insert(token.text_range(), CURLY, ModifierSet::default());
                }
                bibtex::L_PAREN | bibtex::R_PAREN => {
                    context.insert(token.text_range(), PAREN, ModifierSet::default());
                }
                bibtex::WORD => {
                    context.insert(
                        token.text_range(),
                        ENTRY_KEY,
                        ModifierSet::default()
                            | SemanticTokenModifier::DEFINITION
                            | SemanticTokenModifier::READONLY,
                    );
                }
                bibtex::COMMA => {
                    context.insert(token.text_range(), COMMA, ModifierSet::default());
                }
                _ => {}
            },
        }
    }

    Some(())
}

fn visit_field(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    context.cancellation_token.result().ok()?;
    let field = bibtex::Field::cast(node)?;
    context.insert(field.name()?.text_range(), FIELD, ModifierSet::default());
    if let Some(token) = field.equality_sign() {
        context.insert(token.text_range(), EQUALITY_SIGN, ModifierSet::default());
    }

    if let Some(value) = field.value() {
        visit_value(context, value.syntax());
    }
    Some(())
}

fn visit_value(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    context.cancellation_token.result().ok()?;
    let value = bibtex::Value::cast(node)?;
    for child in value.syntax().children_with_tokens() {
        match child {
            NodeOrToken::Token(token) if token.kind() == bibtex::HASH => {
                context.insert(token.text_range(), HASH, ModifierSet::default());
            }
            NodeOrToken::Token(_) => {}
            NodeOrToken::Node(token) => {
                if let Some(word) = token
                    .first_child_or_token()
                    .and_then(|child| child.into_token())
                {
                    if word.text().chars().all(|c| c.is_ascii_digit()) {
                        context.insert(
                            word.text_range(),
                            SemanticTokenType::NUMBER,
                            ModifierSet::default(),
                        );
                    } else {
                        context.insert(word.text_range(), STRING_NAME, ModifierSet::default());
                    }
                } else {
                    visit_content(context, token);
                }
            }
        }
    }
    Some(())
}

fn visit_content(context: &mut Context, node: &bibtex::SyntaxNode) -> Option<()> {
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        match token.kind() {
            bibtex::WORD => {
                context.insert(token.text_range(), TEXT, ModifierSet::default());
            }
            bibtex::COMMA => {
                context.insert(token.text_range(), COMMA, ModifierSet::default());
            }
            bibtex::L_CURLY | bibtex::R_CURLY => {
                context.insert(token.text_range(), CURLY, ModifierSet::default());
            }
            bibtex::L_PAREN | bibtex::R_PAREN => {
                context.insert(token.text_range(), PAREN, ModifierSet::default());
            }
            bibtex::SyntaxKind::QUOTE => {
                context.insert(token.text_range(), QUOTE, ModifierSet::default());
            }
            bibtex::SyntaxKind::EQUALITY_SIGN => {
                context.insert(token.text_range(), EQUALITY_SIGN, ModifierSet::default());
            }
            bibtex::SyntaxKind::COMMAND_NAME => {
                context.insert(token.text_range(), GENERIC_COMMAND, ModifierSet::default());
            }
            _ => {}
        }
    }
    Some(())
}
