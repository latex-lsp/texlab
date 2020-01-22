mod name;
mod ris;

use self::ris::*;
use bibutils::{InputFormat, OutputFormat};
use citeproc::prelude::*;
use citeproc_db::PredefinedLocales;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use texlab_protocol::{MarkupContent, MarkupKind};
use texlab_syntax::*;

static APA_STYLE: &str = include_str!("apa.csl");

static DOI_URL_PATTERN: &str = r#"https://doi.org/\[.*\]\(.*\)"#;

static DOI_URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(DOI_URL_PATTERN).unwrap());

pub fn render_citation(tree: &BibtexSyntaxTree, key: &str) -> Option<MarkupContent> {
    let ris_reference = convert_to_ris(tree, key)?;
    let doi_url = get_doi_url_markdown(&ris_reference);
    let csl_reference: Reference = ris_reference.into();
    let html = generate_bibliography(csl_reference)?;

    let mut markdown = html2md::parse_html(&html).trim().to_owned();
    if markdown == "" {
        return None;
    }

    if let Some(doi_url) = doi_url {
        markdown = DOI_URL_REGEX
            .replace(&markdown, doi_url.as_str())
            .into_owned();
    }

    markdown = markdown.replace("..", ".");
    markdown = markdown.replace("\\\'", "'");
    markdown = markdown.replace("\\-", "-");
    let content = MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown,
    };
    Some(content)
}

fn convert_to_ris(tree: &BibtexSyntaxTree, key: &str) -> Option<RisReference> {
    let bib_params = BibtexFormattingParams::default();
    let mut bib_code = String::new();

    for string in tree.strings() {
        bib_code.push_str(&format_string(string, &bib_params));
        bib_code.push('\n');
    }

    let entry = tree.entry(key)?;
    if let Some(crossref) = tree.crossref(entry) {
        bib_code.push_str(&format_entry(crossref, &bib_params));
        bib_code.push('\n');
    }

    bib_code.push_str(&format_entry(entry, &bib_params));
    bib_code.push('\n');
    bib_code = bib_code.replace("\\hyphen ", "-");

    let ris_code = bibutils::convert(bib_code, InputFormat::Biblatex, OutputFormat::Ris)?;
    let ris_lib = RisLibrary::parse(ris_code.lines());
    ris_lib
        .references
        .into_iter()
        .find(|reference| reference.id.as_ref().map(AsRef::as_ref) == Some(key))
}

fn get_doi_url_markdown(ris_reference: &RisReference) -> Option<String> {
    ris_reference
        .doi
        .as_ref()
        .map(|doi| format!("[doi:{}](https://doi.org/{})", doi, doi))
}

fn generate_bibliography(reference: Reference) -> Option<String> {
    let locales = Arc::new(PredefinedLocales::bundled_en_us());
    let mut processor = Processor::new(APA_STYLE, locales, false, SupportedFormat::Html).unwrap();
    let cite = Cite::basic(&reference.id);
    let cluster = Cluster {
        id: 1,
        cites: vec![cite],
    };
    processor.insert_reference(reference);
    processor.init_clusters(vec![cluster]);
    processor
        .set_cluster_order(&[ClusterPosition {
            id: 1,
            note: Some(1),
        }])
        .unwrap();
    processor.get_bibliography().pop()
}
