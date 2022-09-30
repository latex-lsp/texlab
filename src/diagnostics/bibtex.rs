use std::sync::Arc;

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, Url};
use rowan::{ast::AstNode, TextRange};

use crate::{
    syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue},
    Document, LineIndexExt, Workspace,
};

use super::{BibtexCode, Diagnostic, DiagnosticCode};

pub fn collect_bibtex_diagnostics(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    workspace: &Workspace,
    uri: &Url,
) -> Option<()> {
    let document = workspace.get(uri)?;
    let data = document.data().as_bibtex()?;

    all_diagnostics.alter(uri, |_, mut diagnostics| {
        diagnostics.retain(|diag| !matches!(diag.code, DiagnosticCode::Bibtex(_)));
        diagnostics
    });

    let root = bibtex::SyntaxNode::new_root(data.green.clone());
    for node in root.descendants() {
        analyze_entry(all_diagnostics, &document, node.clone())
            .or_else(|| analyze_field(all_diagnostics, &document, node));
    }

    Some(())
}

fn analyze_entry(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    document: &Document,
    node: bibtex::SyntaxNode,
) -> Option<()> {
    let entry = bibtex::Entry::cast(node)?;
    if entry.left_delim_token().is_none() {
        let code = BibtexCode::ExpectingLCurly;
        all_diagnostics
            .entry(Arc::clone(document.uri()))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index()
                    .line_col_lsp_range(entry.type_token()?.text_range()),
                code: DiagnosticCode::Bibtex(code),
                message: String::from(code),
            });

        return Some(());
    }

    if entry.name_token().is_none() {
        let code = BibtexCode::ExpectingKey;
        all_diagnostics
            .entry(Arc::clone(document.uri()))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index()
                    .line_col_lsp_range(entry.left_delim_token()?.text_range()),
                code: DiagnosticCode::Bibtex(code),
                message: String::from(code),
            });

        return Some(());
    }

    if entry.right_delim_token().is_none() {
        let code = BibtexCode::ExpectingRCurly;
        all_diagnostics
            .entry(Arc::clone(document.uri()))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index()
                    .line_col_lsp_range(TextRange::empty(entry.syntax().text_range().end())),
                code: DiagnosticCode::Bibtex(code),
                message: String::from(code),
            });

        return Some(());
    }

    Some(())
}

fn analyze_field(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    document: &Document,
    node: bibtex::SyntaxNode,
) -> Option<()> {
    let field = bibtex::Field::cast(node)?;
    if field.eq_token().is_none() {
        let code = BibtexCode::ExpectingEq;
        all_diagnostics
            .entry(Arc::clone(document.uri()))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index()
                    .line_col_lsp_range(field.name_token()?.text_range()),

                code: DiagnosticCode::Bibtex(code),
                message: String::from(code),
            });

        return Some(());
    }

    if field.value().is_none() {
        let code = BibtexCode::ExpectingFieldValue;
        all_diagnostics
            .entry(Arc::clone(document.uri()))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index()
                    .line_col_lsp_range(field.name_token()?.text_range()),

                code: DiagnosticCode::Bibtex(code),
                message: String::from(code),
            });

        return Some(());
    }

    Some(())
}
