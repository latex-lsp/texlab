use base_db::{Document, DocumentData, Workspace};
use rowan::{ast::AstNode, TextRange};
use syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue};

use crate::{
    types::{DiagnosticData, SyntaxError},
    util::SimpleDiagnosticSource,
    Diagnostic, DiagnosticBuilder, DiagnosticSource,
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

    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        self.0.publish(workspace, builder);
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
            let offset = entry.type_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(offset),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingLCurly),
            });

            return;
        }

        if entry.name_token().is_none() {
            let offset = entry.left_delim_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(offset),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingKey),
            });

            return;
        }

        if entry.right_delim_token().is_none() {
            let offset = entry.syntax().text_range().end();
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(offset),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingRCurly),
            });
        }
    }

    fn analyze_field(&mut self, field: bibtex::Field) {
        if field.eq_token().is_none() {
            let offset = field.name_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(offset),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingEq),
            });

            return;
        }

        if field.value().is_none() {
            let offset = field.eq_token().unwrap().text_range().end();
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(offset),
                data: DiagnosticData::Syntax(SyntaxError::ExpectingFieldValue),
            });
        }
    }
}
