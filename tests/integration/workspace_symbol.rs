use indoc::indoc;
use texlab::test::{TestBed, TestBedBuilder, TestLspClient, PULL_CAPABILITIES};
use texlab_protocol::{Location, Range, RangeExt, SymbolInformation, WorkspaceSymbolParams};

pub fn verify_symbol_info(
    symbol: &SymbolInformation,
    test_bed: &TestBed,
    relative_path: &str,
    name: &str,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
) {
    assert_eq!(symbol.name, name);
    let range = Range::new_simple(start_line, start_character, end_line, end_character);
    assert_eq!(
        symbol.location,
        Location::new(test_bed.uri(relative_path).into(), range)
    );
}

async fn run(query: &str) -> (TestBed, Vec<SymbolInformation>) {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \usepackage{caption}
                    \usepackage{amsmath}
                    \usepackage{amsthm}
                    %
                    \begin{document}
                    %
                    \section{Foo}\label{sec:foo}
                    %
                    \begin{equation}\label{eq:foo}
                        Foo
                    \end{equation}
                    %
                    \section{Bar}\label{sec:bar}
                    %
                    \begin{figure}
                        Bar
                        \caption{Bar}
                        \label{fig:bar}
                    \end{figure}
                    %
                    \section{Baz}\label{sec:baz}
                    %
                    \begin{enumerate}
                        \item\label{itm:foo} Foo
                        \item\label{itm:bar} Bar
                        \item\label{itm:baz} Baz    
                    \end{enumerate}
                    %
                    \section{Qux}\label{sec:qux}
                    %
                    \newtheorem{lemma}{Lemma}
                    %
                    \begin{lemma}[Qux]\label{thm:qux}
                        Qux
                    \end{lemma}
                    %
                    \end{document}
                "#
            ),
        )
        .file(
            "foo.aux",
            indoc!(
                r#"
                    \relax 
                    \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Bar\relax }}{1}\protected@file@percent }
                    \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
                    \newlabel{fig:bar}{{1}{1}}
                    \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
                    \newlabel{sec:foo}{{1}{1}}
                    \newlabel{eq:foo}{{1}{1}}
                    \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
                    \newlabel{sec:bar}{{2}{1}}
                    \@writefile{toc}{\contentsline {section}{\numberline {3}Baz}{1}\protected@file@percent }
                    \newlabel{sec:baz}{{3}{1}}
                    \newlabel{itm:foo}{{1}{1}}
                    \newlabel{itm:bar}{{2}{1}}
                    \newlabel{itm:baz}{{3}{1}}
                    \@writefile{toc}{\contentsline {section}{\numberline {4}Qux}{1}\protected@file@percent }
                    \newlabel{sec:qux}{{4}{1}}
                    \newlabel{thm:qux}{{1}{1}}
                "#
            ),
        )
        .file(
            "bar.bib",
            indoc!(
                r#"
                    @article{foo,}
                    %
                    @string{bar = "bar"}                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("foo.tex").await;
    test_bed.open("foo.aux").await;
    test_bed.open("bar.bib").await;

    let params = WorkspaceSymbolParams {
        query: query.into(),
        ..WorkspaceSymbolParams::default()
    };
    let actual_symbols = test_bed.client.workspace_symbol(params).await.unwrap();

    test_bed.shutdown().await;

    (test_bed, actual_symbols)
}

#[tokio::test]
async fn filter_type_section() {
    let (test_bed, actual_symbols) = run("section").await;
    assert_eq!(actual_symbols.len(), 4);
    verify_symbol_info(
        &actual_symbols[0],
        &test_bed,
        "foo.tex",
        "1 Foo",
        7,
        0,
        13,
        0,
    );
    verify_symbol_info(
        &actual_symbols[1],
        &test_bed,
        "foo.tex",
        "2 Bar",
        13,
        0,
        21,
        0,
    );
    verify_symbol_info(
        &actual_symbols[2],
        &test_bed,
        "foo.tex",
        "3 Baz",
        21,
        0,
        29,
        0,
    );
    verify_symbol_info(
        &actual_symbols[3],
        &test_bed,
        "foo.tex",
        "4 Qux",
        29,
        0,
        37,
        0,
    );
}

#[tokio::test]
async fn filter_type_figure() {
    let (test_bed, actual_symbols) = run("figure").await;
    assert_eq!(actual_symbols.len(), 1);
    verify_symbol_info(
        &actual_symbols[0],
        &test_bed,
        "foo.tex",
        "Figure 1: Bar",
        15,
        0,
        19,
        12,
    );
}

#[tokio::test]
async fn filter_type_item() {
    let (test_bed, actual_symbols) = run("item").await;
    assert_eq!(actual_symbols.len(), 3);
    verify_symbol_info(&actual_symbols[0], &test_bed, "foo.tex", "1", 24, 4, 25, 4);
    verify_symbol_info(&actual_symbols[1], &test_bed, "foo.tex", "2", 25, 4, 26, 4);
    verify_symbol_info(&actual_symbols[2], &test_bed, "foo.tex", "3", 26, 4, 27, 0);
}

#[tokio::test]
async fn filter_type_math() {
    let (test_bed, actual_symbols) = run("math").await;
    assert_eq!(actual_symbols.len(), 2);
    verify_symbol_info(
        &actual_symbols[0],
        &test_bed,
        "foo.tex",
        "Equation (1)",
        9,
        0,
        11,
        14,
    );
    verify_symbol_info(
        &actual_symbols[1],
        &test_bed,
        "foo.tex",
        "Lemma 1 (Qux)",
        33,
        0,
        35,
        11,
    );
}

#[tokio::test]
async fn filter_bibtex() {
    let (test_bed, actual_symbols) = run("bibtex").await;
    assert_eq!(actual_symbols.len(), 2);
    verify_symbol_info(&actual_symbols[0], &test_bed, "bar.bib", "foo", 0, 0, 0, 14);
    verify_symbol_info(&actual_symbols[1], &test_bed, "bar.bib", "bar", 2, 0, 2, 20);
}
