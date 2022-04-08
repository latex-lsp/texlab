mod bibutils;
mod ris;

use std::sync::Arc;

use citeproc::{prelude::SupportedFormat, ClusterPosition, InitOptions, Processor};
use citeproc_db::PredefinedLocales;
use citeproc_io::{Cite, Reference};
use lsp_types::{MarkupContent, MarkupKind};
use once_cell::sync::Lazy;
use regex::Regex;
use rowan::ast::AstNode;

use crate::syntax::bibtex;

use self::{
    bibutils::*,
    ris::{RisLibrary, RisReference},
};

static APA_STYLE: &str = include_str!("apa.csl");

static DOI_URL_PATTERN: &str = r#"https://doi.org/\[.*\]\(.*\)"#;

static DOI_URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(DOI_URL_PATTERN).unwrap());

pub fn render_citation(root: &bibtex::SyntaxNode, key: &str) -> Option<MarkupContent> {
    let ris_reference = convert_to_ris(root, key)?;
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

    markdown = markdown
        .replace("..", ".")
        .replace("\\\'", "'")
        .replace("\\-", "-")
        .replace("\\\\textsubscript", "")
        .replace("\\\\textsuperscript", "");
    let content = MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown,
    };
    Some(content)
}

fn convert_to_ris(root: &bibtex::SyntaxNode, key: &str) -> Option<RisReference> {
    let mut bib_code = String::new();
    for string in root
        .children()
        .filter_map(bibtex::String::cast)
        .filter(|string| string.name().is_some())
    {
        bib_code.push_str(&string.syntax().to_string());
    }

    let entry = root
        .children()
        .filter_map(bibtex::Entry::cast)
        .find(|entry| entry.key().map(|key| key.to_string()).as_deref() == Some(key))
        .filter(|entry| entry.fields().next().is_some())?;

    bib_code.push_str(&entry.syntax().to_string());

    bib_code = bib_code.replace("\\hypen", "-");

    let ris_code = bibutils::convert(&bib_code, InputFormat::Biblatex, OutputFormat::Ris)?;
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
    let mut processor = Processor::new(InitOptions {
        style: APA_STYLE,
        format: SupportedFormat::Html,
        fetcher: Some(Arc::new(PredefinedLocales::bundled_en_us())),
        ..InitOptions::default()
    })
    .ok()?;
    let cite = Cite::basic(&reference.id);
    let cluster_id = processor.cluster_id("texlab");
    processor.insert_reference(reference);
    processor.insert_cites(cluster_id, &[cite]);
    processor
        .set_cluster_order(&[ClusterPosition {
            id: Some(cluster_id),
            note: Some(1),
        }])
        .unwrap();
    Some(processor.get_bibliography().pop()?.value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let document = bibtex::parse(
            r#"
                @article{foo,
                    author = {Foo Bar},
                    title = {Baz Qux},
                    year = {2020}
                }
            "#,
        );

        let actual_md =
            render_citation(&bibtex::SyntaxNode::new_root(document.green), "foo").unwrap();

        let expected_md = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, Foo. (2020). *Baz Qux*.".into(),
        };

        assert_eq!(actual_md, expected_md);
    }

    #[test]
    fn test_string() {
        let document = bibtex::parse(
            r#"
                @string{author = "Foo Bar"}
                @article{foo,
                    author = author,
                    title = {Baz Qux},
                    year = {2020}
                }
            "#,
        );
        let actual_md =
            render_citation(&bibtex::SyntaxNode::new_root(document.green), "foo").unwrap();

        let expected_md = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, Foo. (2020). *Baz Qux*.".into(),
        };

        assert_eq!(actual_md, expected_md);
    }

    #[test]
    fn test_unknown_key() {
        let document = bibtex::parse("");

        let actual_md = render_citation(&bibtex::SyntaxNode::new_root(document.green), "foo");

        assert_eq!(actual_md, None);
    }
}
