pub mod support;

use lsp_types::*;
use std::sync::Arc;
use support::*;
use tex::DistributionKind::{Miktex, Texlive};
use tokio::process::Command;
use std::process::Stdio;

const SCENARIO: &str = "hover/latex/preview";

async fn dvipng_installed() -> bool {
    Command::new("dvipng")
        .arg("--version")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await
        .is_ok()
}

async fn run(name: &'static str, line: u64, character: u64) {
    let _ = tex::with_distro(&[Texlive, Miktex], |distro| {
        async move {
            if dvipng_installed().await {
                let scenario = Scenario::new(SCENARIO, Arc::new(distro));
                scenario
                    .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
                    .await;

                let tex_file = format!("{}.tex", name);
                let png_file = format!("{}.png", name);
                scenario.open(&tex_file).await;

                let text_document = TextDocumentIdentifier::new(scenario.uri(&tex_file).into());
                let position = Position::new(line, character);
                let params = TextDocumentPositionParams {
                    text_document,
                    position,
                };
                let hover = scenario
                    .server
                    .execute_async(|svr| svr.hover(params))
                    .await
                    .unwrap()
                    .unwrap();

                let png_data = scenario.read_bytes(&png_file).await;
                assert_eq!(
                    hover.contents,
                    HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "![preview](data:image/png;base64,{})",
                            base64::encode(&png_data)
                        )
                    })
                );
            }
        }
    })
    .await;
}

#[tokio::test]
async fn equation() {
    run("equation", 7, 10).await;
}

#[tokio::test]
async fn inline() {
    run("inline", 7, 7).await;
}

#[tokio::test]
async fn environment() {
    run("environment", 10, 10).await;
}
