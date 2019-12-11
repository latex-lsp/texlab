pub mod support;

use std::sync::Arc;
use support::capabilities::CLIENT_FULL_CAPABILITIES;
use support::*;
use texlab_distro::UnknownDistribution;
use texlab_protocol::*;
use tokio::fs;

const SCENARIO: &str = "hover/latex/label";

#[tokio::test]
async fn reload_aux() {
    let scenario = Scenario::new(SCENARIO, Arc::new(Box::new(UnknownDistribution)));
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;
    scenario.open("section.tex").await;
    let position = Position::new(3, 10);
    let identifier = TextDocumentIdentifier::new(scenario.uri("section.tex").into());
    let params = TextDocumentPositionParams::new(identifier, position);
    let contents = scenario
        .server
        .execute_async(|svr| svr.hover(params.clone()))
        .await
        .unwrap()
        .unwrap()
        .contents;

    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Section (Foo)".into()
        })
    );

    let aux_path = scenario
        .uri("section.tex")
        .to_file_path()
        .unwrap()
        .with_extension("aux");
    fs::write(aux_path, "\\newlabel{sec:foo}{{1}{1}}")
        .await
        .unwrap();

    let contents = scenario
        .server
        .execute_async(|svr| svr.hover(params))
        .await
        .unwrap()
        .unwrap()
        .contents;

    assert_eq!(
        contents,
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::PlainText,
            value: "Section 1 (Foo)".into()
        })
    );
}
