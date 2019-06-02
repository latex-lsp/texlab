#![feature(async_await)]

use futures::executor::block_on;
use lsp_types::*;
use texlab::build::*;
use texlab::scenario::Scenario;
use jsonrpc::server::ActionHandler;

async fn run(executable: &'static str, on_save: bool, name: &'static str) -> (Scenario, BuildResult) {
    let scenario = Scenario::new("build").await;
    scenario.open(name).await;
    let mut build_options = BuildOptions::default();
    build_options.executable = executable.to_owned();
    build_options.args.push("--view=none".to_owned());
    build_options.on_save = on_save;
    {
        let mut options = scenario.client.options.lock().await;
        options.latex_build = Some(build_options);
    }
    let text_document = TextDocumentIdentifier::new(scenario.uri(name));
    let params = BuildParams { text_document};
    let result = scenario.server.build(params).await.unwrap();
    scenario.server.execute_actions().await;
    (scenario, result)
}

#[test]
fn test_success() {
    block_on(async move {
        let (scenario, result) = run("latexmk", false, "bar.tex").await;
        assert_eq!(result.status, BuildStatus::Success);
        let path = scenario.directory.path().join("foo.pdf");
        assert!(path.exists());
    });
}

#[test]
fn test_error() {
    block_on(async move {
        let (_, result) = run("latexmk", false, "baz.tex").await;
        assert_eq!(result.status, BuildStatus::Error);
    })
}

#[test]
fn test_failure() {
    block_on(async move {
        let (_, result) = run("foobarbaz", false, "foo.tex").await;
        assert_eq!(result.status, BuildStatus::Failure);
    });
}

#[test]
fn test_on_save() {
    block_on(async move {
        let scenario = Scenario::new("build").await;
        scenario.open("foo.tex").await;
        let mut build_options = BuildOptions::default();
        build_options.args.push("--view=none".to_owned());
        build_options.on_save = true;
        {
            let mut options = scenario.client.options.lock().await;
            options.latex_build = Some(build_options);
        }
        let text_document = TextDocumentIdentifier::new(scenario.uri("foo.tex"));
        scenario.server.did_save(DidSaveTextDocumentParams { text_document });
        scenario.server.execute_actions().await;
        let path = scenario.directory.path().join("foo.pdf");
        assert!(path.exists());
    });
}
