use expect_test::{expect, Expect};

fn check(input: &str, expect: Expect) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let (params, _) = fixture.make_params().unwrap();
    let links = crate::find_links(&params);

    let actual_ranges = links.iter().map(|link| link.range).collect::<Vec<_>>();

    let expected_ranges = fixture
        .locations()
        .map(|location| location.range)
        .collect::<Vec<_>>();

    assert_eq!(actual_ranges, expected_ranges);

    let actual_targets = links
        .iter()
        .map(|link| link.document.uri.as_str())
        .collect::<Vec<_>>();

    expect.assert_debug_eq(&actual_targets);
}

#[test]
fn test_document_include() {
    check(
        r#"
%! foo.tex
\input{bar.tex}
       ^^^^^^^

%! bar.tex"#,
        expect![[r#"
            [
                "file:///texlab/bar.tex",
            ]
        "#]],
    );
}

#[test]
fn test_document_import() {
    check(
        r#"
%! foo.tex
\import{.}{bar/baz}
           ^^^^^^^

%! bar/baz.tex"#,
        expect![[r#"
            [
                "file:///texlab/bar/baz.tex",
            ]
        "#]],
    );
}
