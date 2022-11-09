use lsp_types::DiagnosticSeverity;
use rowan::{ast::AstNode, TextRange};

use crate::{
    db::document::Document,
    syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue},
    util::line_index_ext::LineIndexExt,
    Db,
};

use super::{BibCode, Diagnostic, DiagnosticCode};

#[salsa::tracked(return_ref)]
pub fn collect(db: &dyn Db, document: Document) -> Vec<Diagnostic> {
    let mut results = Vec::new();
    let data = match document.parse(db).as_bib() {
        Some(data) => data,
        None => return results,
    };

    for node in data.root(db).descendants() {
        analyze_entry(db, document, node.clone(), &mut results)
            .or_else(|| analyze_field(db, document, node, &mut results));
    }

    results
}

fn analyze_entry(
    db: &dyn Db,
    document: Document,
    node: bibtex::SyntaxNode,
    results: &mut Vec<Diagnostic>,
) -> Option<()> {
    let line_index = document.contents(db).line_index(db);

    let entry = bibtex::Entry::cast(node)?;
    if entry.left_delim_token().is_none() {
        let code = BibCode::ExpectingLCurly;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: line_index.line_col_lsp_range(entry.type_token()?.text_range()),
            code: DiagnosticCode::Bib(code),
            message: String::from(code),
        });

        return Some(());
    }

    if entry.name_token().is_none() {
        let code = BibCode::ExpectingKey;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: line_index.line_col_lsp_range(entry.left_delim_token()?.text_range()),
            code: DiagnosticCode::Bib(code),
            message: String::from(code),
        });

        return Some(());
    }

    if entry.right_delim_token().is_none() {
        let code = BibCode::ExpectingRCurly;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: line_index
                .line_col_lsp_range(TextRange::empty(entry.syntax().text_range().end())),
            code: DiagnosticCode::Bib(code),
            message: String::from(code),
        });

        return Some(());
    }

    Some(())
}

fn analyze_field(
    db: &dyn Db,
    document: Document,
    node: bibtex::SyntaxNode,
    results: &mut Vec<Diagnostic>,
) -> Option<()> {
    let line_index = document.contents(db).line_index(db);

    let field = bibtex::Field::cast(node)?;
    if field.eq_token().is_none() {
        let code = BibCode::ExpectingEq;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: line_index.line_col_lsp_range(field.name_token()?.text_range()),
            code: DiagnosticCode::Bib(code),
            message: String::from(code),
        });

        return Some(());
    }

    if field.value().is_none() {
        let code = BibCode::ExpectingFieldValue;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: line_index.line_col_lsp_range(field.name_token()?.text_range()),
            code: DiagnosticCode::Bib(code),
            message: String::from(code),
        });

        return Some(());
    }

    Some(())
}
