use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName, HasType};

use crate::{
    db::Document,
    util::{
        lang_data::{BibtexEntryTypeCategory, LANGUAGE_DATA},
        line_index::LineIndex,
        line_index_ext::LineIndexExt,
    },
    Db,
};

use super::types::{InternalSymbol, InternalSymbolKind};

pub fn find_symbols(db: &dyn Db, document: Document, buf: &mut Vec<InternalSymbol>) -> Option<()> {
    let data = document.parse(db).as_bib()?;
    let line_index = document.line_index(db);
    for node in data.root(db).children() {
        process_string(node.clone(), line_index, buf)
            .or_else(|| process_entry(node, line_index, buf));
    }

    Some(())
}

fn process_string(
    node: bibtex::SyntaxNode,
    line_index: &LineIndex,
    buf: &mut Vec<InternalSymbol>,
) -> Option<()> {
    let string = bibtex::StringDef::cast(node)?;
    let name = string.name_token()?;
    buf.push(InternalSymbol {
        name: name.text().into(),
        label: None,
        kind: InternalSymbolKind::String,
        deprecated: false,
        full_range: line_index.line_col_lsp_range(string.syntax().text_range()),
        selection_range: line_index.line_col_lsp_range(name.text_range()),
        children: Vec::new(),
    });

    Some(())
}

fn process_entry(
    node: bibtex::SyntaxNode,
    line_index: &LineIndex,
    buf: &mut Vec<InternalSymbol>,
) -> Option<()> {
    let entry = bibtex::Entry::cast(node)?;
    let ty = entry.type_token()?;
    let key = entry.name_token()?;
    let mut children = Vec::new();
    for field in entry.fields() {
        if let Some(name) = field.name_token() {
            let symbol = InternalSymbol {
                name: name.text().to_string(),
                label: None,
                kind: InternalSymbolKind::Field,
                deprecated: false,
                full_range: line_index.line_col_lsp_range(field.syntax().text_range()),
                selection_range: line_index.line_col_lsp_range(name.text_range()),
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
        full_range: line_index.line_col_lsp_range(entry.syntax().text_range()),
        selection_range: line_index.line_col_lsp_range(key.text_range()),
        children,
    });

    Some(())
}
