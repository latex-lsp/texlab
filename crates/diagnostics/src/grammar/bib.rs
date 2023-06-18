use base_db::{Document, DocumentData, Workspace};
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;
use syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue};
use url::Url;

use crate::{
    util::SimpleDiagnosticSource, Diagnostic, DiagnosticData, DiagnosticSource, SyntaxError,
};

#[derive(Default)]
pub struct BibSyntaxErrors(SimpleDiagnosticSource);

impl DiagnosticSource for BibSyntaxErrors {
    fn update(&mut self, _workspace: &Workspace, document: &Document) {
        let mut analyzer = Analyzer {
            document,
            diagnostics: Vec::new(),
        };

        analyzer.analyze_root();
        self.0
            .errors
            .insert(document.uri.clone(), analyzer.diagnostics);
    }

    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    ) {
        self.0.publish(workspace, results);
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
                data: DiagnosticData::Syntax(SyntaxError::ExpectingLCurly),
            });

            return;
        }

        if entry.name_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: entry.left_delim_token().unwrap().text_range(),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingKey),
            });

            return;
        }

        if entry.right_delim_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(entry.syntax().text_range().end()),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingRCurly),
            });
        }
    }

    fn analyze_field(&mut self, field: bibtex::Field) {
        if field.eq_token().is_none() {
            self.diagnostics.push(Diagnostic {
                range: field.name_token().unwrap().text_range(),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingEq),
            });

            return;
        }

        if field.value().is_none() {
            self.diagnostics.push(Diagnostic {
                range: field.name_token().unwrap().text_range(),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingFieldValue),
            });
        }
    }
}
