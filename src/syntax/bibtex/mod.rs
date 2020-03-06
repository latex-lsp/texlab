mod ast;
mod formatter;
mod lexer;
mod parser;

pub use self::{ast::*, formatter::*};

use self::{lexer::Lexer, parser::Parser};

pub fn open(text: &str) -> Tree {
    let lexer = Lexer::new(text);
    let parser = Parser::new(lexer);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        protocol::{Range, RangeExt},
        syntax::text::SyntaxNode,
    };
    use petgraph::graph::NodeIndex;

    #[derive(Debug, Default)]
    struct TreeTraversal {
        nodes: Vec<NodeIndex>,
    }

    impl<'a> Visitor<'a> for TreeTraversal {
        fn visit(&mut self, tree: &Tree, node: NodeIndex) {
            self.nodes.push(node);
            tree.walk(self, node);
        }
    }

    mod range {
        use super::*;
        use indoc::indoc;

        fn verify(expected_ranges: Vec<Range>, text: &str) {
            let tree = open(text.trim());

            let mut traversal = TreeTraversal::default();
            traversal.visit(&tree, tree.root);
            let actual_ranges: Vec<_> = traversal
                .nodes
                .into_iter()
                .map(|node| tree.graph[node].range())
                .collect();

            println!("{:#?}", actual_ranges);
            assert_eq!(actual_ranges, expected_ranges);
        }

        #[test]
        fn empty_document() {
            verify(vec![Range::new_simple(0, 0, 0, 0)], "");
        }

        #[test]
        fn comment() {
            verify(
                vec![Range::new_simple(0, 0, 0, 3), Range::new_simple(0, 0, 0, 3)],
                "foo",
            );
        }

        #[test]
        fn preamble_no_left() {
            verify(
                vec![Range::new_simple(0, 0, 0, 9), Range::new_simple(0, 0, 0, 9)],
                "@preamble",
            );
        }

        #[test]
        fn preamble_no_content() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 10),
                    Range::new_simple(0, 0, 0, 10),
                ],
                "@preamble{",
            );
        }

        #[test]
        fn preamble_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 10, 0, 15),
                    Range::new_simple(0, 11, 0, 14),
                ],
                r#"@preamble{"foo""#,
            );
        }

        #[test]
        fn preamble_complete() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 10, 0, 15),
                    Range::new_simple(0, 11, 0, 14),
                ],
                r#"@preamble{"foo"}"#,
            );
        }

        #[test]
        fn string_no_left() {
            verify(
                vec![Range::new_simple(0, 0, 0, 7), Range::new_simple(0, 0, 0, 7)],
                r#"@string"#,
            );
        }

        #[test]
        fn string_no_name() {
            verify(
                vec![Range::new_simple(0, 0, 0, 8), Range::new_simple(0, 0, 0, 8)],
                r#"@string{"#,
            );
        }

        #[test]
        fn string_no_assign() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 11),
                    Range::new_simple(0, 0, 0, 11),
                ],
                r#"@string{foo"#,
            );
        }

        #[test]
        fn string_no_value() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 13),
                    Range::new_simple(0, 0, 0, 13),
                ],
                r#"@string{foo ="#,
            );
        }

        #[test]
        fn string_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 19),
                    Range::new_simple(0, 0, 0, 19),
                    Range::new_simple(0, 14, 0, 19),
                    Range::new_simple(0, 15, 0, 18),
                ],
                r#"@string{foo = "bar""#,
            );
        }

        #[test]
        fn string_complete() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 20),
                    Range::new_simple(0, 0, 0, 20),
                    Range::new_simple(0, 14, 0, 19),
                    Range::new_simple(0, 15, 0, 18),
                ],
                r#"@string{foo = "bar"}"#,
            );
        }

        #[test]
        fn entry_no_left() {
            verify(
                vec![Range::new_simple(0, 0, 0, 8), Range::new_simple(0, 0, 0, 8)],
                r#"@article"#,
            );
        }

        #[test]
        fn entry_no_key() {
            verify(
                vec![Range::new_simple(0, 0, 0, 9), Range::new_simple(0, 0, 0, 9)],
                r#"@article{"#,
            );
        }

        #[test]
        fn entry_no_comma() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 12),
                    Range::new_simple(0, 0, 0, 12),
                ],
                r#"@article{foo"#,
            );
        }

        #[test]
        fn entry_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 13),
                    Range::new_simple(0, 0, 0, 13),
                ],
                r#"@article{foo,"#,
            );
        }

        #[test]
        fn entry_parentheses() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 14),
                    Range::new_simple(0, 0, 0, 14),
                ],
                r#"@article(foo,)"#,
            );
        }

        #[test]
        fn field_no_assign() {
            verify(
                vec![
                    Range::new_simple(0, 0, 1, 10),
                    Range::new_simple(0, 0, 1, 10),
                    Range::new_simple(1, 4, 1, 10),
                ],
                indoc!(
                    r#"
                        @article{foo,
                            author
                    "#
                ),
            );
        }

        #[test]
        fn field_no_value() {
            verify(
                vec![
                    Range::new_simple(0, 0, 1, 12),
                    Range::new_simple(0, 0, 1, 12),
                    Range::new_simple(1, 4, 1, 12),
                ],
                indoc!(
                    r#"
                        @article{foo,
                            author =
                    "#
                ),
            );
        }

        #[test]
        fn field_no_comma() {
            verify(
                vec![
                    Range::new_simple(0, 0, 1, 16),
                    Range::new_simple(0, 0, 1, 16),
                    Range::new_simple(1, 4, 1, 16),
                    Range::new_simple(1, 13, 1, 16),
                ],
                indoc!(
                    r#"
                        @article{foo,
                            author = bar
                    "#
                ),
            );
        }

        #[test]
        fn field_complete() {
            verify(
                vec![
                    Range::new_simple(0, 0, 2, 1),
                    Range::new_simple(0, 0, 2, 1),
                    Range::new_simple(1, 4, 1, 17),
                    Range::new_simple(1, 13, 1, 16),
                ],
                indoc!(
                    r#"
                        @article{foo,
                            author = bar,
                        }
                    "#
                ),
            );
        }

        #[test]
        fn entry_two_fields() {
            verify(
                vec![
                    Range::new_simple(0, 0, 3, 1),
                    Range::new_simple(0, 0, 3, 1),
                    Range::new_simple(1, 4, 1, 17),
                    Range::new_simple(1, 13, 1, 16),
                    Range::new_simple(2, 4, 2, 16),
                    Range::new_simple(2, 12, 2, 15),
                ],
                indoc!(
                    r#"
                        @article{foo,
                            author = bar,
                            title = baz,
                        }
                    "#
                ),
            );
        }

        #[test]
        fn quoted_content_no_children() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 12),
                    Range::new_simple(0, 0, 0, 12),
                    Range::new_simple(0, 10, 0, 11),
                ],
                r#"@preamble{"}"#,
            );
        }

        #[test]
        fn quoted_content_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 10, 0, 15),
                    Range::new_simple(0, 11, 0, 15),
                ],
                r#"@preamble{"word}"#,
            );
        }

        #[test]
        fn braced_content_no_children() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 11),
                    Range::new_simple(0, 0, 0, 11),
                    Range::new_simple(0, 10, 0, 11),
                ],
                r#"@preamble{{"#,
            );
        }

        #[test]
        fn braced_content_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 0, 0, 15),
                    Range::new_simple(0, 10, 0, 15),
                    Range::new_simple(0, 11, 0, 15),
                ],
                r#"@preamble{{word"#,
            );
        }

        #[test]
        fn concat_no_right() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 0, 0, 16),
                    Range::new_simple(0, 10, 0, 15),
                    Range::new_simple(0, 10, 0, 13),
                ],
                r#"@preamble{foo #}"#,
            );
        }

        #[test]
        fn concat_complete() {
            verify(
                vec![
                    Range::new_simple(0, 0, 0, 20),
                    Range::new_simple(0, 0, 0, 20),
                    Range::new_simple(0, 10, 0, 19),
                    Range::new_simple(0, 10, 0, 13),
                    Range::new_simple(0, 16, 0, 19),
                ],
                r#"@preamble{foo # bar}"#,
            );
        }
    }
}
