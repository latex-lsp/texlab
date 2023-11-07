use base_db::FeatureParams;

use crate::{HighlightKind, HighlightParams};

fn check(input: &str, kinds: &[HighlightKind]) {
    let fixture = test_utils::fixture::Fixture::parse(input);
    let workspace = &fixture.workspace;
    let (document, spec, offset) = fixture
        .documents
        .iter()
        .find_map(|spec| Some((workspace.lookup(&spec.uri)?, spec, spec.cursor?)))
        .unwrap();

    let feature = FeatureParams::new(&fixture.workspace, document);
    let params = HighlightParams { feature, offset };
    let results = crate::find_all(params);

    assert_eq!(
        results
            .iter()
            .map(|result| result.range)
            .collect::<Vec<_>>(),
        spec.ranges
    );

    assert_eq!(
        results.iter().map(|result| result.kind).collect::<Vec<_>>(),
        kinds
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
