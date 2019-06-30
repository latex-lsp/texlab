#![feature(async_await)]

use jsonrpc::server::ActionHandler;
use lsp_types::*;
use texlab::build::*;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};

async fn set_options<'a>(scenario: &'a Scenario, executable: &'a str, on_save: bool) {
    let mut build_options = BuildOptions::default();
    build_options.executable = Some(executable.to_owned());
    let mut args = build_options.args();
    args.push("--view=none".to_owned());
    build_options.args = Some(args);
    build_options.on_save = Some(on_save);
    let mut options = scenario.client.options.lock().await;
    options.latex_build = Some(build_options);
}

async fn run(executable: &'static str, name: &'static str) -> (Scenario, BuildResult) {
    let scenario = Scenario::new("build", &FULL_CAPABILITIES).await;
    scenario.open(name).await;
    set_options(&scenario, executable, false).await;
    let text_document = TextDocumentIdentifier::new(scenario.uri(name));
    let params = BuildParams { text_document };
    let result = scenario.server.build(params).await.unwrap();
    scenario.server.execute_actions().await;
    scenario.server.stop_scanning().await;
    (scenario, result)
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_success() {
    let (scenario, result) = run("latexmk", "bar.tex").await;
    let log = scenario.client.log().await;
    assert_eq!(
        result.status,
        BuildStatus::Success,
        "{}",
        log
    );
    let path = scenario.directory.path().join("foo.pdf");
    assert!(path.exists(), "{}", log);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_error() {
    let (_, result) = run("latexmk", "baz.tex").await;
    assert_eq!(result.status, BuildStatus::Error);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_failure() {
    let (_, result) = run("foobarbaz", "foo.tex").await;
    assert_eq!(result.status, BuildStatus::Failure);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_on_save() {
    let scenario = Scenario::new("build", &FULL_CAPABILITIES).await;
    scenario.open("foo.tex").await;
    set_options(&scenario, "latexmk", true).await;
    let text_document = TextDocumentIdentifier::new(scenario.uri("foo.tex"));
    scenario
        .server
        .did_save(DidSaveTextDocumentParams { text_document });
    scenario.server.execute_actions().await;
    let path = scenario.directory.path().join("foo.pdf");
    assert!(path.exists(), "{}", scenario.client.log().await);
}
