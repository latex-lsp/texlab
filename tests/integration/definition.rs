use indoc::indoc;
use texlab::test::{TestBedBuilder, LOCATION_LINK_CAPABILITIES, PULL_CAPABILITIES};
use texlab_protocol::{LocationLink, Range, RangeExt};

fn verify_origin_selection_range(
    link: &LocationLink,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
) {
    assert_eq!(
        link.origin_selection_range,
        Some(Range::new_simple(
            start_line,
            start_character,
            end_line,
            end_character
        ))
    );
}

#[tokio::test]
async fn empty_latex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.tex", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.tex").await;

    let actual_locations = test_bed
        .definition_location("main.tex", 0, 0)
        .await
        .unwrap();

    test_bed.shutdown().await;

    assert!(actual_locations.is_empty());
}

#[tokio::test]
async fn empty_bibtex_document() {
    let mut test_bed = TestBedBuilder::new().file("main.bib", "").build().await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("main.bib").await;

    let actual_locations = test_bed
        .definition_location("main.bib", 0, 0)
        .await
        .unwrap();

    test_bed.shutdown().await;

    assert!(actual_locations.is_empty());
}

#[tokio::test]
async fn bibtex_string() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.bib",
            indoc!(
                r#"
                    @string{foo = "Foo"}
                    %
                    @string{bar = "Bar"}
                    %
                    @article{baz, 
                        author = bar
                    }
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.bib").await;

    let mut actual_links = test_bed.definition_link("main.bib", 5, 14).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 5, 13, 5, 16);
    assert_eq!(link.target_uri, test_bed.uri("main.bib").into());
    assert_eq!(link.target_range, Range::new_simple(2, 0, 2, 20));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 8, 2, 11));
}

#[tokio::test]
async fn latex_citation_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \bibliography{bar}
                    \cite{bar}                
                "#
            ),
        )
        .file(
            "bar.bib",
            indoc!(
                r#"
                    @article{foo,}
                    %
                    @article{bar,}                
                "#
            ),
        )
        .file(
            "baz.bib",
            indoc!(
                r#"
                    @article{foo,}
                    %
                    @article{bar,}                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("foo.tex").await;
    test_bed.open("bar.bib").await;
    test_bed.open("baz.bib").await;

    let mut actual_links = test_bed.definition_link("foo.tex", 1, 7).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 1, 6, 1, 9);
    assert_eq!(link.target_uri, test_bed.uri("bar.bib").into());
    assert_eq!(link.target_range, Range::new_simple(2, 0, 2, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 12));
}

#[tokio::test]
async fn latex_citation_location() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "foo.tex",
            indoc!(
                r#"
                    \bibliography{bar}
                    \cite{bar}                
                "#
            ),
        )
        .file(
            "bar.bib",
            indoc!(
                r#"
                    @article{foo,}
                    %
                    @article{bar,}                
                "#
            ),
        )
        .file(
            "baz.bib",
            indoc!(
                r#"
                    @article{foo,}
                    %
                    @article{bar,}                
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed.initialize(PULL_CAPABILITIES.clone()).await;
    test_bed.open("foo.tex").await;
    test_bed.open("bar.bib").await;
    test_bed.open("baz.bib").await;

    let mut actual_locations = test_bed.definition_location("foo.tex", 1, 7).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_locations.len(), 1);
    let location = actual_locations.pop().unwrap();
    assert_eq!(location.uri, test_bed.uri("bar.bib").into());
    assert_eq!(location.range, Range::new_simple(2, 9, 2, 12));
}

#[tokio::test]
async fn latex_command_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \newcommand{\foo}{Foo}
                    %
                    \foo            
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 2, 2).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 22));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 22));
}

#[tokio::test]
async fn latex_math_operator_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \DeclareMathOperator{\foo}{foo}
                    %
                    \foo
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 2, 2).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 31));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 31));
}

#[tokio::test]
async fn latex_label_unknown_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \label{foo}
                    \ref{foo}                         
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 1, 7).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 1, 5, 1, 8);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 11));
}

#[tokio::test]
async fn latex_label_equation_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \begin{equation}%
                    \label{eq:foo}
                        Foo
                    \end{equation}
                    %
                    \ref{eq:foo}                                     
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 5, 8).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 5, 5, 5, 11);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 14));
}

#[tokio::test]
async fn latex_label_float_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \begin{figure}
                    Foo
                    \caption{Bar}
                    \label{fig}
                    \end{figure}
                    %
                    \ref{fig}                                                   
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 6, 6).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 12));
    assert_eq!(link.target_selection_range, Range::new_simple(3, 0, 3, 11));
}

#[tokio::test]
async fn latex_label_item_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                \begin{enumerate}
                    \item Foo
                    \item\label{bar} Bar
                    \item Baz
                \end{enumerate}
                %
                \ref{bar}                                           
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 6, 6).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 15));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 20));
}

#[tokio::test]
async fn latex_label_section_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \section{Foo}
                    \label{sec:foo}
                    %
                    \section{Bar}
                    \label{sec:bar}
                    %
                    \ref{sec:foo}                                                      
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 6, 6).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 12);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 0));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 15));
}

#[tokio::test]
async fn latex_label_theorem_link() {
    let mut test_bed = TestBedBuilder::new()
        .file(
            "main.tex",
            indoc!(
                r#"
                    \usepackage{amsthm}
                    \newtheorem{lemma}{Lemma}
                    %
                    \begin{lemma}%
                    \label{thm:foo}
                        Foo
                    \end{lemma}
                    %
                    \ref{thm:foo}                                                                                       
                "#
            ),
        )
        .build()
        .await;
    test_bed.spawn();
    test_bed
        .initialize(LOCATION_LINK_CAPABILITIES.clone())
        .await;
    test_bed.open("main.tex").await;

    let mut actual_links = test_bed.definition_link("main.tex", 8, 7).await.unwrap();

    test_bed.shutdown().await;

    assert_eq!(actual_links.len(), 1);
    let link = actual_links.pop().unwrap();
    verify_origin_selection_range(&link, 8, 5, 8, 12);
    assert_eq!(link.target_uri, test_bed.uri("main.tex").into());
    assert_eq!(link.target_range, Range::new_simple(3, 0, 6, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(4, 0, 4, 15));
}
