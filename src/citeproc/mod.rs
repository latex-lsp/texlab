mod name;
mod ris;

use self::ris::*;
use crate::formatting::bibtex::{format_entry, format_string, BibtexFormattingParams};
use crate::syntax::*;
use bibutils::{InputFormat, OutputFormat};
use citeproc::prelude::*;
use citeproc::Processor;
use citeproc_db::PredefinedLocales;
use citeproc_io::Reference;
use lsp_types::*;
use std::sync::Arc;

static APA_STYLE: &str = include_str!("apa.csl");

pub fn render_citation(tree: &BibtexSyntaxTree, key: &str) -> Option<MarkupContent> {
    let bib_params = BibtexFormattingParams::default();
    let mut bib_code = String::new();

    for string in tree.strings() {
        bib_code.push_str(&format_string(string, &bib_params));
        bib_code.push('\n');
    }

    let entry = tree.find_entry(key)?;

    if let Some(crossref_field) = entry.find_field("crossref") {
        if let Some(BibtexContent::BracedContent(crossref_content)) = &crossref_field.content {
            if let Some(BibtexContent::Word(crossref_name)) = crossref_content.children.get(0) {
                if let Some(crossref_entry) = tree.find_entry(crossref_name.token.text()) {
                    bib_code.push_str(&format_entry(crossref_entry, &bib_params));
                    bib_code.push('\n');
                }
            }
        }
    }

    bib_code.push_str(&format_entry(entry, &bib_params));
    bib_code.push('\n');

    let ris_code = unsafe { bibutils::convert(bib_code, InputFormat::Biblatex, OutputFormat::Ris) };
    let ris_lib = RisLibrary::parse(ris_code.lines());
    let ris_ref = ris_lib
        .references
        .into_iter()
        .find(|reference| reference.id.as_ref().map(AsRef::as_ref) == Some(key))?;

    let csl_ref: Reference = ris_ref.into();

    let locales = Arc::new(PredefinedLocales::bundled_en_us());
    let mut processor = Processor::new(APA_STYLE, locales, false, SupportedFormat::Html).unwrap();
    let mut clusters = Vec::new();
    let cite = Cite::basic(&csl_ref.id);
    clusters.push(Cluster2::Note {
        id: 1,
        note: IntraNote::Single(1),
        cites: vec![cite],
    });
    processor.insert_reference(csl_ref);
    processor.init_clusters(clusters);
    let html = processor.get_bibliography().pop().unwrap();
    let markdown = html2md::parse_html(&html).trim().to_owned();
    if markdown == "" {
        return None;
    }

    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown,
    })
}
