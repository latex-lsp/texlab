use lsp_types::DocumentSymbolParams;
use rowan::ast::AstNode;

use crate::{
    features::FeatureRequest,
    syntax::bibtex::{self, HasName, HasType},
    BibtexEntryTypeCategory, LineIndexExt, LANGUAGE_DATA,
};

use super::types::{InternalSymbol, InternalSymbolKind};

pub fn find_bibtex_symbols(
    request: &FeatureRequest<DocumentSymbolParams>,
    buf: &mut Vec<InternalSymbol>,
) -> Option<()> {
    let main_document = request.main_document();
    let data = main_document.data.as_bibtex()?;

    for node in bibtex::SyntaxNode::new_root(data.green.clone()).children() {
        if let Some(string) = bibtex::StringDef::cast(node.clone()) {
            if let Some(name) = string.name_token() {
                buf.push(InternalSymbol {
                    name: name.text().into(),
                    label: None,
                    kind: InternalSymbolKind::String,
                    deprecated: false,
                    full_range: main_document
                        .line_index
                        .line_col_lsp_range(string.syntax().text_range()),
                    selection_range: main_document
                        .line_index
                        .line_col_lsp_range(name.text_range()),
                    children: Vec::new(),
                })
            }
        } else if let Some(entry) = bibtex::Entry::cast(node) {
            if let Some(ty) = entry.type_token() {
                if let Some(key) = entry.name_token() {
                    let mut children = Vec::new();
                    for field in entry.fields() {
                        if let Some(name) = field.name_token() {
                            let symbol = InternalSymbol {
                                name: name.text().to_string(),
                                label: None,
                                kind: InternalSymbolKind::Field,
                                deprecated: false,
                                full_range: main_document
                                    .line_index
                                    .line_col_lsp_range(field.syntax().text_range()),
                                selection_range: main_document
                                    .line_index
                                    .line_col_lsp_range(name.text_range()),
                                children: Vec::new(),
                            };
                            children.push(symbol);
                        }
                    }

                    let category = LANGUAGE_DATA
                        .find_entry_type(&ty.text()[1..])
                        .map(|ty| ty.category)
                        .unwrap_or(BibtexEntryTypeCategory::Misc);

                    buf.push(InternalSymbol {
                        name: key.to_string(),
                        label: None,
                        kind: InternalSymbolKind::Entry(category),
                        deprecated: false,
                        full_range: main_document
                            .line_index
                            .line_col_lsp_range(entry.syntax().text_range()),
                        selection_range: main_document
                            .line_index
                            .line_col_lsp_range(key.text_range()),
                        children,
                    });
                }
            }
        }
    }
    Some(())
}
