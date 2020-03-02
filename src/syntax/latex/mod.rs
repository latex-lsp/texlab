mod analysis;
mod ast;
mod lexer;
mod parser;

pub use self::{analysis::*, ast::*};

use self::{lexer::Lexer, parser::Parser};
use crate::{
    protocol::{Options, Uri},
    tex::Resolver,
};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpenParams<'a> {
    pub text: &'a str,
    pub uri: &'a Uri,
    pub resolver: &'a Resolver,
    pub options: &'a Options,
    pub cwd: &'a Path,
}

pub fn open(params: OpenParams) -> SymbolTable {
    let OpenParams {
        text,
        uri,
        resolver,
        options,
        cwd,
    } = params;

    let lexer = Lexer::new(text);
    let parser = Parser::new(lexer);
    let tree = parser.parse();

    let params = SymbolTableParams {
        tree,
        uri,
        resolver,
        options,
        cwd,
    };
    SymbolTable::analyze(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldenfile::Mint;
    use indoc::indoc;
    use std::{env, io::Write};

    fn verify(name: &str, text: &str) {
        let table = open(OpenParams {
            text,
            uri: &Uri::parse("file:///home/user/foo.tex").unwrap(),
            resolver: &Resolver::default(),
            options: &Options::default(),
            cwd: &env::current_dir().unwrap(),
        });

        let mut mint = Mint::new("tests/goldenfiles/latex");
        let mut file = mint.new_goldenfile(name).unwrap();
        let json = serde_json::to_string_pretty(&table).unwrap();
        write!(file, "{}", json).unwrap();
    }

    #[test]
    fn open_environment() {
        verify(
            "open_environment.json",
            indoc!(
                r#"
                \begin{document}
                \begin{a}
                    Foo
                    \begin{b}
                        Bar
                    \end{b}
                \end{}
                \end{document}
                "#
            ),
        )
    }

    #[test]
    fn open_include() {
        verify(
            "open_include.json",
            indoc!(
                r#"
                \documentclass{article}
                \usepackage{amsmath}
                \usepackage{lipsum, geometry}
                \include{foo}
                \input{bar.tex}
                \include
                "#
            ),
        )
    }

    #[test]
    fn open_citation() {
        verify(
            "open_citation.json",
            indoc!(
                r#"
                \cite{foo,bar,baz}
                \cite{foo,,}
                \nocite{*}
                \cite
                "#
            ),
        );
    }

    #[test]
    fn open_command_definition() {
        verify(
            "open_command_definition.json",
            indoc!(
                r#"
                \newcommand{\foo}{foo {bar}}
                \newcommand{\bar}
                \newcommand{}
                \newcommand
                "#
            ),
        )
    }

    #[test]
    fn open_glossary_entry() {
        verify(
            "open_glossary_entry.json",
            indoc!(
                r#"
                \newglossaryentry{foo}{...}
                \newglossaryentry{bar}
                \newglossaryentry{}
                \newglossaryentry
                "#
            ),
        )
    }

    #[test]
    fn open_equation() {
        verify(
            "open_equation.json",
            indoc!(
                r#"
                \[ foo bar baz \]
                \[ e^{i \pi} + 1 = 0 \]
                \] \[
                "#
            ),
        )
    }

    #[test]
    fn open_inline() {
        verify(
            "open_inline.json",
            indoc!(
                r#"
                $ e^{i \pi} + 1 = 0 $
                $$ f(x, y) = x^2 + 1 - y^2 $$
                $ x $$
                $
                "#
            ),
        )
    }

    #[test]
    fn open_math_operator() {
        verify(
            "open_math_operator.json",
            indoc!(
                r#"
                \DeclareMathOperator{\foo}{foo}
                \DeclareMathOperator{\foo}
                \DeclareMathOperator{}
                \DeclareMathOperator
                "#
            ),
        )
    }

    #[test]
    fn open_theorem_definition() {
        verify(
            "open_theorem_definition.json",
            indoc!(
                r#"
                \newtheorem{lemma}{Lemma}
                \newtheorem{foo}
                \newtheorem{}{}
                \newtheorem
                "#
            ),
        )
    }

    #[test]
    fn open_section() {
        verify(
            "open_section.json",
            indoc!(
                r#"
                \section{Foo}
                \subsection{Bar Baz}
                \paragraph*{Qux}
                "#
            ),
        )
    }

    #[test]
    fn open_label() {
        verify(
            "open_label.json",
            indoc!(
                r#"
                \label{foo}
                \label{bar}
                \ref{foo, bar}
                \eqref
                "#
            ),
        )
    }

    #[test]
    fn open_label_numbering() {
        verify(
            "open_label_numbering.json",
            indoc!(
                r#"
                \newlabel{foo}{{1}{1}}
                \newlabel{foo}
                \newlabel
                "#
            ),
        )
    }

    #[test]
    fn open_caption() {
        verify(
            "open_caption.json",
            indoc!(
                r#"
                \caption{Foo \LaTeX Bar}
                \caption{}
                \caption
                "#
            ),
        )
    }

    #[test]
    fn open_item() {
        verify(
            "open_item.json",
            indoc!(
                r#"
                \item[foo]{bar}
                \item
                "#
            ),
        )
    }
}
