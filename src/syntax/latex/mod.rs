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
    use crate::{
        protocol::{Range, RangeExt},
        syntax::text::SyntaxNode,
    };
    use indoc::indoc;
    use petgraph::graph::NodeIndex;
    use std::env;

    fn open_simple(text: &str) -> SymbolTable {
        open(OpenParams {
            text: text.trim(),
            uri: &Uri::parse("http://www.foo.com/bar.tex").unwrap(),
            resolver: &Resolver::default(),
            options: &Options::default(),
            cwd: &env::current_dir().unwrap(),
        })
    }

    #[derive(Debug, Default)]
    struct TreeTraversal {
        nodes: Vec<NodeIndex>,
    }

    impl Visitor for TreeTraversal {
        fn visit(&mut self, tree: &Tree, node: NodeIndex) {
            self.nodes.push(node);
            tree.walk(self, node);
        }
    }

    mod range {
        use super::*;

        fn verify(expected_ranges: Vec<Range>, text: &str) {
            let table = open_simple(text);

            let mut traversal = TreeTraversal::default();
            traversal.visit(&table.tree, table.tree.root);
            let actual_ranges: Vec<_> = traversal
                .nodes
                .into_iter()
                .map(|node| table.tree.graph[node].range())
                .collect();
            assert_eq!(actual_ranges, expected_ranges);
        }

        #[test]
        fn command() {
            verify(
                vec![
                    Range::new_simple(0, 0, 2, 14),
                    Range::new_simple(0, 0, 0, 23),
                    Range::new_simple(0, 14, 0, 23),
                    Range::new_simple(0, 15, 0, 22),
                    Range::new_simple(1, 0, 1, 20),
                    Range::new_simple(1, 11, 1, 20),
                    Range::new_simple(1, 12, 1, 19),
                    Range::new_simple(2, 0, 2, 14),
                    Range::new_simple(2, 4, 2, 9),
                    Range::new_simple(2, 5, 2, 8),
                    Range::new_simple(2, 9, 2, 14),
                    Range::new_simple(2, 10, 2, 13),
                ],
                indoc!(
                    r#"
                        \documentclass{article}
                        \usepackage{amsmath}
                        \foo[bar]{baz}
                    "#
                ),
            );
        }

        #[test]
        fn text() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 11),
                    Range::new_simple(0, 0, 0, 11),
                ],
                indoc!(
                    r#"
                        foo bar baz
                    "#
                ),
            );
        }

        #[test]
        fn text_bracket() {
            verify(
                vec![Range::new_simple(0, 0, 0, 5), Range::new_simple(0, 0, 0, 5)],
                indoc!(
                    r#"
                        ]foo[
                    "#
                ),
            );
        }

        #[test]
        fn group() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 2, 0, 5),
                    Range::new_simple(0, 6, 0, 13),
                    Range::new_simple(0, 8, 0, 11),
                ],
                indoc!(
                    r#"
                        { foo { bar } }
                    "#
                ),
            );
        }

        #[test]
        fn group_incomplete() {
            verify(
                vec![Range::new_simple(0, 1, 0, 2), Range::new_simple(0, 1, 0, 2)],
                indoc!(
                    r#"
                        }{
                    "#
                ),
            );
        }

        #[test]
        fn math() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 9),
                    Range::new_simple(0, 0, 0, 1),
                    Range::new_simple(0, 2, 0, 7),
                    Range::new_simple(0, 8, 0, 9),
                ],
                indoc!(
                    r#"
                        $ x = 1 $
                    "#
                ),
            );
        }

        #[test]
        fn comma() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 8),
                    Range::new_simple(0, 0, 0, 3),
                    Range::new_simple(0, 3, 0, 4),
                    Range::new_simple(0, 5, 0, 8),
                ],
                indoc!(
                    r#"
                        foo, bar
                    "#
                ),
            );
        }
    }

    mod command {
        use super::*;

        fn verify(expected_names: Vec<&str>, text: &str) {
            let table = open(OpenParams {
                text,
                uri: &Uri::parse("http://www.foo.com/bar.tex").unwrap(),
                resolver: &Resolver::default(),
                options: &Options::default(),
                cwd: &env::current_dir().unwrap(),
            });

            let actual_names: Vec<_> = table
                .commands
                .iter()
                .map(|node| table.tree.as_command(*node).unwrap().name.text())
                .collect();

            assert_eq!(actual_names, expected_names);
        }

        #[test]
        fn basic() {
            verify(
                vec!["\\documentclass", "\\usepackage", "\\begin", "\\end"],
                indoc!(
                    r#"
                        \documentclass{article}
                        \usepackage{amsmath}
                        \begin{document}
                        Hello World
                        \end{document}
                    "#
                ),
            );
        }

        #[test]
        fn star() {
            verify(
                vec!["\\section*", "\\subsection*"],
                indoc!(
                    r#"
                        \section*{Foo}
                        \subsection**{Bar}
                    "#
                ),
            );
        }

        #[test]
        fn at() {
            verify(vec!["\\foo@bar"], indoc!(r#"\foo@bar"#));
        }

        #[test]
        fn escape() {
            verify(vec!["\\%"], indoc!(r#"\%foo"#))
        }
    }

    mod environment {
        use super::*;

        fn verify(expected_names: Vec<(&str, &str)>, text: &str) {
            let table = open_simple(text);
            let actual_names: Vec<_> = table
                .environments
                .iter()
                .map(|env| {
                    (
                        env.left
                            .name(&table.tree)
                            .map(Token::text)
                            .unwrap_or_default(),
                        env.right
                            .name(&table.tree)
                            .map(Token::text)
                            .unwrap_or_default(),
                    )
                })
                .collect();

            assert_eq!(actual_names, expected_names);
        }

        #[test]
        fn nested() {
            verify(
                vec![("b", "b"), ("a", "a")],
                indoc!(
                    r#"
                        \begin{a}
                            \begin{b}
                            \end{b}
                        \end{a}
                    "#
                ),
            );
        }

        #[test]
        fn empty_name() {
            verify(
                vec![("a", ""), ("", "b")],
                indoc!(
                    r#"
                        \begin{a}
                        \end{}
                        \begin{}
                        \end{b}
                    "#
                ),
            );
        }

        #[test]
        fn incomplete() {
            verify(
                Vec::new(),
                indoc!(
                    r#"
                        \end{a}
                        \begin{a}
                    "#
                ),
            );
        }

        #[test]
        fn standalone_true() {
            let table = open_simple(r#"\begin{document}\end{document}"#);
            assert!(table.is_standalone);
        }

        #[test]
        fn standalone_false() {
            let table = open_simple(r#"\begin{doc}\end{doc}"#);
            assert!(!table.is_standalone);
        }
    }

    mod include {
        use super::*;

        fn verify(expected_targets: Vec<Vec<&str>>, resolver: Resolver, text: &str) {
            let table = open(OpenParams {
                text,
                uri: &Uri::parse("http://www.foo.com/dir1/dir2/foo.tex").unwrap(),
                resolver: &resolver,
                options: &Options::default(),
                cwd: &env::current_dir().unwrap(),
            });

            assert_eq!(table.includes.len(), 1);
            let include = &table.includes[0];
            let actual_targets: Vec<Vec<&str>> = include
                .all_targets
                .iter()
                .map(|targets| targets.iter().map(|target| target.as_str()).collect())
                .collect();

            assert_eq!(actual_targets, expected_targets);
        }

        #[test]
        fn same_directory() {
            verify(
                vec![vec![
                    "http://www.foo.com/dir1/dir2/bar",
                    "http://www.foo.com/dir1/dir2/bar.tex",
                ]],
                Resolver::default(),
                indoc!(r#"\include{bar}"#),
            );
        }

        #[test]
        fn two_paths() {
            verify(
                vec![
                    vec![
                        "http://www.foo.com/dir1/dir2/bar.tex",
                        "http://www.foo.com/dir1/dir2/bar.tex.tex",
                    ],
                    vec![
                        "http://www.foo.com/dir1/dir2/baz.tex",
                        "http://www.foo.com/dir1/dir2/baz.tex.tex",
                    ],
                ],
                Resolver::default(),
                indoc!(r#"\input{bar.tex, ./baz.tex}"#),
            );
        }

        #[test]
        fn sub_directory() {
            verify(
                vec![vec![
                    "http://www.foo.com/dir1/dir2/dir3/bar",
                    "http://www.foo.com/dir1/dir2/dir3/bar.tex",
                ]],
                Resolver::default(),
                indoc!(r#"\include{dir3/bar}"#),
            );
        }

        #[test]
        fn parent_directory() {
            verify(
                vec![vec![
                    "http://www.foo.com/dir1/bar",
                    "http://www.foo.com/dir1/bar.tex",
                ]],
                Resolver::default(),
                indoc!(r#"\include{../bar}"#),
            );
        }

        #[test]
        fn distro_file() {
            let mut resolver = Resolver::default();
            let path = env::current_dir().unwrap().join("biblatex-examples.bib");
            resolver
                .files_by_name
                .insert("biblatex-examples.bib".into(), path.clone());
            verify(
                vec![vec![
                    "http://www.foo.com/dir1/dir2/biblatex-examples.bib",
                    "http://www.foo.com/dir1/dir2/biblatex-examples.bib.bib",
                    Uri::from_file_path(&path).unwrap().as_str(),
                ]],
                resolver,
                indoc!(r#"\addbibresource{biblatex-examples.bib}"#),
            );
        }

        #[test]
        fn component() {
            let table = open(OpenParams {
                text: indoc!(
                    r#"
                        \documentclass{article}
                        \usepackage{amsmath}
                        \usepackage{geometry, lipsum}
                    "#
                ),
                uri: &Uri::parse("http://www.foo.com/bar.tex").unwrap(),
                resolver: &Resolver::default(),
                options: &Options::default(),
                cwd: &env::current_dir().unwrap(),
            });
            assert_eq!(
                table.components,
                vec!["article.cls", "amsmath.sty", "geometry.sty", "lipsum.sty"]
            );
        }
    }

    #[test]
    fn citation() {
        let table = open_simple(indoc!(
            r#"
                \cite{key1}
                \cite{key2, key3}
                \nocite{*}
            "#
        ));

        let expected_keys = vec![vec!["key1"], vec!["key2", "key3"], vec!["*"]];

        let actual_keys: Vec<Vec<&str>> = table
            .citations
            .iter()
            .map(|cit| cit.keys(&table.tree).into_iter().map(Token::text).collect())
            .collect();

        assert_eq!(actual_keys, expected_keys);
    }

    #[test]
    fn command_definition() {
        let table = open_simple(indoc!(
            r#"
                \newcommand{\foo}{Foo}
                \newcommand[2]{\bar}{Bar}
                \renewcommand{\baz}{Baz}
                \qux
            "#
        ));

        let expected_cmds = vec!["\\foo", "\\bar", "\\baz"];

        let actual_cmds: Vec<&str> = table
            .command_definitions
            .iter()
            .map(|def| def.definition_name(&table.tree))
            .collect();

        assert_eq!(actual_cmds, expected_cmds);
    }

    #[test]
    fn glossary_entry() {
        let table = open_simple(indoc!(
            r#"
                \newglossaryentry{foo}{...}
                \newacronym{bar}{...}
            "#
        ));

        let expected_entries = vec!["foo", "bar"];

        let actual_entries: Vec<&str> = table
            .glossary_entries
            .iter()
            .map(|entry| entry.label(&table.tree).text())
            .collect();

        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    fn equation() {
        let table = open_simple(indoc!(
            r#"
                \[
                    e^{i \pi} + 1 = 0
                \]
                \] \[
            "#
        ));

        assert_eq!(table.equations.len(), 1);
    }

    #[test]
    fn inline() {
        let table = open_simple(indoc!(
            r#"
                $ x $
                $
            "#
        ));

        assert_eq!(table.inlines.len(), 1);
    }

    #[test]
    fn math_operator() {
        let table = open_simple(indoc!(
            r#"
                \DeclareMathOperator{\foo}{foo}
            "#
        ));

        assert_eq!(table.math_operators.len(), 1);
        assert_eq!(
            table.math_operators[0].definition_name(&table.tree),
            "\\foo"
        );
    }

    #[test]
    fn theorem_definition() {
        let table = open_simple(indoc!(
            r#"
                \newtheorem{lemma}{Lemma}
            "#
        ));

        assert_eq!(table.theorem_definitions.len(), 1);
        assert_eq!(
            table.theorem_definitions[0].name(&table.tree).text(),
            "lemma"
        );
    }

    #[test]
    fn section() {
        let table = open_simple(indoc!(
            r#"
                \section{Introduction to \LaTeX}
                \subsection*{Foo
            "#
        ));
        assert_eq!(table.sections.len(), 2);
        assert_eq!(
            table.sections[0].print(&table.tree).unwrap(),
            "Introduction to \\LaTeX"
        );
        assert_eq!(table.sections[1].print(&table.tree), None);
    }

    #[test]
    fn label() {
        let table = open_simple(indoc!(
            r#"
                \label{foo}
                \ref{bar, baz}
            "#
        ));

        let expected_names = vec![vec!["foo"], vec!["bar", "baz"]];

        let actual_names: Vec<Vec<&str>> = table
            .labels
            .iter()
            .map(|label| {
                label
                    .names(&table.tree)
                    .into_iter()
                    .map(Token::text)
                    .collect()
            })
            .collect();

        assert_eq!(actual_names, expected_names);
    }

    #[test]
    fn label_numbering() {
        let table = open_simple(indoc!(
            r#"
                \newlabel{foo}{{1}{1}}
            "#
        ));

        assert_eq!(table.label_numberings.len(), 1);
        assert_eq!(table.label_numberings[0].name(&table.tree).text(), "foo");
        assert_eq!(table.label_numberings[0].number, "1");
    }

    #[test]
    fn caption() {
        let table = open_simple(indoc!(
            r#"
                \caption{Foo \LaTeX Bar}
            "#
        ));

        assert_eq!(table.captions.len(), 1);
        assert_eq!(
            table.captions[0].print(&table.tree).unwrap(),
            "Foo \\LaTeX Bar"
        );
    }

    #[test]
    fn item_without_name() {
        let table = open_simple(indoc!(
            r#"
                \item
            "#
        ));

        assert_eq!(table.items.len(), 1);
        assert_eq!(table.items[0].name(&table.tree), None);
    }

    #[test]
    fn item_with_name() {
        let table = open_simple(indoc!(
            r#"
                \item[foo bar]
            "#
        ));

        assert_eq!(table.items.len(), 1);
        assert_eq!(table.items[0].name(&table.tree).unwrap(), "foo bar");
    }
}
