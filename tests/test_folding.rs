pub mod support;

use support::folding::*;
use texlab_protocol::*;

#[tokio::test]
async fn bibtex() {
    let foldings = run("bar.bib").await;
    assert_eq!(
        foldings,
        vec![
            FoldingRange {
                start_line: 0,
                start_character: Some(0),
                end_line: 0,
                end_character: Some(23),
                kind: Some(FoldingRangeKind::Region)
            },
            FoldingRange {
                start_line: 2,
                start_character: Some(0),
                end_line: 2,
                end_character: Some(19),
                kind: Some(FoldingRangeKind::Region)
            },
            FoldingRange {
                start_line: 4,
                start_character: Some(0),
                end_line: 23,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region)
            }
        ]
    );
}

#[tokio::test]
async fn latex() {
    let foldings = run("foo.tex").await;
    assert_eq!(
        foldings,
        vec![
            FoldingRange {
                start_line: 4,
                start_character: Some(16),
                end_line: 12,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region)
            },
            FoldingRange {
                start_line: 6,
                start_character: Some(13),
                end_line: 9,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region)
            },
            FoldingRange {
                start_line: 8,
                start_character: Some(16),
                end_line: 9,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region)
            },
        ]
    );
}
