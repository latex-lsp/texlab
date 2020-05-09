mod bibutils;
mod name;
mod ris;

use self::{
    bibutils::{InputFormat, OutputFormat},
    ris::{RisLibrary, RisReference},
};
use crate::{
    protocol::{BibtexFormattingOptions, MarkupContent, MarkupKind},
    syntax::bibtex,
};
use citeproc::prelude::*;
use citeproc_db::PredefinedLocales;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;

static APA_STYLE: &str = include_str!("apa.csl");

static DOI_URL_PATTERN: &str = r#"https://doi.org/\[.*\]\(.*\)"#;

static DOI_URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(DOI_URL_PATTERN).unwrap());

pub fn render_citation(tree: &bibtex::Tree, key: &str) -> Option<MarkupContent> {
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

fn convert_to_ris(tree: &bibtex::Tree, key: &str) -> Option<RisReference> {
    let options = BibtexFormattingOptions {
        line_length: None,
        formatter: None,
    };
    let params = bibtex::FormattingParams {
        insert_spaces: true,
        tab_size: 4,
        options: &options,
    };

    let mut bib_code = String::new();
    tree.children(tree.root)
        .filter(|node| tree.as_string(*node).is_some())
        .map(|node| bibtex::format(tree, node, params))
        .for_each(|string| {
            bib_code.push_str(&string);
            bib_code.push('\n');
        });

    let entry = tree.entry_by_key(key)?;
    if let Some(crossref) = tree.crossref(entry) {
        bib_code.push_str(&bibtex::format(tree, crossref, params));
        bib_code.push('\n');
    }

    bib_code.push_str(&bibtex::format(tree, entry, params));
    bib_code.push('\n');
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

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn simple() {
        let tree = bibtex::open(indoc!(
            r#"
                @article{foo, 
                    author = {Foo Bar}, 
                    title = {Baz Qux},
                    year = {2020}
                }
            "#
        ));

        let actual_md = render_citation(&tree, "foo").unwrap();

        let expected_md = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, F. (2020). *Baz Qux*.".into(),
        };

        assert_eq!(actual_md, expected_md);
    }

    #[test]
    fn crossref() {
        let tree = bibtex::open(indoc!(
            r#"
                https://tex.stackexchange.com/questions/401138/what-is-the-bibtex-crossref-field-used-for

                @inproceedings{duck2015,
                    author = {Duck, D.},
                    title = {Duck tales},
                    crossref = {ICRC2015},
                }
                
                @inproceedings{mouse2015,
                    author = {Mouse, M.},
                    title = {Mouse stories},
                    crossref = {ICRC2015},
                }
                
                @proceedings{ICRC2015,
                    title = "{Proceedings of the 34\textsuperscript{th} International Cosmic Ray Conference}",
                    year = "2015",
                    month = aug,
                }
            "#
        ));

        let actual_md = render_citation(&tree, "mouse2015").unwrap();

        let expected_md = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Mouse, M. (2015). Mouse stories. In *Proceedings of the 34th International Cosmic Ray Conference*.".into(),
        };

        assert_eq!(actual_md, expected_md);
    }

    #[test]
    fn string() {
        let tree = bibtex::open(indoc!(
            r#"
                @string{author = "Foo Bar"}
                @article{foo, 
                    author = author, 
                    title = {Baz Qux},
                    year = {2020}
                }
            "#
        ));

        let actual_md = render_citation(&tree, "foo").unwrap();

        let expected_md = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, F. (2020). *Baz Qux*.".into(),
        };

        assert_eq!(actual_md, expected_md);
    }

    #[test]
    fn unknown_key() {
        let tree = bibtex::open("");

        let actual_md = render_citation(&tree, "foo");

        assert_eq!(actual_md, None);
    }
}
