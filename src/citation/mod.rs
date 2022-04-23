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
    if markdown.is_empty() {
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

    use insta::assert_snapshot;

    fn render_simple(text: &str) -> String {
        let root = bibtex::SyntaxNode::new_root(bibtex::parse(text).green);
        let key = root
            .children()
            .find_map(bibtex::Entry::cast)
            .and_then(|entry| entry.key())
            .unwrap()
            .to_string();

        render_citation(&root, &key).unwrap().value
    }

    #[test]
    fn test_biblatex_examples_001() {
        assert_snapshot!(render_simple(
            r#"
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
}

% booktitle and booksubtitle are only needed for BibTeX's less sophisticated
% inheritance set-up to make sure westfahl:space shows correctly.
% With Biber they are not needed.
@collection{westfahl:frontier,
  editor       = {Westfahl, Gary},
  title        = {Space and Beyond},
  date         = 2000,
  subtitle     = {The Frontier Theme in Science Fiction},
  publisher    = {Greenwood},
  location     = {Westport, Conn. and London},
  langid       = {english},
  langidopts   = {variant=american},
  booktitle    = {Space and Beyond},
  booksubtitle = {The Frontier Theme in Science Fiction},
  annotation   = {This is a \texttt{collection} entry. Note the format of the
                  \texttt{location} field as well as the \texttt{subtitle}
                  field},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_002() {
        assert_snapshot!(render_simple(
            r#"
@string{jomch   = {J.~Organomet. Chem.}}

@article{aksin,
  author       = {Aks{\i}n, {\"O}zge and T{\"u}rkmen, Hayati and Artok, Levent
                  and {\c{C}}etinkaya, Bekir and Ni, Chaoying and
                  B{\"u}y{\"u}kg{\"u}ng{\"o}r, Orhan and {\"O}zkal, Erhan},
  title        = {Effect of immobilization on catalytic characteristics of
                  saturated {Pd-N}-heterocyclic carbenes in {Mizoroki-Heck}
                  reactions},
  journaltitle = jomch,
  date         = 2006,
  volume       = 691,
  number       = 13,
  pages        = {3027-3036},
  indextitle   = {Effect of immobilization on catalytic characteristics},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_003() {
        assert_snapshot!(render_simple(
            r#"
@article{angenendt,
    author       = {Angenendt, Arnold},
    title        = {In Honore Salvatoris~-- Vom Sinn und Unsinn der
                    Patrozinienkunde},
    journaltitle = {Revue d'Histoire Eccl{\'e}siastique},
    date         = 2002,
    volume       = 97,
    pages        = {431--456, 791--823},
    langid       = {german},
    indextitle   = {In Honore Salvatoris},
    shorttitle   = {In Honore Salvatoris},
    annotation   = {A German article in a French journal. Apart from that, a
                    typical \texttt{article} entry. Note the \texttt{indextitle}
                    field},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_004() {
        assert_snapshot!(render_simple(
            r#"
@article{baez/article,
    author       = {Baez, John C. and Lauda, Aaron D.},
    title        = {Higher-Dimensional Algebra {V}: 2-Groups},
    journaltitle = {Theory and Applications of Categories},
    date         = 2004,
    volume       = 12,
    pages        = {423-491},
    version      = 3,
    eprint       = {math/0307200v3},
    eprinttype   = {arxiv},
    langid       = {english},
    langidopts   = {variant=american},
    annotation   = {An \texttt{article} with \texttt{eprint} and
                    \texttt{eprinttype} fields. Note that the arXiv reference is
                    transformed into a clickable link if \texttt{hyperref} support
                    has been enabled.  Compare \texttt{baez\slash online}, which
                    is the same item given as an \texttt{online} entry},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_005() {
        assert_snapshot!(render_simple(
            r#"
@string{jams    = {J.~Amer. Math. Soc.}}

@article{bertram,
    author       = {Bertram, Aaron and Wentworth, Richard},
    title        = {Gromov invariants for holomorphic maps on {Riemann} surfaces},
    journaltitle = jams,
    date         = 1996,
    volume       = 9,
    number       = 2,
    pages        = {529-571},
    langid       = {english},
    langidopts   = {variant=american},
    shorttitle   = {Gromov invariants},
    annotation   = {An \texttt{article} entry with a \texttt{volume} and a
                    \texttt{number} field},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_006() {
        assert_snapshot!(render_simple(
            r#"
@article{doody,
    author       = {Doody, Terrence},
    title        = {Hemingway's Style and {Jake's} Narration},
    year         = 1974,
    volume       = 4,
    number       = 3,
    pages        = {212-225},
    langid       = {english},
    langidopts   = {variant=american},
    related      = {matuz:doody},
    relatedstring= {\autocap{e}xcerpt in},
    journal      = {The Journal of Narrative Technique},
    annotation   = {An \texttt{article} entry cited as an excerpt from a
                    \texttt{collection} entry. Note the format of the
                    \texttt{related} and \texttt{relatedstring} fields},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_007() {
        assert_snapshot!(render_simple(
            r#"
@article{gillies,
    author       = {Gillies, Alexander},
    title        = {Herder and the Preparation of {Goethe's} Idea of World
                    Literature},
    journaltitle = {Publications of the English Goethe Society},
    date         = 1933,
    series       = {newseries},
    volume       = 9,
    pages        = {46-67},
    langid       = {english},
    langidopts   = {variant=british},
    annotation   = {An \texttt{article} entry with a \texttt{series} and a
                    \texttt{volume} field. Note that format of the \texttt{series}
                    field in the database file},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_008() {
        assert_snapshot!(render_simple(
            r#"
@article{glashow,
    author       = {Glashow, Sheldon},
    title        = {Partial Symmetries of Weak Interactions},
    journaltitle = {Nucl.~Phys.},
    date         = 1961,
    volume       = 22,
    pages        = {579-588},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_009() {
        assert_snapshot!(render_simple(
            r#"
@string{anch-ie = {Angew.~Chem. Int.~Ed.}}

@article{herrmann,
    author       = {Herrmann, Wolfgang A. and {\"O}fele, Karl and Schneider,
                    Sabine K.  and Herdtweck, Eberhardt and Hoffmann, Stephan D.},
    title        = {A carbocyclic carbene as an efficient catalyst ligand for {C--C}
                    coupling reactions},
    journaltitle = anch-ie,
    date         = 2006,
    volume       = 45,
    number       = 23,
    pages        = {3859-3862},
    indextitle   = {Carbocyclic carbene as an efficient catalyst, A},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_010() {
        assert_snapshot!(render_simple(
            r#"
@string{jchph   = {J.~Chem. Phys.}}

@article{kastenholz,
    author       = {Kastenholz, M. A. and H{\"u}nenberger, Philippe H.},
    title        = {Computation of methodology\hyphen independent ionic solvation
                    free energies from molecular simulations},
    journaltitle = jchph,
    date         = 2006,
    subtitle     = {{I}. {The} electrostatic potential in molecular liquids},
    volume       = 124,
    eid          = 124106,
    doi          = {10.1063/1.2172593},
    langid       = {english},
    langidopts   = {variant=american},
    indextitle   = {Computation of ionic solvation free energies},
    annotation   = {An \texttt{article} entry with an \texttt{eid} and a
                    \texttt{doi} field. Note that the \textsc{doi} is transformed
                    into a clickable link if \texttt{hyperref} support has been
                    enabled},
    abstract     = {The computation of ionic solvation free energies from
                    atomistic simulations is a surprisingly difficult problem that
                    has found no satisfactory solution for more than 15 years. The
                    reason is that the charging free energies evaluated from such
                    simulations are affected by very large errors. One of these is
                    related to the choice of a specific convention for summing up
                    the contributions of solvent charges to the electrostatic
                    potential in the ionic cavity, namely, on the basis of point
                    charges within entire solvent molecules (M scheme) or on the
                    basis of individual point charges (P scheme). The use of an
                    inappropriate convention may lead to a charge-independent
                    offset in the calculated potential, which depends on the
                    details of the summation scheme, on the quadrupole-moment
                    trace of the solvent molecule, and on the approximate form
                    used to represent electrostatic interactions in the
                    system. However, whether the M or P scheme (if any) represents
                    the appropriate convention is still a matter of on-going
                    debate. The goal of the present article is to settle this
                    long-standing controversy by carefully analyzing (both
                    analytically and numerically) the properties of the
                    electrostatic potential in molecular liquids (and inside
                    cavities within them).},  
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_011() {
        assert_snapshot!(render_simple(
            r#"
@article{murray,
    author       = {Hostetler, Michael J. and Wingate, Julia E. and Zhong,
                    Chuan-Jian and Harris, Jay E. and Vachet, Richard W. and
                    Clark, Michael R.  and Londono, J. David and Green, Stephen
                    J. and Stokes, Jennifer J.  and Wignall, George D. and Glish,
                    Gary L. and Porter, Marc D.  and Evans, Neal D. and Murray,
                    Royce W.},
    title        = {Alkanethiolate gold cluster molecules with core diameters from
                    1.5 to 5.2~{nm}},
    journaltitle = {Langmuir},
    date         = 1998,
    subtitle     = {Core and monolayer properties as a function of core size},
    volume       = 14,
    number       = 1,
    pages        = {17-30},
    langid       = {english},
    langidopts   = {variant=american},
    indextitle   = {Alkanethiolate gold cluster molecules},
    shorttitle   = {Alkanethiolate gold cluster molecules},
    annotation   = {An \texttt{article} entry with \arabic{author} authors. By
                    default, long author and editor lists are automatically
                    truncated. This is configurable},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_012() {
        assert_snapshot!(render_simple(
            r#"
@article{reese,
    author       = {Reese, Trevor R.},
    title        = {Georgia in {Anglo-Spanish} Diplomacy, 1736--1739},
    journaltitle = {William and Mary Quarterly},
    date         = 1958,
    series       = 3,
    volume       = 15,
    pages        = {168-190},
    langid       = {english},
    langidopts   = {variant=american},
    annotation   = {An \texttt{article} entry with a \texttt{series} and a
                    \texttt{volume} field. Note the format of the series. If the
                    value of the \texttt{series} field is an integer, this number
                    is printed as an ordinal and the string \enquote*{series} is
                    appended automatically},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_013() {
        assert_snapshot!(render_simple(
            r#"
@article{sarfraz,
    author       = {M. Sarfraz and M. F. A. Razzak},
    title        = {Technical section: {An} algorithm for automatic capturing of
                    the font outlines},
    year         = 2002,
    volume       = 26,
    number       = 5,
    pages        = {795-804},
    issn         = {0097-8493},
    journal      = {Computers and Graphics},
    annotation   = {An \texttt{article} entry with an \texttt{issn} field},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_014() {
        assert_snapshot!(render_simple(
            r#"
@article{shore,
    author       = {Shore, Bradd},
    title        = {Twice-Born, Once Conceived},
    journaltitle = {American Anthropologist},
    date         = {1991-03},
    subtitle     = {Meaning Construction and Cultural Cognition},
    series       = {newseries},
    volume       = 93,
    number       = 1,
    pages        = {9-27},
    annotation   = {An \texttt{article} entry with \texttt{series},
                    \texttt{volume}, and \texttt{number} fields. Note the format
                    of the \texttt{series} which is a localization key},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_015() {
        assert_snapshot!(render_simple(
            r#"
@article{sigfridsson,
    author       = {Sigfridsson, Emma and Ryde, Ulf},
    title        = {Comparison of methods for deriving atomic charges from the
                    electrostatic potential and moments},
    journaltitle = {Journal of Computational Chemistry},
    date         = 1998,
    volume       = 19,
    number       = 4,
    pages        = {377-395},
    doi          = {10.1002/(SICI)1096-987X(199803)19:4<377::AID-JCC1>3.0.CO;2-P},
    langid       = {english},
    langidopts   = {variant=american},
    indextitle   = {Methods for deriving atomic charges},
    annotation   = {An \texttt{article} entry with \texttt{volume},
                    \texttt{number}, and \texttt{doi} fields. Note that the
                    \textsc{doi} is transformed into a clickable link if
                    \texttt{hyperref} support has been enabled},
    abstract     = {Four methods for deriving partial atomic charges from the
                    quantum chemical electrostatic potential (CHELP, CHELPG,
                    Merz-Kollman, and RESP) have been compared and critically
                    evaluated. It is shown that charges strongly depend on how and
                    where the potential points are selected. Two alternative
                    methods are suggested to avoid the arbitrariness in the
                    point-selection schemes and van der Waals exclusion radii:
                    CHELP-BOW, which also estimates the charges from the
                    electrostatic potential, but with potential points that are
                    Boltzmann-weighted after their occurrence in actual
                    simulations using the energy function of the program in which
                    the charges will be used, and CHELMO, which estimates the
                    charges directly from the electrostatic multipole
                    moments. Different criteria for the quality of the charges are
                    discussed.},
}"#,
        ));
    }

    #[test]
    fn test_biblatex_examples_016() {
        assert_snapshot!(render_simple(
            r#"
@article{spiegelberg,
    author       = {Spiegelberg, Herbert},
    title        = {\mkbibquote{Intention} und \mkbibquote{Intentionalit{\"a}t} in
                    der Scholastik, bei Brentano und Husserl},
    journaltitle = {Studia Philosophica},
    date         = 1969,
    volume       = 29,
    pages        = {189-216},
    langid       = {german},
    sorttitle    = {Intention und Intentionalitat in der Scholastik, bei Brentano
                    und Husserl},
    indexsorttitle= {Intention und Intentionalitat in der Scholastik, bei Brentano
                    und Husserl},
    shorttitle   = {Intention und Intentionalit{\"a}t},
    annotation   = {An \texttt{article} entry. Note the \texttt{sorttitle} and
                    \texttt{indexsorttitle} fields and the markup of the quotes in
                    the database file},
}"#,
        ));
    }
}
