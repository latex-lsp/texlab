mod bibtex_internal;
mod latexindent;

use lsp_types::{FormattingOptions, TextEdit, Url};

use crate::{
    db::{Language, Workspace},
    Db, Formatter,
};

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

pub fn format_source_code(
    db: &dyn Db,
    uri: &Url,
    options: &FormattingOptions,
) -> Option<Vec<TextEdit>> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;
    let config = workspace.config(db);
    match document.language(db) {
        Language::Tex => match config.formatting.tex_formatter {
            Formatter::Null => None,
            Formatter::Server => None,
            Formatter::LatexIndent => format_with_latexindent(db, document),
        },
        Language::Bib => match config.formatting.bib_formatter {
            Formatter::Null => None,
            Formatter::Server => format_bibtex_internal(db, document, options),
            Formatter::LatexIndent => format_with_latexindent(db, document),
        },
        Language::Log | Language::TexlabRoot | Language::Tectonic => None,
    }
}
