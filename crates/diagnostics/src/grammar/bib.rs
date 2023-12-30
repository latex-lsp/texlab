use base_db::{BibDocumentData, Document};
use multimap::MultiMap;
use rowan::{ast::AstNode, TextRange};
use syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue};
use url::Url;

use crate::types::{BibError, Diagnostic};

pub fn update(document: &Document, results: &mut MultiMap<Url, Diagnostic>) -> Option<()> {
    let data = document.data.as_bib()?;
    let mut analyzer = Analyzer {
        data,
        diagnostics: Vec::new(),
    };

    analyzer.analyze_root();

    *results
        .entry(document.uri.clone())
        .or_insert_vec(Vec::new()) = analyzer.diagnostics;

    Some(())
}

struct Analyzer<'a> {
    data: &'a BibDocumentData,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Analyzer<'a> {
    fn analyze_root(&mut self) {
        for node in self.data.root_node().descendants() {
            if let Some(entry) = bibtex::Entry::cast(node.clone()) {
                self.analyze_entry(entry);
            } else if let Some(field) = bibtex::Field::cast(node.clone()) {
                self.analyze_field(field);
            }
        }
    }

    fn analyze_entry(&mut self, entry: bibtex::Entry) {
        if entry.left_delim_token().is_none() {
            let offset = entry.type_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic::Bib(
                TextRange::empty(offset),
                BibError::ExpectingLCurly,
            ));

            return;
        }

        if entry.name_token().is_none() {
            let offset = entry.left_delim_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic::Bib(
                TextRange::empty(offset),
                BibError::ExpectingKey,
            ));

            return;
        }

        if entry.right_delim_token().is_none() {
            let offset = entry.syntax().text_range().end();
            self.diagnostics.push(Diagnostic::Bib(
                TextRange::empty(offset),
                BibError::ExpectingRCurly,
            ));
        }
    }

    fn analyze_field(&mut self, field: bibtex::Field) {
        if field.eq_token().is_none() {
            let offset = field.name_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic::Bib(
                TextRange::empty(offset),
                BibError::ExpectingEq,
            ));

            return;
        }

        if field.value().is_none() {
            let offset = field.eq_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic::Bib(
                TextRange::empty(offset),
                BibError::ExpectingFieldValue,
            ));
        }
    }
}
