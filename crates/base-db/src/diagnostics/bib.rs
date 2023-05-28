use rowan::{ast::AstNode, TextRange};
use syntax::bibtex::{self, HasDelims, HasEq, HasName, HasType, HasValue};

use crate::{Document, DocumentData};

use super::{Diagnostic, ErrorCode};

pub fn analyze(document: &mut Document) {
    let DocumentData::Bib(data) = &document.data else { return };

    for node in bibtex::SyntaxNode::new_root(data.green.clone()).descendants() {
        if let Some(entry) = bibtex::Entry::cast(node.clone()) {
            analyze_entry(document, entry);
        } else if let Some(field) = bibtex::Field::cast(node.clone()) {
            analyze_field(document, field);
        }
    }
}

fn analyze_entry(document: &mut Document, entry: bibtex::Entry) {
    if entry.left_delim_token().is_none() {
        document.diagnostics.push(Diagnostic {
            range: entry.type_token().unwrap().text_range(),
            code: ErrorCode::ExpectingLCurly,
        });

        return;
    }

    if entry.name_token().is_none() {
        document.diagnostics.push(Diagnostic {
            range: entry.left_delim_token().unwrap().text_range(),
            code: ErrorCode::ExpectingKey,
        });

        return;
    }

    if entry.right_delim_token().is_none() {
        document.diagnostics.push(Diagnostic {
            range: TextRange::empty(entry.syntax().text_range().end()),
            code: ErrorCode::ExpectingRCurly,
        });
    }
}

fn analyze_field(document: &mut Document, field: bibtex::Field) {
    if field.eq_token().is_none() {
        let code = ErrorCode::ExpectingEq;
        document.diagnostics.push(Diagnostic {
            range: field.name_token().unwrap().text_range(),
            code,
        });

        return;
    }

    if field.value().is_none() {
        let code = ErrorCode::ExpectingFieldValue;
        document.diagnostics.push(Diagnostic {
            range: field.name_token().unwrap().text_range(),
            code,
        });
    }
}
