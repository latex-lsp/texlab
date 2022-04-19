mod bibtex_internal;
mod latexindent;

use lsp_types::{DocumentFormattingParams, TextEdit};

use crate::{BibtexFormatter, LatexFormatter};

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

use super::FeatureRequest;

pub fn format_source_code(
    request: FeatureRequest<DocumentFormattingParams>,
) -> Option<Vec<TextEdit>> {
    let mut edits = None;
    if request.workspace.environment.options.bibtex_formatter == BibtexFormatter::Texlab {
        edits = edits.or_else(|| format_bibtex_internal(&request));
    }

    if request.workspace.environment.options.latex_formatter == LatexFormatter::Texlab {
        edits = edits.or_else(|| Some(vec![]));
    }

    edits = edits.or_else(|| format_with_latexindent(&request));
    edits
}
