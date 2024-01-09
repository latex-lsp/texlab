use expect_test::{expect, Expect};

use crate::{parse_latex, SyntaxConfig};

fn check(input: &str, expect: Expect) {
    let root = syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
    expect.assert_debug_eq(&root);
}

#[test]
fn test_block_comments() {
    check(
        r#"Foo
\iffalse
Test1
\fi
Bar
\iffalse
\fii
\fi
Baz"#,
        expect![[r#"
            ROOT@0..48
              PREAMBLE@0..48
                TEXT@0..4
                  WORD@0..3 "Foo"
                  LINE_BREAK@3..4 "\n"
                BLOCK_COMMENT@4..22
                  COMMAND_NAME@4..12 "\\iffalse"
                  LINE_BREAK@12..13 "\n"
                  WORD@13..18 "Test1"
                  LINE_BREAK@18..19 "\n"
                  COMMAND_NAME@19..22 "\\fi"
                LINE_BREAK@22..23 "\n"
                TEXT@23..27
                  WORD@23..26 "Bar"
                  LINE_BREAK@26..27 "\n"
                BLOCK_COMMENT@27..44
                  COMMAND_NAME@27..35 "\\iffalse"
                  LINE_BREAK@35..36 "\n"
                  COMMAND_NAME@36..40 "\\fii"
                  LINE_BREAK@40..41 "\n"
                  COMMAND_NAME@41..44 "\\fi"
                LINE_BREAK@44..45 "\n"
                TEXT@45..48
                  WORD@45..48 "Baz"

        "#]],
    );
}

#[test]
fn test_caption_default() {
    check(
        r#"\caption[qux]{Foo \Bar Baz}"#,
        expect![[r#"
        ROOT@0..27
          PREAMBLE@0..27
            CAPTION@0..27
              COMMAND_NAME@0..8 "\\caption"
              BRACK_GROUP@8..13
                L_BRACK@8..9 "["
                TEXT@9..12
                  WORD@9..12 "qux"
                R_BRACK@12..13 "]"
              CURLY_GROUP@13..27
                L_CURLY@13..14 "{"
                TEXT@14..18
                  WORD@14..17 "Foo"
                  WHITESPACE@17..18 " "
                GENERIC_COMMAND@18..23
                  COMMAND_NAME@18..22 "\\Bar"
                  WHITESPACE@22..23 " "
                TEXT@23..26
                  WORD@23..26 "Baz"
                R_CURLY@26..27 "}"

    "#]],
    );
}

#[test]
fn test_caption_default_error() {
    check(
        r#"\caption[qux]{Foo \Bar Baz"#,
        expect![[r#"
        ROOT@0..26
          PREAMBLE@0..26
            CAPTION@0..26
              COMMAND_NAME@0..8 "\\caption"
              BRACK_GROUP@8..13
                L_BRACK@8..9 "["
                TEXT@9..12
                  WORD@9..12 "qux"
                R_BRACK@12..13 "]"
              CURLY_GROUP@13..26
                L_CURLY@13..14 "{"
                TEXT@14..18
                  WORD@14..17 "Foo"
                  WHITESPACE@17..18 " "
                GENERIC_COMMAND@18..23
                  COMMAND_NAME@18..22 "\\Bar"
                  WHITESPACE@22..23 " "
                TEXT@23..26
                  WORD@23..26 "Baz"

    "#]],
    );
}

#[test]
fn test_caption_figure() {
    check(
        r#"\begin{figure}\caption{Foo}\end{figure}"#,
        expect![[r#"
        ROOT@0..39
          PREAMBLE@0..39
            ENVIRONMENT@0..39
              BEGIN@0..14
                COMMAND_NAME@0..6 "\\begin"
                CURLY_GROUP_WORD@6..14
                  L_CURLY@6..7 "{"
                  KEY@7..13
                    WORD@7..13 "figure"
                  R_CURLY@13..14 "}"
              CAPTION@14..27
                COMMAND_NAME@14..22 "\\caption"
                CURLY_GROUP@22..27
                  L_CURLY@22..23 "{"
                  TEXT@23..26
                    WORD@23..26 "Foo"
                  R_CURLY@26..27 "}"
              END@27..39
                COMMAND_NAME@27..31 "\\end"
                CURLY_GROUP_WORD@31..39
                  L_CURLY@31..32 "{"
                  KEY@32..38
                    WORD@32..38 "figure"
                  R_CURLY@38..39 "}"

    "#]],
    );
}

#[test]
fn test_caption_minimal() {
    check(
        r#"\caption{Foo \Bar Baz}"#,
        expect![[r#"
        ROOT@0..22
          PREAMBLE@0..22
            CAPTION@0..22
              COMMAND_NAME@0..8 "\\caption"
              CURLY_GROUP@8..22
                L_CURLY@8..9 "{"
                TEXT@9..13
                  WORD@9..12 "Foo"
                  WHITESPACE@12..13 " "
                GENERIC_COMMAND@13..18
                  COMMAND_NAME@13..17 "\\Bar"
                  WHITESPACE@17..18 " "
                TEXT@18..21
                  WORD@18..21 "Baz"
                R_CURLY@21..22 "}"

    "#]],
    );
}

#[test]
fn test_caption_minimal_error() {
    check(
        r#"\caption{Foo \Bar Baz"#,
        expect![[r#"
        ROOT@0..21
          PREAMBLE@0..21
            CAPTION@0..21
              COMMAND_NAME@0..8 "\\caption"
              CURLY_GROUP@8..21
                L_CURLY@8..9 "{"
                TEXT@9..13
                  WORD@9..12 "Foo"
                  WHITESPACE@12..13 " "
                GENERIC_COMMAND@13..18
                  COMMAND_NAME@13..17 "\\Bar"
                  WHITESPACE@17..18 " "
                TEXT@18..21
                  WORD@18..21 "Baz"

    "#]],
    );
}

#[test]
fn test_citation_empty() {
    check(
        r#"\cite{}"#,
        expect![[r#"
        ROOT@0..7
          PREAMBLE@0..7
            CITATION@0..7
              COMMAND_NAME@0..5 "\\cite"
              CURLY_GROUP_WORD_LIST@5..7
                L_CURLY@5..6 "{"
                R_CURLY@6..7 "}"

    "#]],
    );
}

#[test]
fn test_citation_missing_brace() {
    check(
        r#"\cite{foo"#,
        expect![[r#"
        ROOT@0..9
          PREAMBLE@0..9
            CITATION@0..9
              COMMAND_NAME@0..5 "\\cite"
              CURLY_GROUP_WORD_LIST@5..9
                L_CURLY@5..6 "{"
                KEY@6..9
                  WORD@6..9 "foo"

    "#]],
    );
}

#[test]
fn test_citation_multiple_keys() {
    check(
        r#"\cite{foo, bar}"#,
        expect![[r#"
        ROOT@0..15
          PREAMBLE@0..15
            CITATION@0..15
              COMMAND_NAME@0..5 "\\cite"
              CURLY_GROUP_WORD_LIST@5..15
                L_CURLY@5..6 "{"
                KEY@6..9
                  WORD@6..9 "foo"
                COMMA@9..10 ","
                WHITESPACE@10..11 " "
                KEY@11..14
                  WORD@11..14 "bar"
                R_CURLY@14..15 "}"

    "#]],
    );
}

#[test]
fn test_citation_prenote() {
    check(
        r#"\cite[foo]{bar}"#,
        expect![[r#"
        ROOT@0..15
          PREAMBLE@0..15
            CITATION@0..15
              COMMAND_NAME@0..5 "\\cite"
              BRACK_GROUP@5..10
                L_BRACK@5..6 "["
                TEXT@6..9
                  WORD@6..9 "foo"
                R_BRACK@9..10 "]"
              CURLY_GROUP_WORD_LIST@10..15
                L_CURLY@10..11 "{"
                KEY@11..14
                  WORD@11..14 "bar"
                R_CURLY@14..15 "}"

    "#]],
    );
}

#[test]
fn test_citation_prenote_postnote() {
    check(
        r#"\cite[foo][bar]{baz}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            CITATION@0..20
              COMMAND_NAME@0..5 "\\cite"
              BRACK_GROUP@5..10
                L_BRACK@5..6 "["
                TEXT@6..9
                  WORD@6..9 "foo"
                R_BRACK@9..10 "]"
              BRACK_GROUP@10..15
                L_BRACK@10..11 "["
                TEXT@11..14
                  WORD@11..14 "bar"
                R_BRACK@14..15 "]"
              CURLY_GROUP_WORD_LIST@15..20
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "baz"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_citation_redundant_comma() {
    check(
        r#"\cite{,foo,}"#,
        expect![[r#"
        ROOT@0..12
          PREAMBLE@0..12
            CITATION@0..12
              COMMAND_NAME@0..5 "\\cite"
              CURLY_GROUP_WORD_LIST@5..12
                L_CURLY@5..6 "{"
                COMMA@6..7 ","
                KEY@7..10
                  WORD@7..10 "foo"
                COMMA@10..11 ","
                R_CURLY@11..12 "}"

    "#]],
    );
}

#[test]
fn test_citation_simple() {
    check(
        r#"\cite{foo}"#,
        expect![[r#"
        ROOT@0..10
          PREAMBLE@0..10
            CITATION@0..10
              COMMAND_NAME@0..5 "\\cite"
              CURLY_GROUP_WORD_LIST@5..10
                L_CURLY@5..6 "{"
                KEY@6..9
                  WORD@6..9 "foo"
                R_CURLY@9..10 "}"

    "#]],
    );
}

#[test]
fn test_citation_star() {
    check(
        r#"\nocite{*}"#,
        expect![[r#"
        ROOT@0..10
          PREAMBLE@0..10
            CITATION@0..10
              COMMAND_NAME@0..7 "\\nocite"
              CURLY_GROUP_WORD_LIST@7..10
                L_CURLY@7..8 "{"
                KEY@8..9
                  WORD@8..9 "*"
                R_CURLY@9..10 "}"

    "#]],
    );
}

#[test]
fn test_color_definition_simple() {
    check(
        r#"\definecolor{foo}{rgb}{255,168,0}"#,
        expect![[r#"
        ROOT@0..33
          PREAMBLE@0..33
            COLOR_DEFINITION@0..33
              COMMAND_NAME@0..12 "\\definecolor"
              CURLY_GROUP_WORD@12..17
                L_CURLY@12..13 "{"
                KEY@13..16
                  WORD@13..16 "foo"
                R_CURLY@16..17 "}"
              CURLY_GROUP_WORD@17..22
                L_CURLY@17..18 "{"
                KEY@18..21
                  WORD@18..21 "rgb"
                R_CURLY@21..22 "}"
              CURLY_GROUP@22..33
                L_CURLY@22..23 "{"
                TEXT@23..32
                  WORD@23..26 "255"
                  COMMA@26..27 ","
                  WORD@27..30 "168"
                  COMMA@30..31 ","
                  WORD@31..32 "0"
                R_CURLY@32..33 "}"

    "#]],
    );
}

#[test]
fn test_color_reference_simple() {
    check(
        r#"\color{black}"#,
        expect![[r#"
        ROOT@0..13
          PREAMBLE@0..13
            COLOR_REFERENCE@0..13
              COMMAND_NAME@0..6 "\\color"
              CURLY_GROUP_WORD@6..13
                L_CURLY@6..7 "{"
                KEY@7..12
                  WORD@7..12 "black"
                R_CURLY@12..13 "}"

    "#]],
    );
}

#[test]
fn test_color_set_definition_error1() {
    check(
        r#"\definecolorset[ty]{rgb,HTML}{foo}{bar}"#,
        expect![[r#"
        ROOT@0..39
          PREAMBLE@0..39
            COLOR_SET_DEFINITION@0..39
              COMMAND_NAME@0..15 "\\definecolorset"
              BRACK_GROUP_WORD@15..19
                L_BRACK@15..16 "["
                KEY@16..18
                  WORD@16..18 "ty"
                R_BRACK@18..19 "]"
              CURLY_GROUP_WORD_LIST@19..29
                L_CURLY@19..20 "{"
                KEY@20..23
                  WORD@20..23 "rgb"
                COMMA@23..24 ","
                KEY@24..28
                  WORD@24..28 "HTML"
                R_CURLY@28..29 "}"
              CURLY_GROUP_WORD@29..34
                L_CURLY@29..30 "{"
                KEY@30..33
                  WORD@30..33 "foo"
                R_CURLY@33..34 "}"
              CURLY_GROUP_WORD@34..39
                L_CURLY@34..35 "{"
                KEY@35..38
                  WORD@35..38 "bar"
                R_CURLY@38..39 "}"

    "#]],
    );
}

#[test]
fn test_color_set_definition_error2() {
    check(
        r#"\definecolorset{rgb,HTML}{foo}"#,
        expect![[r#"
        ROOT@0..30
          PREAMBLE@0..30
            COLOR_SET_DEFINITION@0..30
              COMMAND_NAME@0..15 "\\definecolorset"
              CURLY_GROUP_WORD_LIST@15..25
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "rgb"
                COMMA@19..20 ","
                KEY@20..24
                  WORD@20..24 "HTML"
                R_CURLY@24..25 "}"
              CURLY_GROUP_WORD@25..30
                L_CURLY@25..26 "{"
                KEY@26..29
                  WORD@26..29 "foo"
                R_CURLY@29..30 "}"

    "#]],
    );
}

#[test]
fn test_color_set_definition_error3() {
    check(
        r#"\definecolorset{rgb,HTML}"#,
        expect![[r#"
        ROOT@0..25
          PREAMBLE@0..25
            COLOR_SET_DEFINITION@0..25
              COMMAND_NAME@0..15 "\\definecolorset"
              CURLY_GROUP_WORD_LIST@15..25
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "rgb"
                COMMA@19..20 ","
                KEY@20..24
                  WORD@20..24 "HTML"
                R_CURLY@24..25 "}"

    "#]],
    );
}

#[test]
fn test_color_set_definition_error4() {
    check(
        r#"\definecolorset"#,
        expect![[r#"
        ROOT@0..15
          PREAMBLE@0..15
            COLOR_SET_DEFINITION@0..15
              COMMAND_NAME@0..15 "\\definecolorset"

    "#]],
    );
}

#[test]
fn test_color_set_definition_simple() {
    check(
        r#"\definecolorset[ty]{rgb,HTML}{foo}{bar}{baz}"#,
        expect![[r#"
        ROOT@0..44
          PREAMBLE@0..44
            COLOR_SET_DEFINITION@0..44
              COMMAND_NAME@0..15 "\\definecolorset"
              BRACK_GROUP_WORD@15..19
                L_BRACK@15..16 "["
                KEY@16..18
                  WORD@16..18 "ty"
                R_BRACK@18..19 "]"
              CURLY_GROUP_WORD_LIST@19..29
                L_CURLY@19..20 "{"
                KEY@20..23
                  WORD@20..23 "rgb"
                COMMA@23..24 ","
                KEY@24..28
                  WORD@24..28 "HTML"
                R_CURLY@28..29 "}"
              CURLY_GROUP_WORD@29..34
                L_CURLY@29..30 "{"
                KEY@30..33
                  WORD@30..33 "foo"
                R_CURLY@33..34 "}"
              CURLY_GROUP_WORD@34..39
                L_CURLY@34..35 "{"
                KEY@35..38
                  WORD@35..38 "bar"
                R_CURLY@38..39 "}"
              CURLY_GROUP_WORD@39..44
                L_CURLY@39..40 "{"
                KEY@40..43
                  WORD@40..43 "baz"
                R_CURLY@43..44 "}"

    "#]],
    );
}

#[test]
fn test_command_definition_no_argc() {
    check(
        r#"\newcommand{\foo}{foo}"#,
        expect![[r#"
        ROOT@0..22
          PREAMBLE@0..22
            COMMAND_DEFINITION@0..22
              COMMAND_NAME@0..11 "\\newcommand"
              CURLY_GROUP_COMMAND@11..17
                L_CURLY@11..12 "{"
                COMMAND_NAME@12..16 "\\foo"
                R_CURLY@16..17 "}"
              CURLY_GROUP@17..22
                L_CURLY@17..18 "{"
                TEXT@18..21
                  WORD@18..21 "foo"
                R_CURLY@21..22 "}"

    "#]],
    );
}

#[test]
fn test_command_definition_no_impl() {
    check(
        r#"\newcommand{\foo}"#,
        expect![[r#"
        ROOT@0..17
          PREAMBLE@0..17
            COMMAND_DEFINITION@0..17
              COMMAND_NAME@0..11 "\\newcommand"
              CURLY_GROUP_COMMAND@11..17
                L_CURLY@11..12 "{"
                COMMAND_NAME@12..16 "\\foo"
                R_CURLY@16..17 "}"

    "#]],
    );
}

#[test]
fn test_command_definition_no_impl_error() {
    check(
        r#"\newcommand{\foo"#,
        expect![[r#"
        ROOT@0..16
          PREAMBLE@0..16
            COMMAND_DEFINITION@0..16
              COMMAND_NAME@0..11 "\\newcommand"
              CURLY_GROUP_COMMAND@11..16
                L_CURLY@11..12 "{"
                COMMAND_NAME@12..16 "\\foo"

    "#]],
    );
}

#[test]
fn test_command_definition_optional() {
    check(
        r#"\newcommand{\foo}[1][def]{#1}"#,
        expect![[r##"
        ROOT@0..29
          PREAMBLE@0..29
            COMMAND_DEFINITION@0..29
              COMMAND_NAME@0..11 "\\newcommand"
              CURLY_GROUP_COMMAND@11..17
                L_CURLY@11..12 "{"
                COMMAND_NAME@12..16 "\\foo"
                R_CURLY@16..17 "}"
              BRACK_GROUP_WORD@17..20
                L_BRACK@17..18 "["
                KEY@18..19
                  WORD@18..19 "1"
                R_BRACK@19..20 "]"
              BRACK_GROUP@20..25
                L_BRACK@20..21 "["
                TEXT@21..24
                  WORD@21..24 "def"
                R_BRACK@24..25 "]"
              CURLY_GROUP@25..29
                L_CURLY@25..26 "{"
                TEXT@26..28
                  WORD@26..28 "#1"
                R_CURLY@28..29 "}"

    "##]],
    );
}

#[test]
fn test_command_definition_simple() {
    check(
        r#"\newcommand[1]{\id}{#1}"#,
        expect![[r##"
        ROOT@0..23
          PREAMBLE@0..23
            COMMAND_DEFINITION@0..19
              COMMAND_NAME@0..11 "\\newcommand"
              BRACK_GROUP_WORD@11..14
                L_BRACK@11..12 "["
                KEY@12..13
                  WORD@12..13 "1"
                R_BRACK@13..14 "]"
              CURLY_GROUP@14..19
                L_CURLY@14..15 "{"
                GENERIC_COMMAND@15..18
                  COMMAND_NAME@15..18 "\\id"
                R_CURLY@18..19 "}"
            CURLY_GROUP@19..23
              L_CURLY@19..20 "{"
              TEXT@20..22
                WORD@20..22 "#1"
              R_CURLY@22..23 "}"

    "##]],
    );
}

#[test]
fn test_command_definition_with_begin() {
    check(
        r#"\newcommand{\CVSubHeadingListStart}{\begin{itemize}[leftmargin=0.5cm, label={}]}"#,
        expect![[r#"
            ROOT@0..80
              PREAMBLE@0..80
                COMMAND_DEFINITION@0..80
                  COMMAND_NAME@0..11 "\\newcommand"
                  CURLY_GROUP_COMMAND@11..35
                    L_CURLY@11..12 "{"
                    COMMAND_NAME@12..34 "\\CVSubHeadingListStart"
                    R_CURLY@34..35 "}"
                  CURLY_GROUP@35..80
                    L_CURLY@35..36 "{"
                    BEGIN@36..79
                      COMMAND_NAME@36..42 "\\begin"
                      CURLY_GROUP_WORD@42..51
                        L_CURLY@42..43 "{"
                        KEY@43..50
                          WORD@43..50 "itemize"
                        R_CURLY@50..51 "}"
                      BRACK_GROUP@51..79
                        L_BRACK@51..52 "["
                        TEXT@52..62
                          WORD@52..62 "leftmargin"
                        EQUALITY_SIGN@62..63 "="
                        TEXT@63..75
                          WORD@63..68 "0.5cm"
                          COMMA@68..69 ","
                          WHITESPACE@69..70 " "
                          WORD@70..75 "label"
                        EQUALITY_SIGN@75..76 "="
                        CURLY_GROUP@76..78
                          L_CURLY@76..77 "{"
                          R_CURLY@77..78 "}"
                        R_BRACK@78..79 "]"
                    R_CURLY@79..80 "}"

        "#]],
    );
}

#[test]
fn test_math_operator_no_impl() {
    check(
        r#"\DeclareMathOperator{\foo}"#,
        expect![[r#"
        ROOT@0..26
          PREAMBLE@0..26
            MATH_OPERATOR@0..26
              COMMAND_NAME@0..20 "\\DeclareMathOperator"
              CURLY_GROUP_COMMAND@20..26
                L_CURLY@20..21 "{"
                COMMAND_NAME@21..25 "\\foo"
                R_CURLY@25..26 "}"

    "#]],
    );
}

#[test]
fn test_math_operator_simple() {
    check(
        r#"\DeclareMathOperator{\foo}{foo}"#,
        expect![[r#"
        ROOT@0..31
          PREAMBLE@0..31
            MATH_OPERATOR@0..31
              COMMAND_NAME@0..20 "\\DeclareMathOperator"
              CURLY_GROUP_COMMAND@20..26
                L_CURLY@20..21 "{"
                COMMAND_NAME@21..25 "\\foo"
                R_CURLY@25..26 "}"
              CURLY_GROUP@26..31
                L_CURLY@26..27 "{"
                TEXT@27..30
                  WORD@27..30 "foo"
                R_CURLY@30..31 "}"

    "#]],
    );
}

#[test]
fn test_environment_asymptote() {
    check(
        r#"\begin{asy}
    printf("Hello World\n");
\end{asy}"#,
        expect![[r#"
            ROOT@0..50
              PREAMBLE@0..50
                ENVIRONMENT@0..50
                  BEGIN@0..16
                    COMMAND_NAME@0..6 "\\begin"
                    CURLY_GROUP_WORD@6..16
                      L_CURLY@6..7 "{"
                      KEY@7..10
                        WORD@7..10 "asy"
                      R_CURLY@10..11 "}"
                      LINE_BREAK@11..12 "\n"
                      WHITESPACE@12..16 "    "
                  TEXT@16..22
                    WORD@16..22 "printf"
                  MIXED_GROUP@22..39
                    L_PAREN@22..23 "("
                    TEXT@23..35
                      WORD@23..29 "\"Hello"
                      WHITESPACE@29..30 " "
                      WORD@30..35 "World"
                    GENERIC_COMMAND@35..37
                      COMMAND_NAME@35..37 "\\n"
                    TEXT@37..38
                      WORD@37..38 "\""
                    R_PAREN@38..39 ")"
                  TEXT@39..41
                    WORD@39..40 ";"
                    LINE_BREAK@40..41 "\n"
                  END@41..50
                    COMMAND_NAME@41..45 "\\end"
                    CURLY_GROUP_WORD@45..50
                      L_CURLY@45..46 "{"
                      KEY@46..49
                        WORD@46..49 "asy"
                      R_CURLY@49..50 "}"

        "#]],
    );
}

#[test]
fn test_environment_definition() {
    check(
        r#"\newenvironment{bar}[1]{\begin{foo}}{\end{foo}}"#,
        expect![[r#"
            ROOT@0..47
              PREAMBLE@0..47
                ENVIRONMENT_DEFINITION@0..47
                  COMMAND_NAME@0..15 "\\newenvironment"
                  CURLY_GROUP_WORD@15..20
                    L_CURLY@15..16 "{"
                    KEY@16..19
                      WORD@16..19 "bar"
                    R_CURLY@19..20 "}"
                  BRACK_GROUP_WORD@20..23
                    L_BRACK@20..21 "["
                    KEY@21..22
                      WORD@21..22 "1"
                    R_BRACK@22..23 "]"
                  CURLY_GROUP@23..36
                    L_CURLY@23..24 "{"
                    GENERIC_COMMAND@24..35
                      COMMAND_NAME@24..30 "\\begin"
                      CURLY_GROUP@30..35
                        L_CURLY@30..31 "{"
                        TEXT@31..34
                          WORD@31..34 "foo"
                        R_CURLY@34..35 "}"
                    R_CURLY@35..36 "}"
                  CURLY_GROUP@36..47
                    L_CURLY@36..37 "{"
                    GENERIC_COMMAND@37..46
                      COMMAND_NAME@37..41 "\\end"
                      CURLY_GROUP@41..46
                        L_CURLY@41..42 "{"
                        TEXT@42..45
                          WORD@42..45 "foo"
                        R_CURLY@45..46 "}"
                    R_CURLY@46..47 "}"

        "#]],
    );
}

#[test]
fn test_environment_definition_optional_arg() {
    check(
        r#"\newenvironment{foo}[1][default]{begin}{end}"#,
        expect![[r#"
        ROOT@0..44
          PREAMBLE@0..44
            ENVIRONMENT_DEFINITION@0..44
              COMMAND_NAME@0..15 "\\newenvironment"
              CURLY_GROUP_WORD@15..20
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "foo"
                R_CURLY@19..20 "}"
              BRACK_GROUP_WORD@20..23
                L_BRACK@20..21 "["
                KEY@21..22
                  WORD@21..22 "1"
                R_BRACK@22..23 "]"
              BRACK_GROUP@23..32
                L_BRACK@23..24 "["
                TEXT@24..31
                  WORD@24..31 "default"
                R_BRACK@31..32 "]"
              CURLY_GROUP@32..39
                L_CURLY@32..33 "{"
                TEXT@33..38
                  WORD@33..38 "begin"
                R_CURLY@38..39 "}"
              CURLY_GROUP@39..44
                L_CURLY@39..40 "{"
                TEXT@40..43
                  WORD@40..43 "end"
                R_CURLY@43..44 "}"

    "#]],
    );
}

#[test]
fn test_environment_nested() {
    check(
        r#"\begin{foo} \begin{qux} \end{baz} \end{bar}"#,
        expect![[r#"
        ROOT@0..43
          PREAMBLE@0..43
            ENVIRONMENT@0..43
              BEGIN@0..12
                COMMAND_NAME@0..6 "\\begin"
                CURLY_GROUP_WORD@6..12
                  L_CURLY@6..7 "{"
                  KEY@7..10
                    WORD@7..10 "foo"
                  R_CURLY@10..11 "}"
                  WHITESPACE@11..12 " "
              ENVIRONMENT@12..34
                BEGIN@12..24
                  COMMAND_NAME@12..18 "\\begin"
                  CURLY_GROUP_WORD@18..24
                    L_CURLY@18..19 "{"
                    KEY@19..22
                      WORD@19..22 "qux"
                    R_CURLY@22..23 "}"
                    WHITESPACE@23..24 " "
                END@24..34
                  COMMAND_NAME@24..28 "\\end"
                  CURLY_GROUP_WORD@28..34
                    L_CURLY@28..29 "{"
                    KEY@29..32
                      WORD@29..32 "baz"
                    R_CURLY@32..33 "}"
                    WHITESPACE@33..34 " "
              END@34..43
                COMMAND_NAME@34..38 "\\end"
                CURLY_GROUP_WORD@38..43
                  L_CURLY@38..39 "{"
                  KEY@39..42
                    WORD@39..42 "bar"
                  R_CURLY@42..43 "}"

    "#]],
    );
}

#[test]
fn test_environment_nested_missing_braces() {
    check(
        r#"\begin{foo \begin{qux Hello World \end{baz} \end{bar"#,
        expect![[r#"
            ROOT@0..52
              PREAMBLE@0..52
                ENVIRONMENT@0..52
                  BEGIN@0..11
                    COMMAND_NAME@0..6 "\\begin"
                    CURLY_GROUP_WORD@6..11
                      L_CURLY@6..7 "{"
                      KEY@7..11
                        WORD@7..10 "foo"
                        WHITESPACE@10..11 " "
                  ENVIRONMENT@11..44
                    BEGIN@11..34
                      COMMAND_NAME@11..17 "\\begin"
                      CURLY_GROUP_WORD@17..34
                        L_CURLY@17..18 "{"
                        KEY@18..34
                          WORD@18..21 "qux"
                          WHITESPACE@21..22 " "
                          WORD@22..27 "Hello"
                          WHITESPACE@27..28 " "
                          WORD@28..33 "World"
                          WHITESPACE@33..34 " "
                    END@34..44
                      COMMAND_NAME@34..38 "\\end"
                      CURLY_GROUP_WORD@38..44
                        L_CURLY@38..39 "{"
                        KEY@39..42
                          WORD@39..42 "baz"
                        R_CURLY@42..43 "}"
                        WHITESPACE@43..44 " "
                  END@44..52
                    COMMAND_NAME@44..48 "\\end"
                    CURLY_GROUP_WORD@48..52
                      L_CURLY@48..49 "{"
                      KEY@49..52
                        WORD@49..52 "bar"

        "#]],
    );
}

#[test]
fn test_environment_simple() {
    check(
        r#"\begin{foo} Hello World \end{bar}"#,
        expect![[r#"
        ROOT@0..33
          PREAMBLE@0..33
            ENVIRONMENT@0..33
              BEGIN@0..12
                COMMAND_NAME@0..6 "\\begin"
                CURLY_GROUP_WORD@6..12
                  L_CURLY@6..7 "{"
                  KEY@7..10
                    WORD@7..10 "foo"
                  R_CURLY@10..11 "}"
                  WHITESPACE@11..12 " "
              TEXT@12..24
                WORD@12..17 "Hello"
                WHITESPACE@17..18 " "
                WORD@18..23 "World"
                WHITESPACE@23..24 " "
              END@24..33
                COMMAND_NAME@24..28 "\\end"
                CURLY_GROUP_WORD@28..33
                  L_CURLY@28..29 "{"
                  KEY@29..32
                    WORD@29..32 "bar"
                  R_CURLY@32..33 "}"

    "#]],
    );
}

#[test]
fn test_equation() {
    check(
        r#"\[ foo bar \]"#,
        expect![[r#"
        ROOT@0..13
          PREAMBLE@0..13
            EQUATION@0..13
              COMMAND_NAME@0..2 "\\["
              WHITESPACE@2..3 " "
              TEXT@3..11
                WORD@3..6 "foo"
                WHITESPACE@6..7 " "
                WORD@7..10 "bar"
                WHITESPACE@10..11 " "
              COMMAND_NAME@11..13 "\\]"

    "#]],
    );
}

#[test]
fn test_equation_missing_begin() {
    check(
        r#"\begin{a} foo bar \] \end{b}"#,
        expect![[r#"
        ROOT@0..28
          PREAMBLE@0..28
            ENVIRONMENT@0..28
              BEGIN@0..10
                COMMAND_NAME@0..6 "\\begin"
                CURLY_GROUP_WORD@6..10
                  L_CURLY@6..7 "{"
                  KEY@7..8
                    WORD@7..8 "a"
                  R_CURLY@8..9 "}"
                  WHITESPACE@9..10 " "
              TEXT@10..18
                WORD@10..13 "foo"
                WHITESPACE@13..14 " "
                WORD@14..17 "bar"
                WHITESPACE@17..18 " "
              GENERIC_COMMAND@18..21
                COMMAND_NAME@18..20 "\\]"
                WHITESPACE@20..21 " "
              END@21..28
                COMMAND_NAME@21..25 "\\end"
                CURLY_GROUP_WORD@25..28
                  L_CURLY@25..26 "{"
                  KEY@26..27
                    WORD@26..27 "b"
                  R_CURLY@27..28 "}"

    "#]],
    );
}

#[test]
fn test_generic_command_args() {
    check(
        r#"\foo{bar}[qux]"#,
        expect![[r#"
        ROOT@0..14
          PREAMBLE@0..14
            GENERIC_COMMAND@0..14
              COMMAND_NAME@0..4 "\\foo"
              CURLY_GROUP@4..9
                L_CURLY@4..5 "{"
                TEXT@5..8
                  WORD@5..8 "bar"
                R_CURLY@8..9 "}"
              MIXED_GROUP@9..14
                L_BRACK@9..10 "["
                TEXT@10..13
                  WORD@10..13 "qux"
                R_BRACK@13..14 "]"

    "#]],
    );
}

#[test]
fn test_generic_command_empty() {
    check(
        r#"\foo"#,
        expect![[r#"
        ROOT@0..4
          PREAMBLE@0..4
            GENERIC_COMMAND@0..4
              COMMAND_NAME@0..4 "\\foo"

    "#]],
    );
}

#[test]
fn test_generic_command_escape() {
    check(
        r#"\#"#,
        expect![[r#"
        ROOT@0..2
          PREAMBLE@0..2
            GENERIC_COMMAND@0..2
              COMMAND_NAME@0..2 "\\#"

    "#]],
    );
}

#[test]
fn test_acronym_declaration() {
    check(
        r#"\DeclareAcronym{eg}{short = e.g,long = for example,tag = abbrev}"#,
        expect![[r#"
            ROOT@0..64
              PREAMBLE@0..64
                ACRONYM_DECLARATION@0..64
                  COMMAND_NAME@0..15 "\\DeclareAcronym"
                  CURLY_GROUP_WORD@15..19
                    L_CURLY@15..16 "{"
                    KEY@16..18
                      WORD@16..18 "eg"
                    R_CURLY@18..19 "}"
                  CURLY_GROUP_KEY_VALUE@19..64
                    L_CURLY@19..20 "{"
                    KEY_VALUE_BODY@20..63
                      KEY_VALUE_PAIR@20..31
                        KEY@20..26
                          WORD@20..25 "short"
                          WHITESPACE@25..26 " "
                        EQUALITY_SIGN@26..27 "="
                        WHITESPACE@27..28 " "
                        VALUE@28..31
                          TEXT@28..31
                            WORD@28..31 "e.g"
                      COMMA@31..32 ","
                      KEY_VALUE_PAIR@32..50
                        KEY@32..37
                          WORD@32..36 "long"
                          WHITESPACE@36..37 " "
                        EQUALITY_SIGN@37..38 "="
                        WHITESPACE@38..39 " "
                        VALUE@39..50
                          TEXT@39..50
                            WORD@39..42 "for"
                            WHITESPACE@42..43 " "
                            WORD@43..50 "example"
                      COMMA@50..51 ","
                      KEY_VALUE_PAIR@51..63
                        KEY@51..55
                          WORD@51..54 "tag"
                          WHITESPACE@54..55 " "
                        EQUALITY_SIGN@55..56 "="
                        WHITESPACE@56..57 " "
                        VALUE@57..63
                          TEXT@57..63
                            WORD@57..63 "abbrev"
                    R_CURLY@63..64 "}"

        "#]],
    );
}

#[test]
fn test_acronym_definition_options() {
    check(
        r#"\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}"#,
        expect![[r#"
            ROOT@0..76
              PREAMBLE@0..76
                ACRONYM_DEFINITION@0..76
                  COMMAND_NAME@0..11 "\\newacronym"
                  BRACK_GROUP_KEY_VALUE@11..43
                    L_BRACK@11..12 "["
                    KEY_VALUE_BODY@12..42
                      KEY_VALUE_PAIR@12..42
                        KEY@12..22
                          WORD@12..22 "longplural"
                        EQUALITY_SIGN@22..23 "="
                        VALUE@23..42
                          CURLY_GROUP@23..42
                            L_CURLY@23..24 "{"
                            TEXT@24..41
                              WORD@24..30 "Frames"
                              WHITESPACE@30..31 " "
                              WORD@31..34 "per"
                              WHITESPACE@34..35 " "
                              WORD@35..41 "Second"
                            R_CURLY@41..42 "}"
                    R_BRACK@42..43 "]"
                  CURLY_GROUP_WORD@43..53
                    L_CURLY@43..44 "{"
                    KEY@44..52
                      WORD@44..52 "fpsLabel"
                    R_CURLY@52..53 "}"
                  CURLY_GROUP@53..58
                    L_CURLY@53..54 "{"
                    TEXT@54..57
                      WORD@54..57 "FPS"
                    R_CURLY@57..58 "}"
                  CURLY_GROUP@58..76
                    L_CURLY@58..59 "{"
                    TEXT@59..75
                      WORD@59..64 "Frame"
                      WHITESPACE@64..65 " "
                      WORD@65..68 "per"
                      WHITESPACE@68..69 " "
                      WORD@69..75 "Second"
                    R_CURLY@75..76 "}"

        "#]],
    );
}

#[test]
fn test_acronym_definition_simple() {
    check(
        r#"\newacronym{fpsLabel}{FPS}{Frame per Second}"#,
        expect![[r#"
        ROOT@0..44
          PREAMBLE@0..44
            ACRONYM_DEFINITION@0..44
              COMMAND_NAME@0..11 "\\newacronym"
              CURLY_GROUP_WORD@11..21
                L_CURLY@11..12 "{"
                KEY@12..20
                  WORD@12..20 "fpsLabel"
                R_CURLY@20..21 "}"
              CURLY_GROUP@21..26
                L_CURLY@21..22 "{"
                TEXT@22..25
                  WORD@22..25 "FPS"
                R_CURLY@25..26 "}"
              CURLY_GROUP@26..44
                L_CURLY@26..27 "{"
                TEXT@27..43
                  WORD@27..32 "Frame"
                  WHITESPACE@32..33 " "
                  WORD@33..36 "per"
                  WHITESPACE@36..37 " "
                  WORD@37..43 "Second"
                R_CURLY@43..44 "}"

    "#]],
    );
}

#[test]
fn test_acronym_package() {
    check(
        r#"\acro{fps}[FPS]{Frames per Second}
"#,
        expect![[r#"
            ROOT@0..35
              PREAMBLE@0..35
                ACRONYM_DEFINITION@0..35
                  COMMAND_NAME@0..5 "\\acro"
                  CURLY_GROUP_WORD@5..10
                    L_CURLY@5..6 "{"
                    KEY@6..9
                      WORD@6..9 "fps"
                    R_CURLY@9..10 "}"
                  BRACK_GROUP@10..15
                    L_BRACK@10..11 "["
                    TEXT@11..14
                      WORD@11..14 "FPS"
                    R_BRACK@14..15 "]"
                  CURLY_GROUP@15..35
                    L_CURLY@15..16 "{"
                    TEXT@16..33
                      WORD@16..22 "Frames"
                      WHITESPACE@22..23 " "
                      WORD@23..26 "per"
                      WHITESPACE@26..27 " "
                      WORD@27..33 "Second"
                    R_CURLY@33..34 "}"
                    LINE_BREAK@34..35 "\n"

        "#]],
    );
}

#[test]
fn test_acronym_reference_options() {
    check(
        r#"\acrshort[foo=bar,baz]{fpsLabel}"#,
        expect![[r#"
        ROOT@0..32
          PREAMBLE@0..32
            ACRONYM_REFERENCE@0..32
              COMMAND_NAME@0..9 "\\acrshort"
              BRACK_GROUP_KEY_VALUE@9..22
                L_BRACK@9..10 "["
                KEY_VALUE_BODY@10..21
                  KEY_VALUE_PAIR@10..17
                    KEY@10..13
                      WORD@10..13 "foo"
                    EQUALITY_SIGN@13..14 "="
                    VALUE@14..17
                      TEXT@14..17
                        WORD@14..17 "bar"
                  COMMA@17..18 ","
                  KEY_VALUE_PAIR@18..21
                    KEY@18..21
                      WORD@18..21 "baz"
                R_BRACK@21..22 "]"
              CURLY_GROUP_WORD@22..32
                L_CURLY@22..23 "{"
                KEY@23..31
                  WORD@23..31 "fpsLabel"
                R_CURLY@31..32 "}"

    "#]],
    );
}

#[test]
fn test_acronym_reference_simple() {
    check(
        r#"\acrshort{fpsLabel}"#,
        expect![[r#"
        ROOT@0..19
          PREAMBLE@0..19
            ACRONYM_REFERENCE@0..19
              COMMAND_NAME@0..9 "\\acrshort"
              CURLY_GROUP_WORD@9..19
                L_CURLY@9..10 "{"
                KEY@10..18
                  WORD@10..18 "fpsLabel"
                R_CURLY@18..19 "}"

    "#]],
    );
}

#[test]
fn test_glossary_entry_definition_simple() {
    check(
        r#"\newglossaryentry{foo}{bar = baz, qux,}"#,
        expect![[r#"
        ROOT@0..39
          PREAMBLE@0..39
            GLOSSARY_ENTRY_DEFINITION@0..39
              COMMAND_NAME@0..17 "\\newglossaryentry"
              CURLY_GROUP_WORD@17..22
                L_CURLY@17..18 "{"
                KEY@18..21
                  WORD@18..21 "foo"
                R_CURLY@21..22 "}"
              CURLY_GROUP_KEY_VALUE@22..39
                L_CURLY@22..23 "{"
                KEY_VALUE_BODY@23..38
                  KEY_VALUE_PAIR@23..32
                    KEY@23..27
                      WORD@23..26 "bar"
                      WHITESPACE@26..27 " "
                    EQUALITY_SIGN@27..28 "="
                    WHITESPACE@28..29 " "
                    VALUE@29..32
                      TEXT@29..32
                        WORD@29..32 "baz"
                  COMMA@32..33 ","
                  WHITESPACE@33..34 " "
                  KEY_VALUE_PAIR@34..37
                    KEY@34..37
                      WORD@34..37 "qux"
                  COMMA@37..38 ","
                R_CURLY@38..39 "}"

    "#]],
    );
}

#[test]
fn test_glossary_entry_reference_options() {
    check(
        r#"\gls[foo = bar, qux]{baz}"#,
        expect![[r#"
        ROOT@0..25
          PREAMBLE@0..25
            GLOSSARY_ENTRY_REFERENCE@0..25
              COMMAND_NAME@0..4 "\\gls"
              BRACK_GROUP_KEY_VALUE@4..20
                L_BRACK@4..5 "["
                KEY_VALUE_BODY@5..19
                  KEY_VALUE_PAIR@5..14
                    KEY@5..9
                      WORD@5..8 "foo"
                      WHITESPACE@8..9 " "
                    EQUALITY_SIGN@9..10 "="
                    WHITESPACE@10..11 " "
                    VALUE@11..14
                      TEXT@11..14
                        WORD@11..14 "bar"
                  COMMA@14..15 ","
                  WHITESPACE@15..16 " "
                  KEY_VALUE_PAIR@16..19
                    KEY@16..19
                      WORD@16..19 "qux"
                R_BRACK@19..20 "]"
              CURLY_GROUP_WORD@20..25
                L_CURLY@20..21 "{"
                KEY@21..24
                  WORD@21..24 "baz"
                R_CURLY@24..25 "}"

    "#]],
    );
}

#[test]
fn test_glossary_entry_reference_simple() {
    check(
        r#"\gls{foo}"#,
        expect![[r#"
        ROOT@0..9
          PREAMBLE@0..9
            GLOSSARY_ENTRY_REFERENCE@0..9
              COMMAND_NAME@0..4 "\\gls"
              CURLY_GROUP_WORD@4..9
                L_CURLY@4..5 "{"
                KEY@5..8
                  WORD@5..8 "foo"
                R_CURLY@8..9 "}"

    "#]],
    );
}

#[test]
fn test_graphics_path() {
    check(
        r#"\graphicspath{{../figures/}}"#,
        expect![[r#"
        ROOT@0..28
          PREAMBLE@0..28
            GRAPHICS_PATH@0..28
              COMMAND_NAME@0..13 "\\graphicspath"
              CURLY_GROUP@13..28
                L_CURLY@13..14 "{"
                CURLY_GROUP_WORD@14..27
                  L_CURLY@14..15 "{"
                  KEY@15..26
                    WORD@15..26 "../figures/"
                  R_CURLY@26..27 "}"
                R_CURLY@27..28 "}"

    "#]],
    );
}

#[test]
fn test_graphics_path_command() {
    check(
        r#"\graphicspath{{\subfix{./img/}}}"#,
        expect![[r#"
        ROOT@0..32
          PREAMBLE@0..32
            GRAPHICS_PATH@0..32
              COMMAND_NAME@0..13 "\\graphicspath"
              CURLY_GROUP@13..32
                L_CURLY@13..14 "{"
                CURLY_GROUP_WORD@14..31
                  L_CURLY@14..15 "{"
                  KEY@15..30
                    COMMAND_NAME@15..22 "\\subfix"
                    CURLY_GROUP_WORD@22..30
                      L_CURLY@22..23 "{"
                      KEY@23..29
                        WORD@23..29 "./img/"
                      R_CURLY@29..30 "}"
                  R_CURLY@30..31 "}"
                R_CURLY@31..32 "}"

    "#]],
    );
}

#[test]
fn test_graphics_path_options() {
    check(
        r#"\graphicspath[foo]{{../figures/}}"#,
        expect![[r#"
        ROOT@0..33
          PREAMBLE@0..33
            GRAPHICS_PATH@0..13
              COMMAND_NAME@0..13 "\\graphicspath"
            MIXED_GROUP@13..18
              L_BRACK@13..14 "["
              TEXT@14..17
                WORD@14..17 "foo"
              R_BRACK@17..18 "]"
            CURLY_GROUP@18..33
              L_CURLY@18..19 "{"
              CURLY_GROUP@19..32
                L_CURLY@19..20 "{"
                TEXT@20..31
                  WORD@20..31 "../figures/"
                R_CURLY@31..32 "}"
              R_CURLY@32..33 "}"

    "#]],
    );
}

#[test]
fn test_curly_group_missing_end() {
    check(
        r#"{hello world"#,
        expect![[r#"
        ROOT@0..12
          PREAMBLE@0..12
            CURLY_GROUP@0..12
              L_CURLY@0..1 "{"
              TEXT@1..12
                WORD@1..6 "hello"
                WHITESPACE@6..7 " "
                WORD@7..12 "world"

    "#]],
    );
}

#[test]
fn test_curly_group_simple() {
    check(
        r#"{hello world}"#,
        expect![[r#"
        ROOT@0..13
          PREAMBLE@0..13
            CURLY_GROUP@0..13
              L_CURLY@0..1 "{"
              TEXT@1..12
                WORD@1..6 "hello"
                WHITESPACE@6..7 " "
                WORD@7..12 "world"
              R_CURLY@12..13 "}"

    "#]],
    );
}

#[test]
fn test_equation_missing_end() {
    check(
        r#"\begin{a} \[ foo bar \end{b}"#,
        expect![[r#"
        ROOT@0..28
          PREAMBLE@0..28
            ENVIRONMENT@0..28
              BEGIN@0..10
                COMMAND_NAME@0..6 "\\begin"
                CURLY_GROUP_WORD@6..10
                  L_CURLY@6..7 "{"
                  KEY@7..8
                    WORD@7..8 "a"
                  R_CURLY@8..9 "}"
                  WHITESPACE@9..10 " "
              EQUATION@10..21
                COMMAND_NAME@10..12 "\\["
                WHITESPACE@12..13 " "
                TEXT@13..21
                  WORD@13..16 "foo"
                  WHITESPACE@16..17 " "
                  WORD@17..20 "bar"
                  WHITESPACE@20..21 " "
              END@21..28
                COMMAND_NAME@21..25 "\\end"
                CURLY_GROUP_WORD@25..28
                  L_CURLY@25..26 "{"
                  KEY@26..27
                    WORD@26..27 "b"
                  R_CURLY@27..28 "}"

    "#]],
    );
}

#[test]
fn test_escaped_brackets() {
    check(
        r#"{[}{]}"#,
        expect![[r#"
        ROOT@0..6
          PREAMBLE@0..6
            CURLY_GROUP@0..3
              L_CURLY@0..1 "{"
              MIXED_GROUP@1..2
                L_BRACK@1..2 "["
              R_CURLY@2..3 "}"
            CURLY_GROUP@3..6
              L_CURLY@3..4 "{"
              ERROR@4..5
                R_BRACK@4..5 "]"
              R_CURLY@5..6 "}"

    "#]],
    );
}

#[test]
fn test_unmatched_braces() {
    check(
        r#"}{"#,
        expect![[r#"
        ROOT@0..2
          PREAMBLE@0..2
            ERROR@0..1
              R_CURLY@0..1 "}"
            CURLY_GROUP@1..2
              L_CURLY@1..2 "{"

    "#]],
    );
}

#[test]
fn test_unmatched_brackets() {
    check(
        r#"]["#,
        expect![[r#"
        ROOT@0..2
          PREAMBLE@0..2
            ERROR@0..1
              R_BRACK@0..1 "]"
            MIXED_GROUP@1..2
              L_BRACK@1..2 "["

    "#]],
    );
}

#[test]
fn test_unmatched_brackets_with_group() {
    check(
        r#"{][}"#,
        expect![[r#"
        ROOT@0..4
          PREAMBLE@0..4
            CURLY_GROUP@0..4
              L_CURLY@0..1 "{"
              ERROR@1..2
                R_BRACK@1..2 "]"
              MIXED_GROUP@2..3
                L_BRACK@2..3 "["
              R_CURLY@3..4 "}"

    "#]],
    );
}

#[test]
fn test_hello_world() {
    check(
        r#"Hello World!"#,
        expect![[r#"
        ROOT@0..12
          PREAMBLE@0..12
            TEXT@0..12
              WORD@0..5 "Hello"
              WHITESPACE@5..6 " "
              WORD@6..12 "World!"

    "#]],
    );
}

#[test]
fn test_biblatex_include_options() {
    check(
        r#"\addbibresource[foo=bar, baz]{foo/bar.bib}"#,
        expect![[r#"
        ROOT@0..42
          PREAMBLE@0..42
            BIBLATEX_INCLUDE@0..42
              COMMAND_NAME@0..15 "\\addbibresource"
              BRACK_GROUP_KEY_VALUE@15..29
                L_BRACK@15..16 "["
                KEY_VALUE_BODY@16..28
                  KEY_VALUE_PAIR@16..23
                    KEY@16..19
                      WORD@16..19 "foo"
                    EQUALITY_SIGN@19..20 "="
                    VALUE@20..23
                      TEXT@20..23
                        WORD@20..23 "bar"
                  COMMA@23..24 ","
                  WHITESPACE@24..25 " "
                  KEY_VALUE_PAIR@25..28
                    KEY@25..28
                      WORD@25..28 "baz"
                R_BRACK@28..29 "]"
              CURLY_GROUP_WORD_LIST@29..42
                L_CURLY@29..30 "{"
                KEY@30..41
                  WORD@30..41 "foo/bar.bib"
                R_CURLY@41..42 "}"

    "#]],
    );
}

#[test]
fn test_biblatex_include_simple() {
    check(
        r#"\addbibresource{foo/bar.bib}"#,
        expect![[r#"
        ROOT@0..28
          PREAMBLE@0..28
            BIBLATEX_INCLUDE@0..28
              COMMAND_NAME@0..15 "\\addbibresource"
              CURLY_GROUP_WORD_LIST@15..28
                L_CURLY@15..16 "{"
                KEY@16..27
                  WORD@16..27 "foo/bar.bib"
                R_CURLY@27..28 "}"

    "#]],
    );
}

#[test]
fn test_bibtex_include_simple() {
    check(
        r#"\bibliography{foo/bar}"#,
        expect![[r#"
        ROOT@0..22
          PREAMBLE@0..22
            BIBTEX_INCLUDE@0..22
              COMMAND_NAME@0..13 "\\bibliography"
              CURLY_GROUP_WORD_LIST@13..22
                L_CURLY@13..14 "{"
                KEY@14..21
                  WORD@14..21 "foo/bar"
                R_CURLY@21..22 "}"

    "#]],
    );
}

#[test]
fn test_class_include_empty() {
    check(
        r#"\documentclass{}"#,
        expect![[r#"
        ROOT@0..16
          PREAMBLE@0..16
            CLASS_INCLUDE@0..16
              COMMAND_NAME@0..14 "\\documentclass"
              CURLY_GROUP_WORD_LIST@14..16
                L_CURLY@14..15 "{"
                R_CURLY@15..16 "}"

    "#]],
    );
}

#[test]
fn test_class_include_options() {
    check(
        r#"\documentclass[foo = bar, baz, qux]{article}"#,
        expect![[r#"
        ROOT@0..44
          PREAMBLE@0..44
            CLASS_INCLUDE@0..44
              COMMAND_NAME@0..14 "\\documentclass"
              BRACK_GROUP_KEY_VALUE@14..35
                L_BRACK@14..15 "["
                KEY_VALUE_BODY@15..34
                  KEY_VALUE_PAIR@15..24
                    KEY@15..19
                      WORD@15..18 "foo"
                      WHITESPACE@18..19 " "
                    EQUALITY_SIGN@19..20 "="
                    WHITESPACE@20..21 " "
                    VALUE@21..24
                      TEXT@21..24
                        WORD@21..24 "bar"
                  COMMA@24..25 ","
                  WHITESPACE@25..26 " "
                  KEY_VALUE_PAIR@26..29
                    KEY@26..29
                      WORD@26..29 "baz"
                  COMMA@29..30 ","
                  WHITESPACE@30..31 " "
                  KEY_VALUE_PAIR@31..34
                    KEY@31..34
                      WORD@31..34 "qux"
                R_BRACK@34..35 "]"
              CURLY_GROUP_WORD_LIST@35..44
                L_CURLY@35..36 "{"
                KEY@36..43
                  WORD@36..43 "article"
                R_CURLY@43..44 "}"

    "#]],
    );
}

#[test]
fn test_class_include_simple() {
    check(
        r#"\documentclass{article}"#,
        expect![[r#"
        ROOT@0..23
          PREAMBLE@0..23
            CLASS_INCLUDE@0..23
              COMMAND_NAME@0..14 "\\documentclass"
              CURLY_GROUP_WORD_LIST@14..23
                L_CURLY@14..15 "{"
                KEY@15..22
                  WORD@15..22 "article"
                R_CURLY@22..23 "}"

    "#]],
    );
}

#[test]
fn test_graphics_include_command() {
    check(
        r#"\includegraphics[width=0.5\textwidth]{\foo.\bar.pdf}"#,
        expect![[r#"
            ROOT@0..52
              PREAMBLE@0..52
                GRAPHICS_INCLUDE@0..52
                  COMMAND_NAME@0..16 "\\includegraphics"
                  BRACK_GROUP_KEY_VALUE@16..37
                    L_BRACK@16..17 "["
                    KEY_VALUE_BODY@17..36
                      KEY_VALUE_PAIR@17..36
                        KEY@17..22
                          WORD@17..22 "width"
                        EQUALITY_SIGN@22..23 "="
                        VALUE@23..36
                          TEXT@23..26
                            WORD@23..26 "0.5"
                          GENERIC_COMMAND@26..36
                            COMMAND_NAME@26..36 "\\textwidth"
                    R_BRACK@36..37 "]"
                  CURLY_GROUP_WORD_LIST@37..52
                    L_CURLY@37..38 "{"
                    KEY@38..51
                      COMMAND_NAME@38..42 "\\foo"
                      WORD@42..43 "."
                      COMMAND_NAME@43..47 "\\bar"
                      WORD@47..51 ".pdf"
                    R_CURLY@51..52 "}"

        "#]],
    );
}

#[test]
fn test_graphics_include_complicated_options() {
    check(
        r#"\includegraphics[width=0.5\textwidth]{}"#,
        expect![[r#"
        ROOT@0..39
          PREAMBLE@0..39
            GRAPHICS_INCLUDE@0..39
              COMMAND_NAME@0..16 "\\includegraphics"
              BRACK_GROUP_KEY_VALUE@16..37
                L_BRACK@16..17 "["
                KEY_VALUE_BODY@17..36
                  KEY_VALUE_PAIR@17..36
                    KEY@17..22
                      WORD@17..22 "width"
                    EQUALITY_SIGN@22..23 "="
                    VALUE@23..36
                      TEXT@23..26
                        WORD@23..26 "0.5"
                      GENERIC_COMMAND@26..36
                        COMMAND_NAME@26..36 "\\textwidth"
                R_BRACK@36..37 "]"
              CURLY_GROUP_WORD_LIST@37..39
                L_CURLY@37..38 "{"
                R_CURLY@38..39 "}"

    "#]],
    );
}

#[test]
fn test_graphics_include_options() {
    check(
        r#"\includegraphics[scale=.5]{foo/bar.pdf}"#,
        expect![[r#"
        ROOT@0..39
          PREAMBLE@0..39
            GRAPHICS_INCLUDE@0..39
              COMMAND_NAME@0..16 "\\includegraphics"
              BRACK_GROUP_KEY_VALUE@16..26
                L_BRACK@16..17 "["
                KEY_VALUE_BODY@17..25
                  KEY_VALUE_PAIR@17..25
                    KEY@17..22
                      WORD@17..22 "scale"
                    EQUALITY_SIGN@22..23 "="
                    VALUE@23..25
                      TEXT@23..25
                        WORD@23..25 ".5"
                R_BRACK@25..26 "]"
              CURLY_GROUP_WORD_LIST@26..39
                L_CURLY@26..27 "{"
                KEY@27..38
                  WORD@27..38 "foo/bar.pdf"
                R_CURLY@38..39 "}"

    "#]],
    );
}

#[test]
fn test_graphics_include_simple() {
    check(
        r#"\includegraphics{foo/bar.pdf}"#,
        expect![[r#"
        ROOT@0..29
          PREAMBLE@0..29
            GRAPHICS_INCLUDE@0..29
              COMMAND_NAME@0..16 "\\includegraphics"
              CURLY_GROUP_WORD_LIST@16..29
                L_CURLY@16..17 "{"
                KEY@17..28
                  WORD@17..28 "foo/bar.pdf"
                R_CURLY@28..29 "}"

    "#]],
    );
}

#[test]
fn test_import_incomplete() {
    check(
        r#"\import{foo"#,
        expect![[r#"
        ROOT@0..11
          PREAMBLE@0..11
            IMPORT@0..11
              COMMAND_NAME@0..7 "\\import"
              CURLY_GROUP_WORD@7..11
                L_CURLY@7..8 "{"
                KEY@8..11
                  WORD@8..11 "foo"

    "#]],
    );
}

#[test]
fn test_import_simple() {
    check(
        r#"\import{foo}{bar}"#,
        expect![[r#"
        ROOT@0..17
          PREAMBLE@0..17
            IMPORT@0..17
              COMMAND_NAME@0..7 "\\import"
              CURLY_GROUP_WORD@7..12
                L_CURLY@7..8 "{"
                KEY@8..11
                  WORD@8..11 "foo"
                R_CURLY@11..12 "}"
              CURLY_GROUP_WORD@12..17
                L_CURLY@12..13 "{"
                KEY@13..16
                  WORD@13..16 "bar"
                R_CURLY@16..17 "}"

    "#]],
    );
}

#[test]
fn test_inkscape_include_options() {
    check(
        r#"\includesvg[scale=.5]{foo/bar}"#,
        expect![[r#"
        ROOT@0..30
          PREAMBLE@0..30
            SVG_INCLUDE@0..30
              COMMAND_NAME@0..11 "\\includesvg"
              BRACK_GROUP_KEY_VALUE@11..21
                L_BRACK@11..12 "["
                KEY_VALUE_BODY@12..20
                  KEY_VALUE_PAIR@12..20
                    KEY@12..17
                      WORD@12..17 "scale"
                    EQUALITY_SIGN@17..18 "="
                    VALUE@18..20
                      TEXT@18..20
                        WORD@18..20 ".5"
                R_BRACK@20..21 "]"
              CURLY_GROUP_WORD_LIST@21..30
                L_CURLY@21..22 "{"
                KEY@22..29
                  WORD@22..29 "foo/bar"
                R_CURLY@29..30 "}"

    "#]],
    );
}

#[test]
fn test_inkscape_include_simple() {
    check(
        r#"\includesvg{foo/bar}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            SVG_INCLUDE@0..20
              COMMAND_NAME@0..11 "\\includesvg"
              CURLY_GROUP_WORD_LIST@11..20
                L_CURLY@11..12 "{"
                KEY@12..19
                  WORD@12..19 "foo/bar"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_latex_include_equality_sign() {
    check(
        r#"\include{foo=bar}"#,
        expect![[r#"
        ROOT@0..17
          PREAMBLE@0..17
            LATEX_INCLUDE@0..17
              COMMAND_NAME@0..8 "\\include"
              CURLY_GROUP_WORD_LIST@8..17
                L_CURLY@8..9 "{"
                KEY@9..16
                  WORD@9..12 "foo"
                  EQUALITY_SIGN@12..13 "="
                  WORD@13..16 "bar"
                R_CURLY@16..17 "}"

    "#]],
    );
}

#[test]
fn test_latex_include_simple() {
    check(
        r#"\include{foo/bar}"#,
        expect![[r#"
        ROOT@0..17
          PREAMBLE@0..17
            LATEX_INCLUDE@0..17
              COMMAND_NAME@0..8 "\\include"
              CURLY_GROUP_WORD_LIST@8..17
                L_CURLY@8..9 "{"
                KEY@9..16
                  WORD@9..16 "foo/bar"
                R_CURLY@16..17 "}"

    "#]],
    );
}

#[test]
fn test_latex_input_path_brackets() {
    check(
        r#"\input{foo[bar].tex}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            LATEX_INCLUDE@0..20
              COMMAND_NAME@0..6 "\\input"
              CURLY_GROUP_WORD_LIST@6..20
                L_CURLY@6..7 "{"
                KEY@7..19
                  WORD@7..10 "foo"
                  L_BRACK@10..11 "["
                  WORD@11..14 "bar"
                  R_BRACK@14..15 "]"
                  WORD@15..19 ".tex"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_package_include_empty() {
    check(
        r#"\usepackage{}"#,
        expect![[r#"
        ROOT@0..13
          PREAMBLE@0..13
            PACKAGE_INCLUDE@0..13
              COMMAND_NAME@0..11 "\\usepackage"
              CURLY_GROUP_WORD_LIST@11..13
                L_CURLY@11..12 "{"
                R_CURLY@12..13 "}"

    "#]],
    );
}

#[test]
fn test_package_include_multiple() {
    check(
        r#"\usepackage{amsmath, lipsum}"#,
        expect![[r#"
        ROOT@0..28
          PREAMBLE@0..28
            PACKAGE_INCLUDE@0..28
              COMMAND_NAME@0..11 "\\usepackage"
              CURLY_GROUP_WORD_LIST@11..28
                L_CURLY@11..12 "{"
                KEY@12..19
                  WORD@12..19 "amsmath"
                COMMA@19..20 ","
                WHITESPACE@20..21 " "
                KEY@21..27
                  WORD@21..27 "lipsum"
                R_CURLY@27..28 "}"

    "#]],
    );
}

#[test]
fn test_package_include_options() {
    check(
        r#"\usepackage[foo = bar, baz, qux]{amsmath}"#,
        expect![[r#"
        ROOT@0..41
          PREAMBLE@0..41
            PACKAGE_INCLUDE@0..41
              COMMAND_NAME@0..11 "\\usepackage"
              BRACK_GROUP_KEY_VALUE@11..32
                L_BRACK@11..12 "["
                KEY_VALUE_BODY@12..31
                  KEY_VALUE_PAIR@12..21
                    KEY@12..16
                      WORD@12..15 "foo"
                      WHITESPACE@15..16 " "
                    EQUALITY_SIGN@16..17 "="
                    WHITESPACE@17..18 " "
                    VALUE@18..21
                      TEXT@18..21
                        WORD@18..21 "bar"
                  COMMA@21..22 ","
                  WHITESPACE@22..23 " "
                  KEY_VALUE_PAIR@23..26
                    KEY@23..26
                      WORD@23..26 "baz"
                  COMMA@26..27 ","
                  WHITESPACE@27..28 " "
                  KEY_VALUE_PAIR@28..31
                    KEY@28..31
                      WORD@28..31 "qux"
                R_BRACK@31..32 "]"
              CURLY_GROUP_WORD_LIST@32..41
                L_CURLY@32..33 "{"
                KEY@33..40
                  WORD@33..40 "amsmath"
                R_CURLY@40..41 "}"

    "#]],
    );
}

#[test]
fn test_package_include_simple() {
    check(
        r#"\usepackage{amsmath}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            PACKAGE_INCLUDE@0..20
              COMMAND_NAME@0..11 "\\usepackage"
              CURLY_GROUP_WORD_LIST@11..20
                L_CURLY@11..12 "{"
                KEY@12..19
                  WORD@12..19 "amsmath"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_pgf_library_import_simple() {
    check(
        r#"\usepgflibrary{foo}"#,
        expect![[r#"
        ROOT@0..19
          PREAMBLE@0..19
            TIKZ_LIBRARY_IMPORT@0..19
              COMMAND_NAME@0..14 "\\usepgflibrary"
              CURLY_GROUP_WORD_LIST@14..19
                L_CURLY@14..15 "{"
                KEY@15..18
                  WORD@15..18 "foo"
                R_CURLY@18..19 "}"

    "#]],
    );
}

#[test]
fn test_svg_include_options() {
    check(
        r#"\includesvg[scale=.5]{foo/bar.svg}"#,
        expect![[r#"
        ROOT@0..34
          PREAMBLE@0..34
            SVG_INCLUDE@0..34
              COMMAND_NAME@0..11 "\\includesvg"
              BRACK_GROUP_KEY_VALUE@11..21
                L_BRACK@11..12 "["
                KEY_VALUE_BODY@12..20
                  KEY_VALUE_PAIR@12..20
                    KEY@12..17
                      WORD@12..17 "scale"
                    EQUALITY_SIGN@17..18 "="
                    VALUE@18..20
                      TEXT@18..20
                        WORD@18..20 ".5"
                R_BRACK@20..21 "]"
              CURLY_GROUP_WORD_LIST@21..34
                L_CURLY@21..22 "{"
                KEY@22..33
                  WORD@22..33 "foo/bar.svg"
                R_CURLY@33..34 "}"

    "#]],
    );
}

#[test]
fn test_svg_include_simple() {
    check(
        r#"\includesvg{foo/bar.svg}"#,
        expect![[r#"
        ROOT@0..24
          PREAMBLE@0..24
            SVG_INCLUDE@0..24
              COMMAND_NAME@0..11 "\\includesvg"
              CURLY_GROUP_WORD_LIST@11..24
                L_CURLY@11..12 "{"
                KEY@12..23
                  WORD@12..23 "foo/bar.svg"
                R_CURLY@23..24 "}"

    "#]],
    );
}

#[test]
fn test_tikz_library_import_simple() {
    check(
        r#"\usetikzlibrary{foo}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            TIKZ_LIBRARY_IMPORT@0..20
              COMMAND_NAME@0..15 "\\usetikzlibrary"
              CURLY_GROUP_WORD_LIST@15..20
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "foo"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_verbatim_include_simple() {
    check(
        r#"\verbatiminput{foo/bar.txt}"#,
        expect![[r#"
        ROOT@0..27
          PREAMBLE@0..27
            VERBATIM_INCLUDE@0..27
              COMMAND_NAME@0..14 "\\verbatiminput"
              CURLY_GROUP_WORD_LIST@14..27
                L_CURLY@14..15 "{"
                KEY@15..26
                  WORD@15..26 "foo/bar.txt"
                R_CURLY@26..27 "}"

    "#]],
    );
}

#[test]
fn test_inline() {
    check(
        r#"$x \in [0, \infty)$"#,
        expect![[r#"
        ROOT@0..19
          PREAMBLE@0..19
            FORMULA@0..19
              DOLLAR@0..1 "$"
              TEXT@1..3
                WORD@1..2 "x"
                WHITESPACE@2..3 " "
              GENERIC_COMMAND@3..18
                COMMAND_NAME@3..6 "\\in"
                WHITESPACE@6..7 " "
                MIXED_GROUP@7..18
                  L_BRACK@7..8 "["
                  TEXT@8..11
                    WORD@8..9 "0"
                    COMMA@9..10 ","
                    WHITESPACE@10..11 " "
                  GENERIC_COMMAND@11..17
                    COMMAND_NAME@11..17 "\\infty"
                  R_PAREN@17..18 ")"
              DOLLAR@18..19 "$"

    "#]],
    );
}

#[test]
fn test_inline_double_dollar() {
    check(
        r#"$$x \in [0, \infty)$$"#,
        expect![[r#"
        ROOT@0..21
          PREAMBLE@0..21
            FORMULA@0..21
              DOLLAR@0..2 "$$"
              TEXT@2..4
                WORD@2..3 "x"
                WHITESPACE@3..4 " "
              GENERIC_COMMAND@4..19
                COMMAND_NAME@4..7 "\\in"
                WHITESPACE@7..8 " "
                MIXED_GROUP@8..19
                  L_BRACK@8..9 "["
                  TEXT@9..12
                    WORD@9..10 "0"
                    COMMA@10..11 ","
                    WHITESPACE@11..12 " "
                  GENERIC_COMMAND@12..18
                    COMMAND_NAME@12..18 "\\infty"
                  R_PAREN@18..19 ")"
              DOLLAR@19..21 "$$"

    "#]],
    );
}

#[test]
fn test_issue_568() {
    check(
        r#"\input{|ipython scripts/test.ipynb}
\label{fig:x=2}"#,
        expect![[r#"
            ROOT@0..51
              PREAMBLE@0..51
                LATEX_INCLUDE@0..36
                  COMMAND_NAME@0..6 "\\input"
                  CURLY_GROUP_WORD_LIST@6..36
                    L_CURLY@6..7 "{"
                    WORD@7..8 "|"
                    KEY@8..34
                      WORD@8..15 "ipython"
                      WHITESPACE@15..16 " "
                      WORD@16..34 "scripts/test.ipynb"
                    R_CURLY@34..35 "}"
                    LINE_BREAK@35..36 "\n"
                LABEL_DEFINITION@36..51
                  COMMAND_NAME@36..42 "\\label"
                  CURLY_GROUP_WORD@42..51
                    L_CURLY@42..43 "{"
                    KEY@43..50
                      WORD@43..48 "fig:x"
                      EQUALITY_SIGN@48..49 "="
                      WORD@49..50 "2"
                    R_CURLY@50..51 "}"

        "#]],
    );
}

#[test]
fn test_issue_745() {
    check(
        r#"\documentclass{article}
\usepackage{tabularray} 

\ExplSyntaxOn
\NewDocumentEnvironment{exptblr}{O{}m}
    {
    \use:x
    {
    \exp_not:N \begin{tblr}
    [\exp_not:n{#1}]
    {#2}
    }
    }
    {
    \end{tblr}
    }
\ExplSyntaxOff

\begin{document}

\end{document}"#,
        expect![[r##"
            ROOT@0..271
              PREAMBLE@0..271
                CLASS_INCLUDE@0..24
                  COMMAND_NAME@0..14 "\\documentclass"
                  CURLY_GROUP_WORD_LIST@14..24
                    L_CURLY@14..15 "{"
                    KEY@15..22
                      WORD@15..22 "article"
                    R_CURLY@22..23 "}"
                    LINE_BREAK@23..24 "\n"
                PACKAGE_INCLUDE@24..50
                  COMMAND_NAME@24..35 "\\usepackage"
                  CURLY_GROUP_WORD_LIST@35..50
                    L_CURLY@35..36 "{"
                    KEY@36..46
                      WORD@36..46 "tabularray"
                    R_CURLY@46..47 "}"
                    WHITESPACE@47..48 " "
                    LINE_BREAK@48..50 "\n\n"
                GENERIC_COMMAND@50..64
                  COMMAND_NAME@50..63 "\\ExplSyntaxOn"
                  LINE_BREAK@63..64 "\n"
                GENERIC_COMMAND@64..223
                  COMMAND_NAME@64..87 "\\NewDocumentEnvironment"
                  CURLY_GROUP@87..96
                    L_CURLY@87..88 "{"
                    TEXT@88..95
                      WORD@88..95 "exptblr"
                    R_CURLY@95..96 "}"
                  CURLY_GROUP@96..107
                    L_CURLY@96..97 "{"
                    TEXT@97..98
                      WORD@97..98 "O"
                    CURLY_GROUP@98..100
                      L_CURLY@98..99 "{"
                      R_CURLY@99..100 "}"
                    TEXT@100..101
                      WORD@100..101 "m"
                    R_CURLY@101..102 "}"
                    LINE_BREAK@102..103 "\n"
                    WHITESPACE@103..107 "    "
                  CURLY_GROUP@107..200
                    L_CURLY@107..108 "{"
                    LINE_BREAK@108..109 "\n"
                    WHITESPACE@109..113 "    "
                    GENERIC_COMMAND@113..194
                      COMMAND_NAME@113..119 "\\use:x"
                      LINE_BREAK@119..120 "\n"
                      WHITESPACE@120..124 "    "
                      CURLY_GROUP@124..194
                        L_CURLY@124..125 "{"
                        LINE_BREAK@125..126 "\n"
                        WHITESPACE@126..130 "    "
                        GENERIC_COMMAND@130..141
                          COMMAND_NAME@130..140 "\\exp_not:N"
                          WHITESPACE@140..141 " "
                        ENVIRONMENT@141..188
                          BEGIN@141..179
                            COMMAND_NAME@141..147 "\\begin"
                            CURLY_GROUP_WORD@147..158
                              L_CURLY@147..148 "{"
                              KEY@148..152
                                WORD@148..152 "tblr"
                              R_CURLY@152..153 "}"
                              LINE_BREAK@153..154 "\n"
                              WHITESPACE@154..158 "    "
                            BRACK_GROUP@158..179
                              L_BRACK@158..159 "["
                              GENERIC_COMMAND@159..173
                                COMMAND_NAME@159..169 "\\exp_not:n"
                                CURLY_GROUP@169..173
                                  L_CURLY@169..170 "{"
                                  TEXT@170..172
                                    WORD@170..172 "#1"
                                  R_CURLY@172..173 "}"
                              R_BRACK@173..174 "]"
                              LINE_BREAK@174..175 "\n"
                              WHITESPACE@175..179 "    "
                          CURLY_GROUP@179..188
                            L_CURLY@179..180 "{"
                            TEXT@180..182
                              WORD@180..182 "#2"
                            R_CURLY@182..183 "}"
                            LINE_BREAK@183..184 "\n"
                            WHITESPACE@184..188 "    "
                        R_CURLY@188..189 "}"
                        LINE_BREAK@189..190 "\n"
                        WHITESPACE@190..194 "    "
                    R_CURLY@194..195 "}"
                    LINE_BREAK@195..196 "\n"
                    WHITESPACE@196..200 "    "
                  CURLY_GROUP@200..223
                    L_CURLY@200..201 "{"
                    LINE_BREAK@201..202 "\n"
                    WHITESPACE@202..206 "    "
                    GENERIC_COMMAND@206..221
                      COMMAND_NAME@206..210 "\\end"
                      CURLY_GROUP@210..221
                        L_CURLY@210..211 "{"
                        TEXT@211..215
                          WORD@211..215 "tblr"
                        R_CURLY@215..216 "}"
                        LINE_BREAK@216..217 "\n"
                        WHITESPACE@217..221 "    "
                    R_CURLY@221..222 "}"
                    LINE_BREAK@222..223 "\n"
                GENERIC_COMMAND@223..239
                  COMMAND_NAME@223..237 "\\ExplSyntaxOff"
                  LINE_BREAK@237..239 "\n\n"
                ENVIRONMENT@239..271
                  BEGIN@239..257
                    COMMAND_NAME@239..245 "\\begin"
                    CURLY_GROUP_WORD@245..257
                      L_CURLY@245..246 "{"
                      KEY@246..254
                        WORD@246..254 "document"
                      R_CURLY@254..255 "}"
                      LINE_BREAK@255..257 "\n\n"
                  END@257..271
                    COMMAND_NAME@257..261 "\\end"
                    CURLY_GROUP_WORD@261..271
                      L_CURLY@261..262 "{"
                      KEY@262..270
                        WORD@262..270 "document"
                      R_CURLY@270..271 "}"

        "##]],
    );
}

#[test]
fn test_issue_789() {
    check(
        r#"\graphicspath{test}"#,
        expect![[r#"
        ROOT@0..19
          PREAMBLE@0..19
            GRAPHICS_PATH@0..19
              COMMAND_NAME@0..13 "\\graphicspath"
              CURLY_GROUP_WORD@13..19
                L_CURLY@13..14 "{"
                KEY@14..18
                  WORD@14..18 "test"
                R_CURLY@18..19 "}"

    "#]],
    );
}

#[test]
fn test_issue_828() {
    check(
        r#"\verb|<STATEMENT>     if(<expr>){<body>else{<body>|"#,
        expect![[r#"
            ROOT@0..51
              PREAMBLE@0..51
                GENERIC_COMMAND@0..5
                  COMMAND_NAME@0..5 "\\verb"
                VERBATIM@5..6 "|"
                VERBATIM@6..17 "<STATEMENT>"
                VERBATIM@17..22 "     "
                VERBATIM@22..24 "if"
                VERBATIM@24..25 "("
                VERBATIM@25..31 "<expr>"
                VERBATIM@31..32 ")"
                VERBATIM@32..33 "{"
                VERBATIM@33..43 "<body>else"
                VERBATIM@43..44 "{"
                VERBATIM@44..50 "<body>"
                VERBATIM@50..51 "|"

        "#]],
    );
}

#[test]
fn test_issue_853() {
    check(
        r#"\documentclass{minimal}
\begin{document}
This is an asdf undefined command
\iffalse
  \iffalse\fi
  \end{enumerate} 
\fi
\end{document}"#,
        expect![[r#"
            ROOT@0..135
              PREAMBLE@0..135
                CLASS_INCLUDE@0..24
                  COMMAND_NAME@0..14 "\\documentclass"
                  CURLY_GROUP_WORD_LIST@14..24
                    L_CURLY@14..15 "{"
                    KEY@15..22
                      WORD@15..22 "minimal"
                    R_CURLY@22..23 "}"
                    LINE_BREAK@23..24 "\n"
                ENVIRONMENT@24..135
                  BEGIN@24..41
                    COMMAND_NAME@24..30 "\\begin"
                    CURLY_GROUP_WORD@30..41
                      L_CURLY@30..31 "{"
                      KEY@31..39
                        WORD@31..39 "document"
                      R_CURLY@39..40 "}"
                      LINE_BREAK@40..41 "\n"
                  TEXT@41..75
                    WORD@41..45 "This"
                    WHITESPACE@45..46 " "
                    WORD@46..48 "is"
                    WHITESPACE@48..49 " "
                    WORD@49..51 "an"
                    WHITESPACE@51..52 " "
                    WORD@52..56 "asdf"
                    WHITESPACE@56..57 " "
                    WORD@57..66 "undefined"
                    WHITESPACE@66..67 " "
                    WORD@67..74 "command"
                    LINE_BREAK@74..75 "\n"
                  BLOCK_COMMENT@75..120
                    COMMAND_NAME@75..83 "\\iffalse"
                    LINE_BREAK@83..84 "\n"
                    WHITESPACE@84..86 "  "
                    BLOCK_COMMENT@86..97
                      COMMAND_NAME@86..94 "\\iffalse"
                      COMMAND_NAME@94..97 "\\fi"
                    LINE_BREAK@97..98 "\n"
                    WHITESPACE@98..100 "  "
                    COMMAND_NAME@100..104 "\\end"
                    L_CURLY@104..105 "{"
                    WORD@105..114 "enumerate"
                    R_CURLY@114..115 "}"
                    WHITESPACE@115..116 " "
                    LINE_BREAK@116..117 "\n"
                    COMMAND_NAME@117..120 "\\fi"
                  LINE_BREAK@120..121 "\n"
                  END@121..135
                    COMMAND_NAME@121..125 "\\end"
                    CURLY_GROUP_WORD@125..135
                      L_CURLY@125..126 "{"
                      KEY@126..134
                        WORD@126..134 "document"
                      R_CURLY@134..135 "}"

        "#]],
    );
}

#[test]
fn test_issue_857() {
    check(
        r#"\newcommand\ö{}
\newcommand{\öö}{}
\newcommand\123{}"#,
        expect![[r#"
            ROOT@0..55
              PREAMBLE@0..55
                COMMAND_DEFINITION@0..11
                  COMMAND_NAME@0..11 "\\newcommand"
                GENERIC_COMMAND@11..17
                  COMMAND_NAME@11..14 "\\ö"
                  CURLY_GROUP@14..17
                    L_CURLY@14..15 "{"
                    R_CURLY@15..16 "}"
                    LINE_BREAK@16..17 "\n"
                COMMAND_DEFINITION@17..38
                  COMMAND_NAME@17..28 "\\newcommand"
                  CURLY_GROUP_COMMAND@28..35
                    L_CURLY@28..29 "{"
                    COMMAND_NAME@29..34 "\\öö"
                    R_CURLY@34..35 "}"
                  CURLY_GROUP@35..38
                    L_CURLY@35..36 "{"
                    R_CURLY@36..37 "}"
                    LINE_BREAK@37..38 "\n"
                COMMAND_DEFINITION@38..49
                  COMMAND_NAME@38..49 "\\newcommand"
                GENERIC_COMMAND@49..55
                  COMMAND_NAME@49..53 "\\123"
                  CURLY_GROUP@53..55
                    L_CURLY@53..54 "{"
                    R_CURLY@54..55 "}"

        "#]],
    );
}

#[test]
fn test_issue_874() {
    check(
        r#"\includegraphics[scale=0.2]{7.4).jpg}"#,
        expect![[r#"
        ROOT@0..37
          PREAMBLE@0..37
            GRAPHICS_INCLUDE@0..37
              COMMAND_NAME@0..16 "\\includegraphics"
              BRACK_GROUP_KEY_VALUE@16..27
                L_BRACK@16..17 "["
                KEY_VALUE_BODY@17..26
                  KEY_VALUE_PAIR@17..26
                    KEY@17..22
                      WORD@17..22 "scale"
                    EQUALITY_SIGN@22..23 "="
                    VALUE@23..26
                      TEXT@23..26
                        WORD@23..26 "0.2"
                R_BRACK@26..27 "]"
              CURLY_GROUP_WORD_LIST@27..37
                L_CURLY@27..28 "{"
                KEY@28..36
                  WORD@28..31 "7.4"
                  R_PAREN@31..32 ")"
                  WORD@32..36 ".jpg"
                R_CURLY@36..37 "}"

    "#]],
    );
}

#[test]
fn test_issue_919() {
    check(
        r#"\documentclass{article}

\usepackage{
    lipsum, % provides blindtext
    booktabs, % better rules for tables
    %xcolor % easily define colors with \definecolor{}{}{}
}

\begin{document}
    \lipsum
\end{document}
"#,
        expect![[r#"
            ROOT@0..217
              PREAMBLE@0..217
                CLASS_INCLUDE@0..25
                  COMMAND_NAME@0..14 "\\documentclass"
                  CURLY_GROUP_WORD_LIST@14..25
                    L_CURLY@14..15 "{"
                    KEY@15..22
                      WORD@15..22 "article"
                    R_CURLY@22..23 "}"
                    LINE_BREAK@23..25 "\n\n"
                PACKAGE_INCLUDE@25..173
                  COMMAND_NAME@25..36 "\\usepackage"
                  CURLY_GROUP_WORD_LIST@36..173
                    L_CURLY@36..37 "{"
                    LINE_BREAK@37..38 "\n"
                    WHITESPACE@38..42 "    "
                    KEY@42..48
                      WORD@42..48 "lipsum"
                    COMMA@48..49 ","
                    WHITESPACE@49..50 " "
                    COMMENT@50..70 "% provides blindtext"
                    LINE_BREAK@70..71 "\n"
                    WHITESPACE@71..75 "    "
                    KEY@75..83
                      WORD@75..83 "booktabs"
                    COMMA@83..84 ","
                    WHITESPACE@84..85 " "
                    COMMENT@85..110 "% better rules for ta ..."
                    LINE_BREAK@110..111 "\n"
                    WHITESPACE@111..115 "    "
                    COMMENT@115..169 "%xcolor % easily defi ..."
                    LINE_BREAK@169..170 "\n"
                    R_CURLY@170..171 "}"
                    LINE_BREAK@171..173 "\n\n"
                ENVIRONMENT@173..217
                  BEGIN@173..194
                    COMMAND_NAME@173..179 "\\begin"
                    CURLY_GROUP_WORD@179..194
                      L_CURLY@179..180 "{"
                      KEY@180..188
                        WORD@180..188 "document"
                      R_CURLY@188..189 "}"
                      LINE_BREAK@189..190 "\n"
                      WHITESPACE@190..194 "    "
                  GENERIC_COMMAND@194..202
                    COMMAND_NAME@194..201 "\\lipsum"
                    LINE_BREAK@201..202 "\n"
                  END@202..217
                    COMMAND_NAME@202..206 "\\end"
                    CURLY_GROUP_WORD@206..217
                      L_CURLY@206..207 "{"
                      KEY@207..215
                        WORD@207..215 "document"
                      R_CURLY@215..216 "}"
                      LINE_BREAK@216..217 "\n"

        "#]],
    );
}

#[test]
fn test_issue_931() {
    check(
        r#"\bibliography{$HOME/Literature}"#,
        expect![[r#"
        ROOT@0..31
          PREAMBLE@0..31
            BIBTEX_INCLUDE@0..31
              COMMAND_NAME@0..13 "\\bibliography"
              CURLY_GROUP_WORD_LIST@13..31
                L_CURLY@13..14 "{"
                KEY@14..30
                  DOLLAR@14..15 "$"
                  WORD@15..30 "HOME/Literature"
                R_CURLY@30..31 "}"

    "#]],
    );
}

#[test]
fn test_label_definition_line_break() {
    check(
        r#"\label{hello
world}"#,
        expect![[r#"
            ROOT@0..19
              PREAMBLE@0..19
                LABEL_DEFINITION@0..13
                  COMMAND_NAME@0..6 "\\label"
                  CURLY_GROUP_WORD@6..13
                    L_CURLY@6..7 "{"
                    KEY@7..13
                      WORD@7..12 "hello"
                      LINE_BREAK@12..13 "\n"
                TEXT@13..18
                  WORD@13..18 "world"
                ERROR@18..19
                  R_CURLY@18..19 "}"

        "#]],
    );
}

#[test]
fn test_label_definition_simple() {
    check(
        r#"\label{foo}"#,
        expect![[r#"
        ROOT@0..11
          PREAMBLE@0..11
            LABEL_DEFINITION@0..11
              COMMAND_NAME@0..6 "\\label"
              CURLY_GROUP_WORD@6..11
                L_CURLY@6..7 "{"
                KEY@7..10
                  WORD@7..10 "foo"
                R_CURLY@10..11 "}"

    "#]],
    );
}

#[test]
fn test_label_number() {
    check(
        r#"\newlabel{foo}{{1.1}}"#,
        expect![[r#"
        ROOT@0..21
          PREAMBLE@0..21
            LABEL_NUMBER@0..21
              COMMAND_NAME@0..9 "\\newlabel"
              CURLY_GROUP_WORD@9..14
                L_CURLY@9..10 "{"
                KEY@10..13
                  WORD@10..13 "foo"
                R_CURLY@13..14 "}"
              CURLY_GROUP@14..21
                L_CURLY@14..15 "{"
                CURLY_GROUP@15..20
                  L_CURLY@15..16 "{"
                  TEXT@16..19
                    WORD@16..19 "1.1"
                  R_CURLY@19..20 "}"
                R_CURLY@20..21 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_equation() {
    check(
        r#"\eqref{foo}"#,
        expect![[r#"
        ROOT@0..11
          PREAMBLE@0..11
            LABEL_REFERENCE@0..11
              COMMAND_NAME@0..6 "\\eqref"
              CURLY_GROUP_WORD_LIST@6..11
                L_CURLY@6..7 "{"
                KEY@7..10
                  WORD@7..10 "foo"
                R_CURLY@10..11 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_incomplete() {
    check(
        r#"Equation \eqref{eq is a \emph{useful} identity."#,
        expect![[r#"
            ROOT@0..47
              PREAMBLE@0..47
                TEXT@0..9
                  WORD@0..8 "Equation"
                  WHITESPACE@8..9 " "
                LABEL_REFERENCE@9..24
                  COMMAND_NAME@9..15 "\\eqref"
                  CURLY_GROUP_WORD_LIST@15..24
                    L_CURLY@15..16 "{"
                    KEY@16..24
                      WORD@16..18 "eq"
                      WHITESPACE@18..19 " "
                      WORD@19..21 "is"
                      WHITESPACE@21..22 " "
                      WORD@22..23 "a"
                      WHITESPACE@23..24 " "
                GENERIC_COMMAND@24..38
                  COMMAND_NAME@24..29 "\\emph"
                  CURLY_GROUP@29..38
                    L_CURLY@29..30 "{"
                    TEXT@30..36
                      WORD@30..36 "useful"
                    R_CURLY@36..37 "}"
                    WHITESPACE@37..38 " "
                TEXT@38..47
                  WORD@38..47 "identity."

        "#]],
    );
}

#[test]
fn test_label_reference_multiple() {
    check(
        r#"\ref{foo, bar}"#,
        expect![[r#"
        ROOT@0..14
          PREAMBLE@0..14
            LABEL_REFERENCE@0..14
              COMMAND_NAME@0..4 "\\ref"
              CURLY_GROUP_WORD_LIST@4..14
                L_CURLY@4..5 "{"
                KEY@5..8
                  WORD@5..8 "foo"
                COMMA@8..9 ","
                WHITESPACE@9..10 " "
                KEY@10..13
                  WORD@10..13 "bar"
                R_CURLY@13..14 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_range_error() {
    check(
        r#"\crefrange{foo{bar}"#,
        expect![[r#"
        ROOT@0..19
          PREAMBLE@0..19
            LABEL_REFERENCE_RANGE@0..19
              COMMAND_NAME@0..10 "\\crefrange"
              CURLY_GROUP_WORD@10..14
                L_CURLY@10..11 "{"
                KEY@11..14
                  WORD@11..14 "foo"
              CURLY_GROUP_WORD@14..19
                L_CURLY@14..15 "{"
                KEY@15..18
                  WORD@15..18 "bar"
                R_CURLY@18..19 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_range_incomplete() {
    check(
        r#"\crefrange{foo}"#,
        expect![[r#"
        ROOT@0..15
          PREAMBLE@0..15
            LABEL_REFERENCE_RANGE@0..15
              COMMAND_NAME@0..10 "\\crefrange"
              CURLY_GROUP_WORD@10..15
                L_CURLY@10..11 "{"
                KEY@11..14
                  WORD@11..14 "foo"
                R_CURLY@14..15 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_range_simple() {
    check(
        r#"\crefrange{foo}{bar}"#,
        expect![[r#"
        ROOT@0..20
          PREAMBLE@0..20
            LABEL_REFERENCE_RANGE@0..20
              COMMAND_NAME@0..10 "\\crefrange"
              CURLY_GROUP_WORD@10..15
                L_CURLY@10..11 "{"
                KEY@11..14
                  WORD@11..14 "foo"
                R_CURLY@14..15 "}"
              CURLY_GROUP_WORD@15..20
                L_CURLY@15..16 "{"
                KEY@16..19
                  WORD@16..19 "bar"
                R_CURLY@19..20 "}"

    "#]],
    );
}

#[test]
fn test_label_reference_simple() {
    check(
        r#"\ref{foo}"#,
        expect![[r#"
        ROOT@0..9
          PREAMBLE@0..9
            LABEL_REFERENCE@0..9
              COMMAND_NAME@0..4 "\\ref"
              CURLY_GROUP_WORD_LIST@4..9
                L_CURLY@4..5 "{"
                KEY@5..8
                  WORD@5..8 "foo"
                R_CURLY@8..9 "}"

    "#]],
    );
}

#[test]
fn test_parameter() {
    check(
        r#"#1"#,
        expect![[r##"
        ROOT@0..2
          PREAMBLE@0..2
            TEXT@0..2
              WORD@0..2 "#1"

    "##]],
    );
}

#[test]
fn test_parameter_error() {
    check(
        r#"#"#,
        expect![[r##"
        ROOT@0..1
          PREAMBLE@0..1
            TEXT@0..1
              WORD@0..1 "#"

    "##]],
    );
}

#[test]
fn test_paragraphs() {
    check(
        r#"\section{Section 1}
Section 1

\paragraph{Paragraph 1}
Paragraph 1

\paragraph{Paragraph 2}
Paragraph 2

\section{Section 2}
Section 2"#,
        expect![[r#"
            ROOT@0..134
              PREAMBLE@0..134
                SECTION@0..105
                  COMMAND_NAME@0..8 "\\section"
                  CURLY_GROUP@8..20
                    L_CURLY@8..9 "{"
                    TEXT@9..18
                      WORD@9..16 "Section"
                      WHITESPACE@16..17 " "
                      WORD@17..18 "1"
                    R_CURLY@18..19 "}"
                    LINE_BREAK@19..20 "\n"
                  TEXT@20..31
                    WORD@20..27 "Section"
                    WHITESPACE@27..28 " "
                    WORD@28..29 "1"
                    LINE_BREAK@29..31 "\n\n"
                  PARAGRAPH@31..68
                    COMMAND_NAME@31..41 "\\paragraph"
                    CURLY_GROUP@41..55
                      L_CURLY@41..42 "{"
                      TEXT@42..53
                        WORD@42..51 "Paragraph"
                        WHITESPACE@51..52 " "
                        WORD@52..53 "1"
                      R_CURLY@53..54 "}"
                      LINE_BREAK@54..55 "\n"
                    TEXT@55..68
                      WORD@55..64 "Paragraph"
                      WHITESPACE@64..65 " "
                      WORD@65..66 "1"
                      LINE_BREAK@66..68 "\n\n"
                  PARAGRAPH@68..105
                    COMMAND_NAME@68..78 "\\paragraph"
                    CURLY_GROUP@78..92
                      L_CURLY@78..79 "{"
                      TEXT@79..90
                        WORD@79..88 "Paragraph"
                        WHITESPACE@88..89 " "
                        WORD@89..90 "2"
                      R_CURLY@90..91 "}"
                      LINE_BREAK@91..92 "\n"
                    TEXT@92..105
                      WORD@92..101 "Paragraph"
                      WHITESPACE@101..102 " "
                      WORD@102..103 "2"
                      LINE_BREAK@103..105 "\n\n"
                SECTION@105..134
                  COMMAND_NAME@105..113 "\\section"
                  CURLY_GROUP@113..125
                    L_CURLY@113..114 "{"
                    TEXT@114..123
                      WORD@114..121 "Section"
                      WHITESPACE@121..122 " "
                      WORD@122..123 "2"
                    R_CURLY@123..124 "}"
                    LINE_BREAK@124..125 "\n"
                  TEXT@125..134
                    WORD@125..132 "Section"
                    WHITESPACE@132..133 " "
                    WORD@133..134 "2"

        "#]],
    );
}

#[test]
fn test_structure_enum_item() {
    check(
        r#"\begin{enumerate} \item 1 \item[2] 2 \item 3 \end{enumerate}"#,
        expect![[r#"
            ROOT@0..60
              PREAMBLE@0..60
                ENVIRONMENT@0..60
                  BEGIN@0..18
                    COMMAND_NAME@0..6 "\\begin"
                    CURLY_GROUP_WORD@6..18
                      L_CURLY@6..7 "{"
                      KEY@7..16
                        WORD@7..16 "enumerate"
                      R_CURLY@16..17 "}"
                      WHITESPACE@17..18 " "
                  ENUM_ITEM@18..26
                    COMMAND_NAME@18..23 "\\item"
                    WHITESPACE@23..24 " "
                    TEXT@24..26
                      WORD@24..25 "1"
                      WHITESPACE@25..26 " "
                  ENUM_ITEM@26..37
                    COMMAND_NAME@26..31 "\\item"
                    BRACK_GROUP@31..35
                      L_BRACK@31..32 "["
                      TEXT@32..33
                        WORD@32..33 "2"
                      R_BRACK@33..34 "]"
                      WHITESPACE@34..35 " "
                    TEXT@35..37
                      WORD@35..36 "2"
                      WHITESPACE@36..37 " "
                  ENUM_ITEM@37..45
                    COMMAND_NAME@37..42 "\\item"
                    WHITESPACE@42..43 " "
                    TEXT@43..45
                      WORD@43..44 "3"
                      WHITESPACE@44..45 " "
                  END@45..60
                    COMMAND_NAME@45..49 "\\end"
                    CURLY_GROUP_WORD@49..60
                      L_CURLY@49..50 "{"
                      KEY@50..59
                        WORD@50..59 "enumerate"
                      R_CURLY@59..60 "}"

        "#]],
    );
}

#[test]
fn test_structure_invalid_nesting() {
    check(
        r#"\section{Foo} \chapter{Bar}"#,
        expect![[r#"
        ROOT@0..27
          PREAMBLE@0..27
            SECTION@0..14
              COMMAND_NAME@0..8 "\\section"
              CURLY_GROUP@8..14
                L_CURLY@8..9 "{"
                TEXT@9..12
                  WORD@9..12 "Foo"
                R_CURLY@12..13 "}"
                WHITESPACE@13..14 " "
            CHAPTER@14..27
              COMMAND_NAME@14..22 "\\chapter"
              CURLY_GROUP@22..27
                L_CURLY@22..23 "{"
                TEXT@23..26
                  WORD@23..26 "Bar"
                R_CURLY@26..27 "}"

    "#]],
    );
}

#[test]
fn test_structure_nested() {
    check(
        r#"\part{1}\chapter{2}\section{3}\subsection{4}\subsubsection{5}\paragraph{6}\subparagraph{7}"#,
        expect![[r#"
            ROOT@0..90
              PREAMBLE@0..90
                PART@0..90
                  COMMAND_NAME@0..5 "\\part"
                  CURLY_GROUP@5..8
                    L_CURLY@5..6 "{"
                    TEXT@6..7
                      WORD@6..7 "1"
                    R_CURLY@7..8 "}"
                  CHAPTER@8..90
                    COMMAND_NAME@8..16 "\\chapter"
                    CURLY_GROUP@16..19
                      L_CURLY@16..17 "{"
                      TEXT@17..18
                        WORD@17..18 "2"
                      R_CURLY@18..19 "}"
                    SECTION@19..90
                      COMMAND_NAME@19..27 "\\section"
                      CURLY_GROUP@27..30
                        L_CURLY@27..28 "{"
                        TEXT@28..29
                          WORD@28..29 "3"
                        R_CURLY@29..30 "}"
                      SUBSECTION@30..90
                        COMMAND_NAME@30..41 "\\subsection"
                        CURLY_GROUP@41..44
                          L_CURLY@41..42 "{"
                          TEXT@42..43
                            WORD@42..43 "4"
                          R_CURLY@43..44 "}"
                        SUBSUBSECTION@44..90
                          COMMAND_NAME@44..58 "\\subsubsection"
                          CURLY_GROUP@58..61
                            L_CURLY@58..59 "{"
                            TEXT@59..60
                              WORD@59..60 "5"
                            R_CURLY@60..61 "}"
                          PARAGRAPH@61..90
                            COMMAND_NAME@61..71 "\\paragraph"
                            CURLY_GROUP@71..74
                              L_CURLY@71..72 "{"
                              TEXT@72..73
                                WORD@72..73 "6"
                              R_CURLY@73..74 "}"
                            SUBPARAGRAPH@74..90
                              COMMAND_NAME@74..87 "\\subparagraph"
                              CURLY_GROUP@87..90
                                L_CURLY@87..88 "{"
                                TEXT@88..89
                                  WORD@88..89 "7"
                                R_CURLY@89..90 "}"

        "#]],
    );
}

#[test]
fn test_structure_siblings() {
    check(
        r#"\section{Foo} Foo \section{Bar} Bar"#,
        expect![[r#"
        ROOT@0..35
          PREAMBLE@0..35
            SECTION@0..18
              COMMAND_NAME@0..8 "\\section"
              CURLY_GROUP@8..14
                L_CURLY@8..9 "{"
                TEXT@9..12
                  WORD@9..12 "Foo"
                R_CURLY@12..13 "}"
                WHITESPACE@13..14 " "
              TEXT@14..18
                WORD@14..17 "Foo"
                WHITESPACE@17..18 " "
            SECTION@18..35
              COMMAND_NAME@18..26 "\\section"
              CURLY_GROUP@26..32
                L_CURLY@26..27 "{"
                TEXT@27..30
                  WORD@27..30 "Bar"
                R_CURLY@30..31 "}"
                WHITESPACE@31..32 " "
              TEXT@32..35
                WORD@32..35 "Bar"

    "#]],
    );
}

#[test]
fn test_theorem_definition_full() {
    check(
        r#"\newtheorem{foo}[bar]{Foo}[baz]"#,
        expect![[r#"
        ROOT@0..31
          PREAMBLE@0..31
            THEOREM_DEFINITION_AMSTHM@0..31
              COMMAND_NAME@0..11 "\\newtheorem"
              CURLY_GROUP_WORD@11..16
                L_CURLY@11..12 "{"
                KEY@12..15
                  WORD@12..15 "foo"
                R_CURLY@15..16 "}"
              BRACK_GROUP_WORD@16..21
                L_BRACK@16..17 "["
                KEY@17..20
                  WORD@17..20 "bar"
                R_BRACK@20..21 "]"
              CURLY_GROUP@21..26
                L_CURLY@21..22 "{"
                TEXT@22..25
                  WORD@22..25 "Foo"
                R_CURLY@25..26 "}"
              BRACK_GROUP_WORD@26..31
                L_BRACK@26..27 "["
                KEY@27..30
                  WORD@27..30 "baz"
                R_BRACK@30..31 "]"

    "#]],
    );
}

#[test]
fn test_theorem_definition_name_with_counter() {
    check(
        r#"\newtheorem{foo}[bar]"#,
        expect![[r#"
        ROOT@0..21
          PREAMBLE@0..21
            THEOREM_DEFINITION_AMSTHM@0..21
              COMMAND_NAME@0..11 "\\newtheorem"
              CURLY_GROUP_WORD@11..16
                L_CURLY@11..12 "{"
                KEY@12..15
                  WORD@12..15 "foo"
                R_CURLY@15..16 "}"
              BRACK_GROUP_WORD@16..21
                L_BRACK@16..17 "["
                KEY@17..20
                  WORD@17..20 "bar"
                R_BRACK@20..21 "]"

    "#]],
    );
}

#[test]
fn test_theorem_definition_name_with_description() {
    check(
        r#"\newtheorem{foo}{Foo}"#,
        expect![[r#"
        ROOT@0..21
          PREAMBLE@0..21
            THEOREM_DEFINITION_AMSTHM@0..21
              COMMAND_NAME@0..11 "\\newtheorem"
              CURLY_GROUP_WORD@11..16
                L_CURLY@11..12 "{"
                KEY@12..15
                  WORD@12..15 "foo"
                R_CURLY@15..16 "}"
              CURLY_GROUP@16..21
                L_CURLY@16..17 "{"
                TEXT@17..20
                  WORD@17..20 "Foo"
                R_CURLY@20..21 "}"

    "#]],
    );
}

#[test]
fn test_theorem_definition_name_with_description_and_counter() {
    check(
        r#"\newtheorem{foo}[bar]{Foo}"#,
        expect![[r#"
        ROOT@0..26
          PREAMBLE@0..26
            THEOREM_DEFINITION_AMSTHM@0..26
              COMMAND_NAME@0..11 "\\newtheorem"
              CURLY_GROUP_WORD@11..16
                L_CURLY@11..12 "{"
                KEY@12..15
                  WORD@12..15 "foo"
                R_CURLY@15..16 "}"
              BRACK_GROUP_WORD@16..21
                L_BRACK@16..17 "["
                KEY@17..20
                  WORD@17..20 "bar"
                R_BRACK@20..21 "]"
              CURLY_GROUP@21..26
                L_CURLY@21..22 "{"
                TEXT@22..25
                  WORD@22..25 "Foo"
                R_CURLY@25..26 "}"

    "#]],
    );
}

#[test]
fn test_theorem_definition_only_name() {
    check(
        r#"\newtheorem{foo}"#,
        expect![[r#"
        ROOT@0..16
          PREAMBLE@0..16
            THEOREM_DEFINITION_AMSTHM@0..16
              COMMAND_NAME@0..11 "\\newtheorem"
              CURLY_GROUP_WORD@11..16
                L_CURLY@11..12 "{"
                KEY@12..15
                  WORD@12..15 "foo"
                R_CURLY@15..16 "}"

    "#]],
    );
}

#[test]
fn test_theorem_definition_thmtools() {
    check(
        r#"\declaretheorem[style=foo, name=bar]{baz}"#,
        expect![[r#"
        ROOT@0..41
          PREAMBLE@0..41
            THEOREM_DEFINITION_THMTOOLS@0..41
              COMMAND_NAME@0..15 "\\declaretheorem"
              BRACK_GROUP_KEY_VALUE@15..36
                L_BRACK@15..16 "["
                KEY_VALUE_BODY@16..35
                  KEY_VALUE_PAIR@16..25
                    KEY@16..21
                      WORD@16..21 "style"
                    EQUALITY_SIGN@21..22 "="
                    VALUE@22..25
                      TEXT@22..25
                        WORD@22..25 "foo"
                  COMMA@25..26 ","
                  WHITESPACE@26..27 " "
                  KEY_VALUE_PAIR@27..35
                    KEY@27..31
                      WORD@27..31 "name"
                    EQUALITY_SIGN@31..32 "="
                    VALUE@32..35
                      TEXT@32..35
                        WORD@32..35 "bar"
                R_BRACK@35..36 "]"
              CURLY_GROUP_WORD@36..41
                L_CURLY@36..37 "{"
                KEY@37..40
                  WORD@37..40 "baz"
                R_CURLY@40..41 "}"

    "#]],
    );
}

#[test]
fn test_command_subscript() {
    check(
        r#"\foo_bar \foo_\bar"#,
        expect![[r#"
        ROOT@0..18
          PREAMBLE@0..18
            GENERIC_COMMAND@0..9
              COMMAND_NAME@0..8 "\\foo_bar"
              WHITESPACE@8..9 " "
            GENERIC_COMMAND@9..13
              COMMAND_NAME@9..13 "\\foo"
            TEXT@13..14
              WORD@13..14 "_"
            GENERIC_COMMAND@14..18
              COMMAND_NAME@14..18 "\\bar"

    "#]],
    );
}
