use indoc::indoc;
use texlab::test::{TestBedBuilder, NESTED_SYMBOL_CAPABILITIES};
use texlab_protocol::{DocumentSymbol, Range, RangeExt};

fn verify_symbol(
    symbol: &DocumentSymbol,
    name: &str,
    detail: Option<&str>,
    selection_range: Range,
    range: Range,
) {
    assert_eq!(symbol.name, name);
    assert_eq!(symbol.detail.as_ref().map(AsRef::as_ref), detail);
    assert_eq!(symbol.selection_range, selection_range);
    assert_eq!(symbol.range, range);
}

#[tokio::test]
async fn enumerate() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    %
                    \begin{document}
                    %
                    \begin{enumerate}
                        \item\label{it:foo} Foo
                        \item\label{it:bar} Bar
                        \item[Baz] Baz
                        \item[Qux]\label{it:qux} Qux
                    \end{enumerate}
                    %
                    \end{document}
                "#
            ),
        )
        .file(
            "main.aux",
            indoc!(
                r#"
                    \relax 
                    \newlabel{it:foo}{{1}{1}}
                    \newlabel{it:qux}{{2}{1}}
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed
        .initialize(NESTED_SYMBOL_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;
    test_bed.open("main.aux").await;

    let mut actual_symbols = test_bed.document_symbol_nested("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_symbols.len(), 1);
    verify_symbol(
        &actual_symbols[0],
        "Enumerate",
        None,
        Range::new_simple(4, 0, 9, 15),
        Range::new_simple(4, 0, 9, 15),
    );

    let children = actual_symbols[0].children.take().unwrap();
    assert_eq!(children.len(), 4);
    verify_symbol(
        &children[0],
        "1",
        Some("it:foo"),
        Range::new_simple(5, 9, 5, 23),
        Range::new_simple(5, 4, 6, 4),
    );
    verify_symbol(
        &children[1],
        "Item",
        Some("it:bar"),
        Range::new_simple(6, 9, 6, 23),
        Range::new_simple(6, 4, 7, 4),
    );
    verify_symbol(
        &children[2],
        "Baz",
        None,
        Range::new_simple(7, 4, 7, 14),
        Range::new_simple(7, 4, 8, 4),
    );
    verify_symbol(
        &children[3],
        "Qux",
        Some("it:qux"),
        Range::new_simple(8, 14, 8, 28),
        Range::new_simple(8, 4, 9, 0),
    );
}

#[tokio::test]
async fn equation() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    %
                    \begin{document}
                    %
                    \begin{equation}\label{eq:foo}
                        Foo
                    \end{equation}
                    %
                    \begin{equation}\label{eq:bar}
                        Bar
                    \end{equation}
                    %
                    \begin{equation}
                        Baz
                    \end{equation}
                    %
                    \end{document}                
                "#
            ),
        )
        .file(
            "main.aux",
            indoc!(
                r#"
                    \relax 
                    \newlabel{eq:foo}{{1}{1}}                
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed
        .initialize(NESTED_SYMBOL_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;
    test_bed.open("main.aux").await;

    let actual_symbols = test_bed.document_symbol_nested("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_symbols.len(), 3);
    verify_symbol(
        &actual_symbols[0],
        "Equation (1)",
        Some("eq:foo"),
        Range::new_simple(4, 16, 4, 30),
        Range::new_simple(4, 0, 6, 14),
    );
    verify_symbol(
        &actual_symbols[1],
        "Equation",
        Some("eq:bar"),
        Range::new_simple(8, 16, 8, 30),
        Range::new_simple(8, 0, 10, 14),
    );
    verify_symbol(
        &actual_symbols[2],
        "Equation",
        None,
        Range::new_simple(12, 0, 14, 14),
        Range::new_simple(12, 0, 14, 14),
    );
}

#[tokio::test]
async fn float() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    %
                    \begin{document}
                    %
                    \begin{figure}
                        Foo
                        \caption{Foo}\label{fig:foo}
                    \end{figure}
                    %
                    \begin{figure}
                        Bar
                        \caption{Bar}\label{fig:bar}
                    \end{figure}
                    %
                    \begin{figure}
                        Baz
                        \caption{Baz}
                    \end{figure}
                    %
                    \begin{figure}
                        Qux
                    \end{figure}
                    %
                    \end{document}                             
                "#
            ),
        )
        .file(
            "main.aux",
            indoc!(
                r#"
                    \relax 
                    \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Foo}}{1}\protected@file@percent }
                    \newlabel{fig:foo}{{1}{1}}
                    \@writefile{lof}{\contentsline {figure}{\numberline {2}{\ignorespaces Bar}}{1}\protected@file@percent }
                    \@writefile{lof}{\contentsline {figure}{\numberline {3}{\ignorespaces Baz}}{1}\protected@file@percent }                              
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed
        .initialize(NESTED_SYMBOL_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;
    test_bed.open("main.aux").await;

    let actual_symbols = test_bed.document_symbol_nested("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_symbols.len(), 3);
    verify_symbol(
        &actual_symbols[0],
        "Figure 1: Foo",
        Some("fig:foo"),
        Range::new_simple(6, 17, 6, 32),
        Range::new_simple(4, 0, 7, 12),
    );
    verify_symbol(
        &actual_symbols[1],
        "Figure: Bar",
        Some("fig:bar"),
        Range::new_simple(11, 17, 11, 32),
        Range::new_simple(9, 0, 12, 12),
    );
    verify_symbol(
        &actual_symbols[2],
        "Figure: Baz",
        None,
        Range::new_simple(14, 0, 17, 12),
        Range::new_simple(14, 0, 17, 12),
    );
}

#[tokio::test]
async fn section() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    %
                    \begin{document}
                    %
                    \section{Foo}
                    %
                    \section{Bar}\label{sec:bar}
                    %
                    \subsection{Baz}\label{sec:baz}
                    %
                    \end{document}                                           
                "#
            ),
        )
        .file(
            "main.aux",
            indoc!(
                r#"
                    \relax 
                    \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
                    \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}\protected@file@percent }
                    \newlabel{sec:bar}{{2}{1}}                                 
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed
        .initialize(NESTED_SYMBOL_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;
    test_bed.open("main.aux").await;

    let mut actual_symbols = test_bed.document_symbol_nested("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_symbols.len(), 2);
    verify_symbol(
        &actual_symbols[0],
        "Foo",
        None,
        Range::new_simple(4, 0, 4, 13),
        Range::new_simple(4, 0, 6, 0),
    );
    verify_symbol(
        &actual_symbols[1],
        "2 Bar",
        Some("sec:bar"),
        Range::new_simple(6, 0, 6, 13),
        Range::new_simple(6, 0, 10, 0),
    );

    let children = actual_symbols[1].children.take().unwrap();
    assert_eq!(children.len(), 1);
    verify_symbol(
        &children[0],
        "Baz",
        Some("sec:baz"),
        Range::new_simple(8, 0, 8, 16),
        Range::new_simple(8, 0, 10, 0),
    );
}

#[tokio::test]
async fn theorem() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \documentclass{article}
                    \usepackage{amsthm}
                    \newtheorem{lemma}{Lemma}
                    %
                    \begin{document}
                    %
                    \begin{lemma}[Foo]\label{thm:foo}
                        Foo
                    \end{lemma}
                    %
                    \begin{lemma}\label{thm:bar}
                        Bar
                    \end{lemma}
                    %
                    \begin{lemma}\label{thm:baz}
                        Baz
                    \end{lemma}
                    %
                    \begin{lemma}[Qux]
                        Qux
                    \end{lemma}
                    %
                    \end{document}                                      
                "#
            ),
        )
        .file(
            "main.aux",
            indoc!(
                r#"
                    \relax
                    \newlabel{thm:foo}{{1}{1}}
                    \newlabel{thm:bar}{{2}{1}}                                                
                "#
            ),
        )
        .build()
        .await;

    test_bed.spawn();
    test_bed
        .initialize(NESTED_SYMBOL_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;
    test_bed.open("main.aux").await;

    let actual_symbols = test_bed.document_symbol_nested("main.tex").await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_symbols.len(), 4);
    verify_symbol(
        &actual_symbols[0],
        "Lemma 1 (Foo)",
        Some("thm:foo"),
        Range::new_simple(6, 18, 6, 33),
        Range::new_simple(6, 0, 8, 11),
    );
    verify_symbol(
        &actual_symbols[1],
        "Lemma 2",
        Some("thm:bar"),
        Range::new_simple(10, 13, 10, 28),
        Range::new_simple(10, 0, 12, 11),
    );
    verify_symbol(
        &actual_symbols[2],
        "Lemma",
        Some("thm:baz"),
        Range::new_simple(14, 13, 14, 28),
        Range::new_simple(14, 0, 16, 11),
    );
    verify_symbol(
        &actual_symbols[3],
        "Lemma (Qux)",
        None,
        Range::new_simple(18, 0, 20, 11),
        Range::new_simple(18, 0, 20, 11),
    );
}
