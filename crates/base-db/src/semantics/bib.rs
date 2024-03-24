use bibtex_utils::field::text::TextFieldData;
use itertools::Itertools;
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;
use syntax::bibtex::{self, HasName, HasType, HasValue};

use crate::data::{BibtexEntryType, BibtexEntryTypeCategory};

use super::Span;

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub entries: Vec<Entry>,
    pub strings: Vec<StringDef>,
    /// Map from string definition keys to their expanded values.
    pub expanded_defs: FxHashMap<String, String>,
}

impl Semantics {
    pub fn process_root(&mut self, root: &bibtex::SyntaxNode) {
        for node in root.children() {
            if let Some(entry) = bibtex::Entry::cast(node.clone()) {
                self.process_entry(entry);
            } else if let Some(string) = bibtex::StringDef::cast(node) {
                self.process_string_def(string);
            }
        }
    }

    fn process_entry(&mut self, entry: bibtex::Entry) {
        if let Some(name) = entry.name_token() {
            let type_token = entry.type_token().unwrap();
            let category = BibtexEntryType::find(type_token.text())
                .map_or(BibtexEntryTypeCategory::Misc, |ty| ty.category);

            let field_values = entry.fields().filter_map(|field| {
                Some(TextFieldData::parse(&field.value()?, &self.expanded_defs)?.text)
            });

            let keywords = [name.text().into(), type_token.text().into()]
                .into_iter()
                .chain(field_values)
                .join(" ");

            self.entries.push(Entry {
                name: Span {
                    range: name.text_range(),
                    text: name.text().into(),
                },
                full_range: entry.syntax().text_range(),
                category,
                keywords,
            });
        }
    }

    fn process_string_def(&mut self, string: bibtex::StringDef) {
        if let Some(name) = string.name_token() {
            self.strings.push(StringDef {
                name: Span {
                    range: name.text_range(),
                    text: name.text().into(),
                },
                full_range: string.syntax().text_range(),
            });
            if let Some(value) = string.value() {
                if let Some(data) = TextFieldData::parse(&value, &self.expanded_defs) {
                    self.expanded_defs.insert(name.text().into(), data.text);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub name: Span,
    pub full_range: TextRange,
    pub keywords: String,
    pub category: BibtexEntryTypeCategory,
}

#[derive(Debug, Clone)]
pub struct StringDef {
    pub name: Span,
    pub full_range: TextRange,
}
