use base_db::{Document, DocumentData, Workspace};
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;
use syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue};
use url::Url;

use crate::{Diagnostic, DiagnosticSource, ErrorCode};

#[derive(Debug, Default)]
pub struct BibSyntaxErrors {
    errors: FxHashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticSource for BibSyntaxErrors {
    fn on_change(&mut self, _workspace: &Workspace, document: &Document) {
        let mut analyzer = Analyzer {
            document,
            diagnostics: Vec::new(),
        };

        analyzer.analyze_root();
        self.errors
            .insert(document.uri.clone(), analyzer.diagnostics);
    }

    fn cleanup(&mut self, workspace: &Workspace) {
        self.errors.retain(|uri, _| workspace.lookup(uri).is_some());
    }

    fn publish<'this, 'db>(
        &'this self,
        workspace: &'db Workspace,
        results: &mut FxHashMap<&'db Url, Vec<&'this Diagnostic>>,
    ) {
        for document in workspace.iter() {
            let Some(diagnostics) = self.errors.get(&document.uri) else { continue };

            results
                .entry(&document.uri)
                .or_default()
                .extend(diagnostics.iter());
        }
    }
}

struct Analyzer<'a> {
    document: &'a Document,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Analyzer<'a> {
    fn analyze_root(&mut self) {
        let DocumentData::Bib(data) = &self.document.data else { return };

        for node in bibtex::SyntaxNode::new_root(data.green.clone()).descendants() {
            if let Some(entry) = bibtex::Entry::cast(node.clone()) {
                self.analyze_entry(entry);
            } else if let Some(field) = bibtex::Field::cast(node.clone()) {
                self.analyze_field(field);
            }
        }
    }

    fn analyze_entry(&mut self, entry: bibtex::Entry) {
        if entry.left_delim_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: entry.type_token().unwrap().text_range(),
                code: ErrorCode::ExpectingLCurly,
            });

            return;
        }

        if entry.name_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: entry.left_delim_token().unwrap().text_range(),
                code: ErrorCode::ExpectingKey,
            });

            return;
        }

        if entry.right_delim_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(entry.syntax().text_range().end()),
                code: ErrorCode::ExpectingRCurly,
            });
        }
    }

    fn analyze_field(&mut self, field: bibtex::Field) {
        if field.eq_token().is_none() {
            let code = ErrorCode::ExpectingEq;
            self.diagnostics.push(Diagnostic {
                range: field.name_token().unwrap().text_range(),
                code,
            });

            return;
        }

        if field.value().is_none() {
            let code = ErrorCode::ExpectingFieldValue;
            self.diagnostics.push(Diagnostic {
                range: field.name_token().unwrap().text_range(),
                code,
            });
        }
    }
}
