mod name;
mod ris;

use self::ris::*;
use crate::formatting::bibtex::{format_entry, format_string, BibtexFormattingParams};
use crate::syntax::*;
use bibutils::{InputFormat, OutputFormat};
use citeproc::prelude::*;
use citeproc_db::PredefinedLocales;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use texlab_protocol::{MarkupContent, MarkupKind};

static APA_STYLE: &str = include_str!("apa.csl");

static DOI_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"https://doi.org/\[.*\]\(.*\)"#).unwrap());

pub fn render_citation(tree: &BibtexSyntaxTree, key: &str) -> Option<MarkupContent> {
    let ris_reference = convert_to_ris(tree, key)?;
    let doi_url = ris_reference
        .doi
        .as_ref()
        .map(|doi| format!("[doi:{}](https://doi.org/{})", doi, doi));

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

    let entry = tree.find_entry(key)?;
    if let Some(crossref) = tree.resolve_crossref(entry) {
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

    // The test cases are taken from biblatex-examples.bib.
    // You can obtain the path with:
    // $ kpsewhich biblatex-examples.bib

    #[test]
    fn test_article() {
        let tree = BibtexSyntaxTree::from(
            r#"
            @article{angenendt,
                author       = {Angenendt, Arnold},
                title        = {In Honore Salvatoris~-- Vom Sinn und Unsinn der
                                Patrozinienkunde},
                journaltitle = {Revue d'Histoire Eccl{\'e}siastique},
                date         = {2002},
                volume       = {97},
                pages        = {431--456, 791--823},
                langid       = {german},
                indextitle   = {In Honore Salvatoris},
                shorttitle   = {In Honore Salvatoris},
                annotation   = {A German article in a French journal. Apart from that, a
                                typical \texttt{article} entry. Note the \texttt{indextitle}
                                field},
            }"#,
        );
        let markdown = render_citation(&tree, "angenendt").unwrap().value;
        assert_eq!(markdown, "Angenendt, A. In Honore Salvatoris – Vom Sinn und Unsinn der Patrozinienkunde. *Revue d’Histoire Ecclésiastique*, *97*, 431\\-456,.");
    }

    #[test]
    fn test_article_doi() {
        let tree = BibtexSyntaxTree::from(
            r#"
            @article{kastenholz,
                author       = {Kastenholz, M. A. and H{\"u}nenberger, Philippe H.},
                title        = {Computation of methodology\hyphen independent ionic solvation
                                free energies from molecular simulations},
                journaltitle = jchph,
                date         = {2006},
                subtitle     = {{I}. {The} electrostatic potential in molecular liquids},
                volume       = {124},
                eid          = {124106},
                doi          = {10.1063/1.2172593},
                langid       = {english},
                langidopts   = {variant=american},
                indextitle   = {Computation of ionic solvation free energies},
                annotation   = {An \texttt{article} entry with an \texttt{eid} and a
                                \texttt{doi} field. Note that the \textsc{doi} is transformed
                                into a clickable link if \texttt{hyperref} support has been
                                enabled},
            }"#,
        );
        let markdown = render_citation(&tree, "kastenholz").unwrap().value;
        assert_eq!(markdown, "Kastenholz, M. A., & Hünenberger, P. H. Computation of methodology\\-independent ionic solvation free energies from molecular simulations: I. The electrostatic potential in molecular liquids. *jchph*, *124*. [doi:10.1063/1.2172593](https://doi.org/10.1063/1.2172593)");
    }

    #[test]
    fn test_book() {
        let tree = BibtexSyntaxTree::from(
            r#"
            @book{aristotle:physics,
                author       = {Aristotle},
                title        = {Physics},
                date         = {1929},
                translator   = {Wicksteed, P. H. and Cornford, F. M.},
                publisher    = {G. P. Putnam},
                location     = {New York},
                keywords     = {primary},
                langid       = {english},
                langidopts   = {variant=american},
                shorttitle   = {Physics},
                annotation   = {A \texttt{book} entry with a \texttt{translator} field},
            }"#,
        );
        let markdown = render_citation(&tree, "aristotle:physics").unwrap().value;
        assert_eq!(markdown, "Aristotle. *Physics*. New York: G. P. Putnam.");
    }

    #[test]
    fn test_book_string() {
        let tree = BibtexSyntaxTree::from(
            r#"
            @string{cup = {Cambridge University Press}}
            @book{aristotle:anima,
                author       = {Aristotle},
                title        = {De Anima},
                date         = {1907},
                editor       = {Hicks, Robert Drew},
                publisher    = cup,
                location     = {Cambridge},
                keywords     = {primary},
                langid       = {english},
                langidopts   = {variant=british},
                annotation   = {A \texttt{book} entry with an \texttt{author} and an
                                \texttt{editor}},
            }"#,
        );
        let markdown = render_citation(&tree, "aristotle:anima").unwrap().value;
        assert_eq!(
            markdown,
            "Aristotle. *De Anima* (R. D. Hicks, ed.). Cambridge: Cambridge University Press."
        )
    }

    #[test]
    fn test_crossref() {
        let tree = BibtexSyntaxTree::from(
            r#"
            @collection{westfahl:frontier,
                editor       = {Westfahl, Gary},
                title        = {Space and Beyond},
                date         = {2000},
                subtitle     = {The Frontier Theme in Science Fiction},
                publisher    = {Greenwood},
                location     = {Westport, Conn. and London},
                langid       = {english},
                langidopts   = {variant=american},
                booktitle    = {Space and Beyond},
                booksubtitle = {The Frontier Theme in Science Fiction},
                annotation   = {This is a \texttt{collection} entry. Note the format of the
                                \texttt{location} field as well as the \texttt{subtitle} and
                                \texttt{booksubtitle} fields},
            }
            
            @incollection{westfahl:space,
                author       = {Westfahl, Gary},
                title        = {The True Frontier},
                subtitle     = {Confronting and Avoiding the Realities of Space in {American}
                                Science Fiction Films},
                pages        = {55-65},
                crossref     = {westfahl:frontier},
                langid       = {english},
                langidopts   = {variant=american},
                indextitle   = {True Frontier, The},
                annotation   = {A cross-referenced article from a \texttt{collection}. This is
                                an \texttt{incollection} entry with a \texttt{crossref}
                                field. Note the \texttt{subtitle} and \texttt{indextitle}
                                fields},
            }"#,
        );
        let markdown = render_citation(&tree, "westfahl:space").unwrap().value;
        assert_eq!(markdown, "Westfahl, G. The True Frontier: Confronting and Avoiding the Realities of Space in American Science Fiction Films The Frontier Theme in Science Fiction. in G. Westfahl (ed.), *Space and Beyond: The Frontier Theme in Science Fiction* (p. 55\\-65). Westport, Conn. and London: Greenwood.");
    }
}
