mod bibtex_internal;
mod latexindent;

use distro::Language;
use lsp_types::{FormattingOptions, TextEdit, Url};

use crate::{db::Workspace, Db, Formatter};

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

pub fn format_source_code(
    db: &dyn Db,
    uri: &Url,
    options: &FormattingOptions,
) -> Option<Vec<TextEdit>> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;
    match document.language(db) {
        Language::Tex => match db.config().formatting.tex_formatter {
            Formatter::Null => None,
            Formatter::Server => None,
            Formatter::LatexIndent => format_with_latexindent(db, document),
        },
        Language::Bib => match db.config().formatting.bib_formatter {
            Formatter::Null => None,
            Formatter::Server => format_bibtex_internal(db, document, options),
            Formatter::LatexIndent => format_with_latexindent(db, document),
        },
        Language::Log | Language::Root | Language::Tectonic => None,
    }
}
