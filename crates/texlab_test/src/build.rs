use super::capabilities::CLIENT_FULL_CAPABILITIES;
use super::scenario::Scenario;
use texlab_distro::DistributionKind::*;
use texlab_protocol::*;

async fn create_scenario(
    executable: &'static str,
    build_on_save: bool,
    file: &'static str,
) -> Scenario {
    let scenario = Scenario::new("build", true).await;
    scenario.initialize(&CLIENT_FULL_CAPABILITIES).await;

    *scenario.client.options.lock().await = Options {
        latex: Some(LatexOptions {
            build: Some(LatexBuildOptions {
                executable: Some(executable.into()),
                args: None,
                on_save: Some(build_on_save),
                output_directory: None,
            }),
            ..LatexOptions::default()
        }),
        bibtex: None,
    };

    scenario.open(file).await;
    scenario
}

pub async fn run_command(executable: &'static str, file: &'static str) -> Option<BuildResult> {
    let scenario = create_scenario(executable, false, file).await;
    match scenario.distribution.kind() {
        Texlive | Miktex => {
            let text_document = TextDocumentIdentifier::new(scenario.uri(file).into());
            let params = BuildParams { text_document };
            let result = scenario
                .server
                .execute(|svr| svr.build(params))
                .await
                .unwrap();
            Some(result)
        }
        Tectonic | Unknown => None,
    }
}

pub async fn run_on_save(executable: &'static str, file: &'static str) -> Option<Scenario> {
    let scenario = create_scenario(executable, true, file).await;
    match scenario.distribution.kind() {
        Texlive | Miktex => {
            let text_document = TextDocumentIdentifier::new(scenario.uri(file).into());
            let params = DidSaveTextDocumentParams { text_document };
            scenario.server.execute(|svr| svr.did_save(params)).await;
            Some(scenario)
        }
        Tectonic | Unknown => None,
    }
}
