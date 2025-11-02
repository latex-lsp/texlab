use expect_test::{Expect, expect};
use line_index::LineIndex;
use syntax::bibtex;

use crate::Options;

fn check(input: &str, expect: Expect) {
    let green = parser::parse_bibtex(input);
    let root = bibtex::SyntaxNode::new_root(green);
    let line_index = LineIndex::new(input);
    let output = crate::format(&root, &line_index, &Options::default());
    expect.assert_eq(&output);
}

#[test]
fn test_wrap_long_lines() {
    check(
        r#"@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.
    Lorem ipsum dolor sit amet,
    consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"#,
        expect![[r#"
            @article{foo,
                bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum
                       dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit
                       amet, consectetur adipiscing elit.},
            }"#]],
    );
}

#[test]
fn test_multiple_entries() {
    check(
        r#"@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
            Lorem ipsum dolor sit amet, 
consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}

@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
    Lorem ipsum dolor sit amet, 
consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"#,
        expect![[r#"
            @article{foo,
                bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum
                       dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit
                       amet, consectetur adipiscing elit.},
            }

            @article{foo,
                bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum
                       dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit
                       amet, consectetur adipiscing elit.},
            }"#]],
    );
}

#[test]
fn test_trailing_comma() {
    check(
        r#"@article{foo, bar = baz}"#,
        expect![[r#"
        @article{foo,
            bar = baz,
        }"#]],
    );
}

#[test]
fn test_insert_braces() {
    check(
        r#"@article{foo, bar = baz,"#,
        expect![[r#"
        @article{foo,
            bar = baz,
        }"#]],
    );
}

#[test]
fn test_comment() {
    check(
        r#"Foo Bar
@article{foo, bar = "\baz",}
Baz
@article{f,}
Qux"#,
        expect![[r#"
            Foo Bar
            @article{foo,
                bar = "\baz",
            }
            Baz
            @article{f,
            }
            Qux"#]],
    );
}

#[test]
fn test_command() {
    check(
        r#"@article{foo, bar = "\baz",}"#,
        expect![[r#"
        @article{foo,
            bar = "\baz",
        }"#]],
    );
}

#[test]
fn test_join_strings() {
    check(
        r#"@article{foo, bar = "baz" # "qux"}"#,
        expect![[r#"
        @article{foo,
            bar = "baz" # "qux",
        }"#]],
    );
}

#[test]
fn test_parens() {
    check(
        r#"@article(foo,)"#,
        expect![[r#"
        @article{foo,
        }"#]],
    );
}

#[test]
fn test_string() {
    check(
        r#"@string{foo="bar"}"#,
        expect![[r#"@string{foo = "bar"}"#]],
    );
}

#[test]
fn test_preamble() {
    check(
        r#"@preamble{
        "foo bar baz" }"#,
        expect![[r#"@preamble{"foo bar baz"}"#]],
    );
}
