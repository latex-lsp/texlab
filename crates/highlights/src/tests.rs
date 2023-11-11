use crate::{HighlightKind, HighlightParams};

fn check(input: &str, expected_kinds: &[HighlightKind]) {
    let fixture = test_utils::fixture::Fixture::parse(input);

    let (feature, offset) = fixture.make_params().unwrap();

    let expected_ranges = fixture
        .locations()
        .map(|location| location.range)
        .collect::<Vec<_>>();

    let results = crate::find_all(HighlightParams { feature, offset });

    let actual_ranges = results
        .iter()
        .map(|result| result.range)
        .collect::<Vec<_>>();
    assert_eq!(actual_ranges, expected_ranges);

    assert_eq!(
        results.iter().map(|result| result.kind).collect::<Vec<_>>(),
        expected_kinds
    );
}

#[test]
fn test_label() {
    check(
        r#"
%! main.tex
\label{foo}
        |
       ^^^
\ref{foo}
     ^^^
\label{bar}
"#,
        &[HighlightKind::Write, HighlightKind::Read],
    )
}
