mod bibtex_internal;
mod latexindent;

use lsp_types::{FormattingOptions, TextEdit, Url};

use crate::{
    db::{document::Language, workspace::Workspace},
    BibtexFormatter, Db, LatexFormatter,
};

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

pub fn format_source_code(
    db: &dyn Db,
    uri: &Url,
    options: &FormattingOptions,
) -> Option<Vec<TextEdit>> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;
    match document.language(db) {
        Language::Tex => match workspace.options(db).latex_formatter {
            LatexFormatter::Texlab => None,
            LatexFormatter::Latexindent => format_with_latexindent(db, document),
        },
        Language::Bib => match workspace.options(db).bibtex_formatter {
            BibtexFormatter::Texlab => format_bibtex_internal(db, document, options),
            BibtexFormatter::Latexindent => format_with_latexindent(db, document),
        },
        Language::Log => None,
    }
}
