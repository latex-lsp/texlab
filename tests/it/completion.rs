use anyhow::Result;
use lsp_types::{CompletionList, Url};

use crate::common::ServerTester;

fn complete_and_resolve(
    server: &ServerTester,
    uri: Url,
    line: u32,
    character: u32,
) -> Result<CompletionList> {
    let mut list = server.complete(uri, line, character)?;
    let mut new_items = Vec::new();
    for item in list.items.into_iter().take(7) {
        let mut new_item = server.resolve_completion_item(item)?;
        new_item.data = None;
        new_items.push(new_item);
    }
    list.items = new_items;
    Ok(list)
}

mod bibtex {
    use insta::assert_json_snapshot;
    use lsp_types::ClientCapabilities;

    use super::*;

    #[test]
    fn test_empty_document() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open("main.bib", "", "bibtex", false)?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 0)?);
        Ok(())
    }

    #[test]
    fn test_junk() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open("main.bib", "foo", "bibtex", false)?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 0)?);
        Ok(())
    }

    #[test]
    fn test_command_incomplete_entry() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @article{foo,
                    author = {\LaT
                }
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 18)?);
        Ok(())
    }

    #[test]
    fn test_command_complete_entry() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @article{foo,
                    author = {\LaT}
                }
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 18)?);
        Ok(())
    }

    #[test]
    fn test_entry_type_empty_name() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 1)?);
        Ok(())
    }

    #[test]
    fn test_entry_type_empty_name_before() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 0)?);
        Ok(())
    }

    #[test]
    fn test_entry_type_incomplete() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @art
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 1)?);
        Ok(())
    }

    #[test]
    fn test_entry_type_complete() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @article
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 1)?);
        Ok(())
    }

    #[test]
    fn test_field_incomplete_entry() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @article{foo,
                    titl
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 6)?);
        Ok(())
    }

    #[test]
    fn test_field_complete_entry() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.bib",
            r#"
                @article{foo,
                    title = {}
                }
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 6)?);
        Ok(())
    }
}

mod latex {
    use insta::assert_json_snapshot;
    use lsp_types::ClientCapabilities;

    use super::*;

    #[test]
    fn test_empty_document() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open("main.tex", "", "latex", false)?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 0)?);
        Ok(())
    }

    #[test]
    fn test_begin_command() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open("main.tex", r#"\b"#, "latex", false)?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 2)?);
        Ok(())
    }

    #[test]
    fn test_citation() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let tex_uri = server.open(
            "main.tex",
            r#"
                \documentclass{article}
                \bibliography{main}
                \begin{document}
                \cite{
                \end{document}
            "#,
            "latex",
            false,
        )?;
        server.open(
            "main.bib",
            r#"
                @article{foo:2019,
                    author = {Foo Bar},
                    title = {Baz Qux},
                    year = {2019},
                }

                @article{bar:2005,}
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, tex_uri, 3, 6)?);
        Ok(())
    }

    #[test]
    fn test_citation_multi_word() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let tex_uri = server.open(
            "main.tex",
            r#"
                \documentclass{article}
                \bibliography{main}
                \begin{document}
                \cite{foo 2
                \end{document}
            "#,
            "latex",
            false,
        )?;
        server.open(
            "main.bib",
            r#"
                @article{foo 2019,
                    author = {Foo Bar},
                    title = {Baz Qux},
                    year = {2019},
                }

                @article{bar:2005,}
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, tex_uri, 3, 7)?);
        Ok(())
    }

    #[test]
    fn test_citation_after() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let tex_uri = server.open(
            "main.tex",
            r#"
                \documentclass{article}
                \bibliography{main}
                \begin{document}
                \cite{}
                \end{document}
            "#,
            "latex",
            false,
        )?;
        server.open(
            "main.bib",
            r#"
                @article{foo:2019,
                    author = {Foo Bar},
                    title = {Baz Qux},
                    year = {2019},
                }

                @article{bar:2005,}
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, tex_uri, 3, 7)?);
        Ok(())
    }

    #[test]
    fn test_citation_open_brace() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let tex_uri = server.open(
            "main.tex",
            r#"
                \documentclass{article}
                \bibliography{main}
                \begin{document}
                \cite{Foo
                \end{document}
            "#,
            "latex",
            false,
        )?;
        server.open(
            "main.bib",
            r#"
                @article{FooBar,
                    author = {Foo Bar},
                    title = {Baz Qux},
                    year = {2019},
                }
            "#,
            "bibtex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, tex_uri, 3, 9)?);
        Ok(())
    }

    #[test]
    fn test_color_name() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \color{re}
                \definecolor{foo}{
                \definecolorset{R}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 9)?);
        Ok(())
    }

    #[test]
    fn test_color_model_define_color() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \color{re}
                \definecolor{foo}{
                \definecolorset{R}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 18)?);
        Ok(())
    }

    #[test]
    fn test_color_model_define_color_set() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \color{re}
                \definecolor{foo}{
                \definecolorset{R}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 2, 17)?);
        Ok(())
    }

    #[test]
    fn test_kernel_command() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 2)?);
        Ok(())
    }

    #[test]
    fn test_kernel_command_glyph() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 7, 7)?);
        Ok(())
    }

    #[test]
    fn test_kernel_command_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 6, 10)?);
        Ok(())
    }

    #[test]
    fn test_class_command() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 2, 5)?);
        Ok(())
    }

    #[test]
    fn test_class_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 4, 13)?);
        Ok(())
    }

    #[test]
    fn test_package_command() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 3, 7)?);
        Ok(())
    }

    #[test]
    fn test_package_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
                \chap
                \varDel
                \begin{theind}
                \end{alig}
                \begin{doc}
                \vareps
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 5, 6)?);
        Ok(())
    }

    #[test]
    fn test_class_import() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 19)?);
        Ok(())
    }

    #[test]
    fn test_package_import() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{book}
                \usepackage{amsmath}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 15)?);
        Ok(())
    }

    #[test]
    fn test_label() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;

        server.open(
            "foo.tex",
            r#"
                \documentclass{article}

                \usepackage{amsmath}
                \usepackage{caption}
                \usepackage{amsthm}
                \newtheorem{lemma}{Lemma}

                \begin{document}

                \section{Foo}%
                \label{sec:foo}

                \begin{equation}%
                \label{eq:foo}
                    1 + 1 = 2
                \end{equation}

                \begin{equation}%
                \label{eq:bar}
                    1 + 1 = 2
                \end{equation}

                \begin{figure}%
                \LaTeX{}
                \caption{Baz}%
                \label{fig:baz}
                \end{figure}

                \begin{lemma}%
                \label{thm:foo}
                    1 + 1 = 2
                \end{lemma}

                \include{bar}

                \end{document}
            "#,
            "latex",
            true,
        )?;
        server.open(
            "foo.aux",
            r#"
                \relax
                \@writefile{lof}{\contentsline {figure}{\numberline {1}{\ignorespaces Baz\relax }}{1}\protected@file@percent }
                \providecommand*\caption@xref[2]{\@setref\relax\@undefined{#1}}
                \newlabel{fig:baz}{{1}{1}}
                \@writefile{toc}{\contentsline {section}{\numberline {1}Foo}{1}\protected@file@percent }
                \newlabel{sec:foo}{{1}{1}}
                \newlabel{eq:foo}{{1}{1}}
                \newlabel{eq:bar}{{2}{1}}
                \newlabel{thm:foo}{{1}{1}}
                \@input{bar.aux}
            "#,
            "latex",
            true,
        )?;
        let uri = server.open(
            "bar.tex",
            r#"
                \section{Bar}%
                \label{sec:bar}

                Lorem ipsum dolor sit amet.
                \ref{}
                \eqref{}
            "#,
            "latex",
            true,
        )?;
        server.open(
            "bar.aux",
            r#"
                \relax
                \@writefile{toc}{\contentsline {section}{\numberline {2}Bar}{2}\protected@file@percent }
                \newlabel{sec:bar}{{2}{2}}
                \@setckpt{bar}{
                \setcounter{page}{3}
                \setcounter{equation}{2}
                \setcounter{enumi}{0}
                \setcounter{enumii}{0}
                \setcounter{enumiii}{0}
                \setcounter{enumiv}{0}
                \setcounter{footnote}{0}
                \setcounter{mpfootnote}{0}
                \setcounter{part}{0}
                \setcounter{section}{2}
                \setcounter{subsection}{0}
                \setcounter{subsubsection}{0}
                \setcounter{paragraph}{0}
                \setcounter{subparagraph}{0}
                \setcounter{figure}{1}
                \setcounter{table}{0}
                \setcounter{parentequation}{0}
                \setcounter{caption@flags}{0}
                \setcounter{ContinuedFloat}{0}
                \setcounter{lemma}{1}
            "#,
            "latex",
            true,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 4, 5)?);
        Ok(())
    }

    #[test]
    fn test_preselect_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \begin{document}
                \end{
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 5)?);
        Ok(())
    }

    #[test]
    fn test_theorem_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \documentclass{article}
                \usepackage{amsthm}
                \newtheorem{foo}{Foo}
                \begin{f}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 3, 8)?);
        Ok(())
    }

    #[test]
    fn test_pgf_library() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \usepackage{tikz}
                \usepgflibrary{}
                \usetikzlibrary{}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 15)?);
        Ok(())
    }

    #[test]
    fn test_user_command() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \foobar
                \fooba
                \begin{foo}
                \end{foo}
                \begin{fo}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 1, 3)?);
        Ok(())
    }

    #[test]
    fn test_user_environment() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \foobar
                \fooba
                \begin{foo}
                \end{foo}
                \begin{fo}
            "#,
            "latex",
            false,
        )?;
        assert_json_snapshot!(complete_and_resolve(&server, uri, 4, 8)?);
        Ok(())
    }

    #[test]
    fn test_multi_line_key() -> Result<()> {
        let server = ServerTester::launch_new_instance()?;
        server.initialize(ClientCapabilities::default(), None)?;
        let uri = server.open(
            "main.tex",
            r#"
                \begin{verb
                Velit tri-tip fig1n shoulder buffalo pariatur porkchop magna chuck sausage,
                sed hamburger fatback ribeye biltong id lorem culpa cow, frankfurter
                deserunt shortloin pancetta dolor et veniam aliqua andouille, pork fugiat eu
                pig landjaeger proident aliquip voluptate.
            "#,
            "latex",
            false,
        )?;

        assert_json_snapshot!(complete_and_resolve(&server, uri, 0, 11)?);
        Ok(())
    }
}
