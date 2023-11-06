use base_db::{Document, Workspace};
use lsp_types::{FormattingOptions, TextEdit};
use rowan::{TextLen, TextRange};

use crate::util::line_index_ext::LineIndexExt;

pub fn format_bibtex_internal(
    workspace: &Workspace,
    document: &Document,
    options: &FormattingOptions,
) -> Option<Vec<TextEdit>> {
    let data = document.data.as_bib()?;
    let options = bibfmt::Options {
        insert_spaces: options.insert_spaces,
        line_length: workspace.config().formatting.line_length,
        tab_size: options.tab_size as usize,
    };

    let output = bibfmt::format(&data.root_node(), &document.line_index, &options);
    let range = TextRange::new(0.into(), document.text.text_len());
    let range = document.line_index.line_col_lsp_range(range)?;
    Some(vec![lsp_types::TextEdit::new(range, output)])
}
