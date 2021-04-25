mod bibtex_internal;
mod latexindent;

use cancellation::CancellationToken;
use lsp_types::{DocumentFormattingParams, TextEdit};

use crate::BibtexFormatter;

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

use super::FeatureRequest;

pub fn format_source_code(
    request: FeatureRequest<DocumentFormattingParams>,
    cancellation_token: &CancellationToken,
) -> Option<Vec<TextEdit>> {
    let mut edits = None;
    if request
        .context
        .options
        .read()
        .unwrap()
        .bibtex_formatter
        .unwrap_or_default()
        == BibtexFormatter::Texlab
    {
        edits = edits.or_else(|| format_bibtex_internal(&request, cancellation_token));
    }

    edits = edits.or_else(|| format_with_latexindent(&request, cancellation_token));
    edits
}
