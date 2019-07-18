#![feature(async_await)]

use jsonrpc::server::ActionHandler;
use lsp_types::*;
use texlab::build::*;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};

async fn set_options<'a>(
    scenario: &'a Scenario,
    executable: String,
    args: Vec<String>,
    on_save: bool,
) {
    let mut options = scenario.client.options.lock().await;
    options.latex_build = Some(BuildOptions {
        executable: Some(executable),
        args: Some(args),
        on_save: Some(on_save),
    });
}

async fn run(executable: &'static str, name: &'static str) -> (Scenario, BuildResult) {
    let scenario = Scenario::new("build", &FULL_CAPABILITIES).await;
    scenario.open("foo.tex").await;
    set_options(&scenario, executable.to_owned(), Vec::new(), false).await;
    let text_document = TextDocumentIdentifier::new(scenario.uri(name));
    let params = BuildParams { text_document };
    let result = scenario.server.build(params).await.unwrap();
    scenario.server.execute_actions().await;
    (scenario, result)
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_success() {
    let (scenario, result) = run("echo", "foo.tex").await;
    assert_eq!(result.status, BuildStatus::Success);
    let log_messages = scenario.client.log_messages.lock().await;
    assert_eq!(log_messages.len(), 1);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_failure() {
    let (scenario, result) = run("foobarbaz", "foo.tex").await;
    assert_eq!(result.status, BuildStatus::Failure);
    let log_messages = scenario.client.log_messages.lock().await;
    assert_eq!(log_messages.len(), 0);
}

#[runtime::test(runtime_tokio::Tokio)]
async fn test_on_save() {
    let scenario = Scenario::new("build", &FULL_CAPABILITIES).await;
    scenario.open("foo.tex").await;
    set_options(&scenario, "echo".into(), Vec::new(), true).await;
    let text_document = TextDocumentIdentifier::new(scenario.uri("foo.tex"));
    scenario
        .server
        .did_save(DidSaveTextDocumentParams { text_document });
    scenario.server.execute_actions().await;
    let log_messages = scenario.client.log_messages.lock().await;
    assert_eq!(log_messages.len(), 1);
}
