use expect_test::{expect, Expect};

use crate::HoverParams;

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let (feature, offset) = fixture.make_params().unwrap();
    let params = HoverParams { feature, offset };
    let data = crate::find(&params).map(|hover| {
        assert_eq!(fixture.documents[0].ranges[0], hover.range);
        hover.data
    });

    expect.assert_debug_eq(&data);
}

#[test]
fn test_smoke() {
    check(
        r#"
%! main.tex

|"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_citation() {
    check(
        r#"
%! main.tex
\addbibresource{main.bib}
\cite{foo}
       |
      ^^^
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}"#,
        expect![[r#"
            Some(
                Citation(
                    "F. Bar: \"Baz Qux\". (1337).",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_key() {
    check(
        r#"
%! main.bib
@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}
          |
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}"#,
        expect![[r#"
            Some(
                Citation(
                    "F. Bar: \"Baz Qux\". (1337).",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_key_empty() {
    check(
        r#"
%! main.bib
@foo{bar,}
      |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_entry_type_known() {
    check(
        r#"
%! main.bib
@article{foo,}
    |
^^^^^^^^"#,
        expect![[r#"
            Some(
                EntryType(
                    BibtexEntryType {
                        name: "@article",
                        category: Article,
                        documentation: Some(
                            "An article in a journal, magazine, newspaper, or other periodical which forms a \n self-contained unit with its own title. The title of the periodical is given in the \n journaltitle field. If the issue has its own title in addition to the main title of \n the periodical, it goes in the issuetitle field. Note that editor and related \n fields refer to the journal while translator and related fields refer to the article.\n\nRequired fields: `author`, `title`, `journaltitle`, `year/date`",
                        ),
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_entry_type_unknown() {
    check(
        r#"
%! main.bib
@foo{bar,}
  |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_field_known() {
    check(
        r#"
%! main.bib
@article{foo, author = bar}
               |
              ^^^^^^"#,
        expect![[r#"
            Some(
                FieldType(
                    BibtexFieldType {
                        name: "author",
                        documentation: "The author(s) of the `title`.",
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_field_unknown() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
               |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_bibtex_string_ref() {
    check(
        r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                        |
                       ^^^"#,
        expect![[r#"
            Some(
                StringRef(
                    "Bar",
                ),
            )
        "#]],
    );
}

#[test]
fn test_bibtex_value() {
    check(
        r#"
%! main.bib
@string{foo = "Foo"}
@string{bar = "Bar"}
@article{baz, author = bar}
                     |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_package_known() {
    check(
        r#"
%! main.tex
\usepackage{amsmath}
             |
            ^^^^^^^"#,
        expect![[r#"
            Some(
                Package(
                    "The package provides the principal packages in the AMS-LaTeX distribution. It adapts for use in LaTeX most of the mathematical features found in AMS-TeX; it is highly recommended as an adjunct to serious mathematical typesetting in LaTeX. When amsmath is loaded, AMS-LaTeX packages amsbsy (for bold symbols), amsopn (for operator names) and amstext (for text embedded in mathematics) are also loaded. amsmath is part of the LaTeX required distribution; however, several contributed packages add still further to its appeal; examples are empheq, which provides functions for decorating and highlighting mathematics, and ntheorem, for specifying theorem (and similar) definitions.",
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_class_unknown() {
    check(
        r#"
%! main.tex
\documentclass{abcdefghijklmnop}
                    |"#,
        expect![[r#"
            None
        "#]],
    );
}

#[test]
fn test_latex_label_section() {
    check(
        r#"
%! main.tex
\section{Foo}
\label{sec:foo}
         |
       ^^^^^^^"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..29,
                        number: None,
                        object: Section {
                            prefix: "Section",
                            text: "Foo",
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_theorem_child_file() {
    check(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}\label{thm:foo}
    1 + 1 = 2
\end{lemma}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..54,
                        number: None,
                        object: Theorem {
                            kind: "Lemma",
                            description: None,
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_theorem_child_file_mumber() {
    check(
        r#"
%! main.tex
\documentclass{article}
\newtheorem{lemma}{Lemma}
\include{child}
\ref{thm:foo}
        |
     ^^^^^^^

%! child.tex
\begin{lemma}[Foo]\label{thm:foo}
    1 + 1 = 2
\end{lemma}

%! child.aux
\newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 0..59,
                        number: Some(
                            "1",
                        ),
                        object: Theorem {
                            kind: "Lemma",
                            description: Some(
                                "Foo",
                            ),
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_latex_label_ntheorem() {
    check(
        r#"
%! main.tex
\newtheorem{theorem}[theoremcounter]{Theorem}
\begin{theorem}%
\label{thm:test}
\end{theorem}
\ref{thm:test}
        |
     ^^^^^^^^

%! main.aux
\newlabel{thm:test}{{1.{1}}{1}}"#,
        expect![[r#"
            Some(
                Label(
                    RenderedLabel {
                        range: 46..93,
                        number: Some(
                            "1.1",
                        ),
                        object: Theorem {
                            kind: "Theorem",
                            description: None,
                        },
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_symbol_command_kernel() {
    check(
        r#"
%! main.tex
\pi
  |
^^^"#,
        expect![[r#"
            Some(
                Command(
                    Command {
                        name: "pi",
                        image: Some(
                            "iVBORw0KGgoAAAANSUhEUgAAADoAAAAxCAIAAAAEHi28AAAGT0lEQVR4nO2ZzU8TWxjG39MpQ1tsaSrCNHwopEWMGqhgJXVBUhIkxrhwK1sTEoPuNbgQE/8BE01cmPjBQtNoUCPVQAWiMWkC0bQhhUChov2yjNIWmekw5y7GO/b20unMcLnGxGd1zvR5z/m18573zJwijDH8PtL8agBl+oO7k/rNcLUyfTzPRyKR58+fP336NBgMplKpXC63Q0wIofLy8ubm5suXL585c+Yfn2EZomn69u3bDoeDJMkdQtxS7e3tBSSlccPhcF9fn8Fg+D9BBTU2NirDnZ2d7e7u1mh+QYprtdr+/v4CHqncXVpaunDhwvj4OP57K0EIkSSp0+m02n8E5nK5dDqNEDKZTARBbDkaxnhtbY3juIqKCp1OV2xShJBWq62pqTl16tT58+cLv0OxMJqmBwcHRVaj0djW1tbV1dXa2kpRlF6vRwiJZq/Xe+XKFYqibt26ZbVatxwwGo329/fHYrFLly719vYWm1ej0ZAkabFYqqqqCn6UH1/632IY5urVq+Xl5QBAkmRPT8+TJ09SqRTP8/82cxx38eJFADhx4kQmkymWV1NTU5WVlWaz+c2bN9IZKKEtcHmeHxkZqaqqAgCLxTI0NJRMJiWGSCaTTqcTITQ4OLi5uVnMdvPmTYIg7Hb78vLyf4m7uLh49OhRAKAo6s6dOwzDSA/x9u1bi8ViMBiePXtWzJPL5c6dOwcAbrc7nU6rxi1c8gzD3LhxY3p62mw2X7t27ezZs9K1FmM8NTX19etXiqIOHDhQzJZOpwOBAAA0NDTo9XqJAaVViDs2Nnb//n2CIAYGBvr6+srKyqTjM5nMq1eveJ5vaWkptsgAIBqNLi8vA0BTU9N2ymJh5PXr15PJZHd398DAgLDUpDU3N/fhwweEUGdnp0R5mpubS6VSWq123759+SVlu7gmk8ntdg8NDe3Zs6dkMMb49evXX7580ev1wmorZpuZmWFZtqysTOIOyFFhYRseHsYYV1ZWygnOZDKjo6M8z1ut1v379xezMQzz/v17jLFOpzObzf8lrkxQQcFgcGZmBgAOHjxIUVQxG03T8/PzAGAwGEwmkyrOH1Kf9TzPv3jxYnV1VaPRHDt2TCJxV1ZWYrEYABgMhoqKCtUzwnZwU6mU1+vFGBsMBqfTKeGcn59fW1sDAL1eX7LUSEs9rt/vDwaDAFBbW9vc3FzMhjEOBoMcxwEASZJbPAYokUpclmU9Hk8mkwGAQ4cOVVdXF3PmcrnZ2VmhTRDENp9FVQaHw2GfzwcAGo2ms7NTokKn02lhgxC0naIL6nAxxl6v9+PHjwCwa9eujo4OCYjV1dVEIiG0eZ7neV4dqCA1uDRNezweIR3r6ursdruEOR6PC+sMADiO+wW47969m56eFtqHDx+W3v8SicTGxobQ3tjYYFlWxYyiFOMyDPPw4UNhkQmJK/3IFo/HhfsAAOvr69+/f1cHKkgxbigUGhsbE9pGo7Gjo0PaT9O0mADZbPbbt29KZ8yXMtzNzU2Px/P582eh29DQYLPZJPwYYzFxASCbzcbjcRWUopThfvr06fHjx+Kv1dbWtnv3bgk/xphhGLHLMMzS0pJyyJ9SgCvUr1AoJHQJgnC5XCU3VTFxAYDn+UAgsJ3ioACXpukHDx6IS9tsNre3tyudLxAIZLNZpVGiFOBOTEz4/X6xa7fbm5qaSkYVbHgLCwsrKyvyJy2QXNxsNnv37t319XWhixByuVwln7WFc538K4lEQqzZKiQX1+/3T05Oil29Xt/V1VXsfEkUQqimpibfxrLs+Pi46sNWWbgMw9y7d4+mafFKfX19a2urnNi6urqCfWRycjISiSiiFCULNxAIjI6O4rz/iJxOp8yXxL179xa8UEUikfwbpUilcTmOGx4eFt5eBJEk2dvbK/No2mq11tbW5l9hWXZkZETdblwaNxwO528NANDY2OhyuWROYDabHQ5HwcXFxcV0Oi2fUlRp3FgslkwmfwZoNKdPn66vr5c5AUEQPT09BS+eNpvNaDQqAv2hkqdoCwsLLS0tghkh5HA4QqGQonO4aDR6/PhxcQSbzebz+RSNIKo0Lsdxjx49OnLkSHV1tdvtnpiY2PKUV0I8z798+dLhcFAUdfLkSZ/Px3GcOlyE//wnvHP6g7uT+s1w/wIcV6JE3mbgEAAAAABJRU5ErkJggg==",
                        ),
                        glyph: Some(
                            "π",
                        ),
                        parameters: [],
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_symbol_command_package() {
    check(
        r#"
%! main.tex
\usepackage{amsmath}
\varDelta
   |
^^^^^^^^^"#,
        expect![[r#"
            Some(
                Command(
                    Command {
                        name: "varDelta",
                        image: Some(
                            "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",
                        ),
                        glyph: None,
                        parameters: [],
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn test_symbol_command_package_not_included() {
    check(
        r#"
%! main.tex
\varDelta
   |"#,
        expect![[r#"
            None
        "#]],
    );
}
