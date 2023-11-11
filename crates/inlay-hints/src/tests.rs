use expect_test::{expect, Expect};
use rowan::{TextLen, TextRange};

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);

    let (feature, _) = fixture.make_params().unwrap();
    let range = TextRange::new(0.into(), feature.document.text.text_len());
    let params = crate::InlayHintParams { range, feature };
    let actual = crate::find_all(params).unwrap_or_default();

    let expected_offsets = fixture.locations().map(|location| location.range.start());
    for (hint, offset) in actual.iter().zip(expected_offsets) {
        assert_eq!(hint.offset, offset);
    }

    let data = actual.into_iter().map(|hint| hint.data).collect::<Vec<_>>();
    expect.assert_debug_eq(&data);
}

#[test]
fn test_label_definition() {
    check(
        r#"
%! main.tex
\documentclass{article}
\usepackage{caption}
\begin{document}
\section{Foo}\label{sec:foo}
                            !
\section{Bar}\label{sec:bar}
                            !
\subsection{Baz}\label{sec:baz}
                               !
\begin{figure}
    Test
    \label{fig:qux}
                   !
    \caption{Qux}
\end{figure}
\end{document}
    
%! main.aux
\relax 
\providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
\newlabel{fig:qux}{{\caption@xref {fig:qux}{ on input line 15}}{1}}
\@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Qux\relax }}{1}{}\protected@file@percent }
\@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}{}\protected@file@percent }
\newlabel{sec:foo}{{1}{1}}
\@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{1}{}\protected@file@percent }
\newlabel{sec:bar}{{2}{1}}
\@writefile{toc}{\contentsline {subsection}{\numberline {2.1}Baz}{1}{}\protected@file@percent }
\newlabel{sec:baz}{{2.1}{1}}
\gdef \@abspage@last{1}"#,
        expect![[r#"
            [
                LabelDefinition(
                    RenderedLabel {
                        range: 62..90,
                        number: Some(
                            "1",
                        ),
                        object: Section {
                            prefix: "Section",
                            text: "Foo",
                        },
                    },
                ),
                LabelDefinition(
                    RenderedLabel {
                        range: 91..226,
                        number: Some(
                            "2",
                        ),
                        object: Section {
                            prefix: "Section",
                            text: "Bar",
                        },
                    },
                ),
                LabelDefinition(
                    RenderedLabel {
                        range: 120..226,
                        number: Some(
                            "2.1",
                        ),
                        object: Section {
                            prefix: "Subsection",
                            text: "Baz",
                        },
                    },
                ),
                LabelDefinition(
                    RenderedLabel {
                        range: 152..226,
                        number: Some(
                            "fig:qux",
                        ),
                        object: Float {
                            kind: Figure,
                            caption: "Qux",
                        },
                    },
                ),
            ]
        "#]],
    );
}
