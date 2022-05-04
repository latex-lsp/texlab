use insta::assert_debug_snapshot;

use super::{parse, SyntaxNode};

fn setup(input: &str) -> SyntaxNode {
    SyntaxNode::new_root(parse(input))
}

#[test]
fn test_preamble() {
    assert_debug_snapshot!(setup(r#"@preamble{"Hello World!"}"#));
}

#[test]
fn test_comment() {
    assert_debug_snapshot!(setup(
        r#"@comment{aksin,
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
}"#
    ));
}

#[test]
fn test_entry_colon() {
    assert_debug_snapshot!(setup(
        r#"@article{foo:2019,
    author = {Foo Bar},
    title = {Baz Qux},
    year = {2019},
}"#
    ));
}

#[test]
fn test_biblatex_examples_001() {
    assert_debug_snapshot!(setup(
        r#"@incollection{westfahl:space,
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
}"#
    ));
}

#[test]
fn test_biblatex_examples_002() {
    assert_debug_snapshot!(setup(
        r#"@string{jomch   = {J.~Organomet. Chem.}}

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
}"#
    ));
}

#[test]
fn test_biblatex_examples_003() {
    assert_debug_snapshot!(setup(
        r#"@article{angenendt,
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
}"#
    ));
}

// #[test]
// fn foo() {
//     panic!(
//         "{:#?}",
//         setup(
//             r#"@article{foo,
//     titl"#
//         )
//     )
// }
