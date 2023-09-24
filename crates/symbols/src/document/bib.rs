use base_db::data::{BibtexEntryType, BibtexEntryTypeCategory};
use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName, HasType};

use crate::{Symbol, SymbolKind};

#[derive(Debug)]
pub struct SymbolBuilder;

impl SymbolBuilder {
    pub fn visit(&self, node: &bibtex::SyntaxNode) -> Option<Symbol> {
        if let Some(string) = bibtex::StringDef::cast(node.clone()) {
            self.visit_string(&string)
        } else if let Some(entry) = bibtex::Entry::cast(node.clone()) {
            self.visit_entry(&entry)
        } else {
            None
        }
    }

    fn visit_string(&self, string: &bibtex::StringDef) -> Option<Symbol> {
        let name = string.name_token()?;
        Some(Symbol {
            name: name.text().into(),
            kind: SymbolKind::Entry(BibtexEntryTypeCategory::String),
            label: None,
            full_range: string.syntax().text_range(),
            selection_range: name.text_range(),
            children: Vec::new(),
        })
    }

    fn visit_entry(&self, entry: &bibtex::Entry) -> Option<Symbol> {
        let ty = entry.type_token()?;
        let key = entry.name_token()?;

        let children = entry
            .fields()
            .filter_map(|field| self.visit_field(&field))
            .collect();

        let category = BibtexEntryType::find(ty.text())
            .map_or(BibtexEntryTypeCategory::Misc, |ty| ty.category);

        Some(Symbol {
            name: key.text().into(),
            kind: SymbolKind::Entry(category),
            label: None,
            full_range: entry.syntax().text_range(),
            selection_range: key.text_range(),
            children,
        })
    }

    fn visit_field(&self, field: &bibtex::Field) -> Option<Symbol> {
        let name = field.name_token()?;
        Some(Symbol::new_simple(
            name.text().into(),
            SymbolKind::Field,
            field.syntax().text_range(),
            name.text_range(),
        ))
    }
}
