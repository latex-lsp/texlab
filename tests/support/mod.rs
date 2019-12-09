use copy_dir::copy_dir;
use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc::client::Result;
use lsp_types::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};
use texlab::client::LspClient;
use texlab::protocol_types::*;
use texlab::server::LatexLspServer;
use texlab::workspace::Uri;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct MockLspClientOptions {
    pub bibtex_formatting: Option<BibtexFormattingOptions>,
    pub latex_lint: Option<LatexLintOptions>,
    pub latex_build: Option<LatexBuildOptions>,
}

#[derive(Debug, Default)]
pub struct MockLspClient {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<MockLspClientOptions>,
    pub diagnostics_by_uri: Mutex<HashMap<Uri, Vec<Diagnostic>>>,
    pub log_messages: Mutex<Vec<LogMessageParams>>,
}

impl MockLspClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn verify_no_diagnostics(&self, uri: &Uri) {
        let diagnostics_by_uri = self.diagnostics_by_uri.lock().await;
        assert_eq!(
            diagnostics_by_uri
                .get(uri.into())
                .map(Vec::len)
                .unwrap_or(0),
            0
        );
    }
}

impl LspClient for MockLspClient {
    #[boxed]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value> {
        fn serialize<T>(options: &Option<T>) -> Result<serde_json::Value>
        where
            T: Serialize,
        {
            options
                .as_ref()
                .map(|options| serde_json::to_value(vec![options]).unwrap())
                .ok_or_else(|| jsonrpc::Error::internal_error("Internal error".to_owned()))
        }

        let options = self.options.lock().await;
        match params.items[0].section.as_ref().unwrap().as_ref() {
            "bibtex.formatting" => serialize(&options.bibtex_formatting),
            "latex.lint" => serialize(&options.latex_lint),
            "latex.build" => serialize(&options.latex_build),
            _ => panic!("Invalid language configuration!"),
        }
    }

    #[boxed]
    async fn show_message(&self, params: ShowMessageParams) {
        let mut messages = self.messages.lock().await;
        messages.push(params);
    }

    #[boxed]
    async fn register_capability(&self, _params: RegistrationParams) -> Result<()> {
        Ok(())
    }

    #[boxed]
    async fn publish_diagnostics(&self, params: PublishDiagnosticsParams) {
        let mut diagnostics_by_uri = self.diagnostics_by_uri.lock().await;
        diagnostics_by_uri.insert(params.uri.into(), params.diagnostics);
    }

    #[boxed]
    async fn work_done_progress_create(&self, _params: WorkDoneProgressCreateParams) -> Result<()> {
        Ok(())
    }

    #[boxed]
    async fn progress(&self, _params: ProgressParams) {}

    #[boxed]
    async fn log_message(&self, params: LogMessageParams) {
        let mut messages = self.log_messages.lock().await;
        messages.push(params);
    }
}

pub struct Scenario {
    pub directory: TempDir,
    pub server: LatexLspServer<MockLspClient>,
    pub client: Arc<MockLspClient>,
}

impl Scenario {
    pub fn new(name: &str, distribution: Arc<Box<dyn tex::Distribution>>) -> Self {
        let directory = tempdir().unwrap();
        remove_dir(directory.path()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("scenarios")
            .join(name);
        copy_dir(source, directory.path()).unwrap();

        let client = Arc::new(MockLspClient::new());
        let server = LatexLspServer::new(Arc::clone(&client), distribution);
        Self {
            directory,
            server,
            client,
        }
    }

    pub async fn initialize(&self, capabilities: &ClientCapabilities) {
        let root_uri = Uri::from_file_path(self.directory.path()).unwrap();
        let params = InitializeParams {
            process_id: None,
            root_path: Some(self.directory.path().to_string_lossy().into_owned()),
            root_uri: Some(root_uri.into()),
            initialization_options: None,
            capabilities: capabilities.clone(),
            trace: None,
            workspace_folders: None,
        };

        self.server
            .execute_async(|svr| svr.initialize(params))
            .await
            .unwrap();

        self.server
            .execute(|svr| svr.initialized(InitializedParams {}))
            .await;
    }

    pub fn uri(&self, name: &str) -> Uri {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        Uri::from_file_path(path).unwrap()
    }

    pub async fn read(&self, name: &'static str) -> String {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        let data = tokio::fs::read(path)
            .await
            .expect("failed to read scenario file");
        let text = String::from_utf8_lossy(&data);
        text.replace('\r', "")
    }

    pub async fn open(&self, name: &'static str) {
        let text = self.read(name).await;
        let language_id = if name.ends_with(".bib") {
            "bibtex"
        } else {
            "latex"
        };

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: self.uri(name).into(),
                version: 0,
                language_id: language_id.to_owned(),
                text,
            },
        };
        self.server.execute(|svr| svr.did_open(params)).await;
    }
}

pub mod capabilities {
    use lsp_types::*;

    pub static CLIENT_FULL_CAPABILITIES: ClientCapabilities = ClientCapabilities {
        workspace: Some(WorkspaceClientCapabilities {
            configuration: Some(true),
            did_change_watched_files: None,
            workspace_folders: None,
            apply_edit: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            did_change_configuration: None,
        }),
        text_document: Some(TextDocumentClientCapabilities {
            synchronization: None,
            completion: None,
            hover: None,
            signature_help: None,
            references: None,
            document_highlight: None,
            document_symbol: Some(DocumentSymbolCapability {
                dynamic_registration: None,
                hierarchical_document_symbol_support: Some(true),
                symbol_kind: None,
            }),
            formatting: None,
            range_formatting: None,
            on_type_formatting: None,
            declaration: None,
            definition: Some(GotoCapability {
                dynamic_registration: None,
                link_support: Some(true),
            }),
            type_definition: None,
            implementation: None,
            code_action: None,
            code_lens: None,
            document_link: None,
            color_provider: None,
            rename: None,
            publish_diagnostics: None,
            folding_range: None,
        }),
        experimental: None,
        window: Some(WindowClientCapabilities {
            work_done_progress: Some(true),
        }),
    };

    pub static CLIENT_NO_LINK_CAPABILITIES: ClientCapabilities = ClientCapabilities {
        workspace: Some(WorkspaceClientCapabilities {
            configuration: Some(true),
            did_change_watched_files: None,
            workspace_folders: None,
            apply_edit: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            did_change_configuration: None,
        }),
        text_document: Some(TextDocumentClientCapabilities {
            synchronization: None,
            completion: None,
            hover: None,
            signature_help: None,
            references: None,
            document_highlight: None,
            document_symbol: None,
            formatting: None,
            range_formatting: None,
            on_type_formatting: None,
            declaration: None,
            definition: Some(GotoCapability {
                dynamic_registration: None,
                link_support: Some(false),
            }),
            type_definition: None,
            implementation: None,
            code_action: None,
            code_lens: None,
            document_link: None,
            color_provider: None,
            rename: None,
            publish_diagnostics: None,
            folding_range: None,
        }),
        experimental: None,
        window: Some(WindowClientCapabilities {
            work_done_progress: Some(true),
        }),
    };
}

pub mod build {
    use super::*;
    use tex::DistributionKind::*;

    async fn create_scenario(
        distribution: Arc<Box<dyn tex::Distribution>>,
        executable: &'static str,
        build_on_save: bool,
        file: &'static str,
    ) -> Scenario {
        let scenario = Scenario::new("build", distribution);
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;

        let options = LatexBuildOptions {
            executable: Some(executable.into()),
            args: None,
            on_save: Some(build_on_save),
        };
        scenario.client.options.lock().await.latex_build = Some(options);

        scenario.open(file).await;
        scenario
    }

    pub async fn run(executable: &'static str, file: &'static str) -> Option<BuildResult> {
        tex::with_distro(&[Texlive, Miktex], |distro| {
            async move {
                let scenario = create_scenario(distro, executable, false, file).await;
                let text_document = TextDocumentIdentifier::new(scenario.uri(file).into());
                let params = BuildParams { text_document };
                scenario
                    .server
                    .execute_async(|svr| svr.build(params))
                    .await
                    .unwrap()
            }
        })
        .await
    }

    pub async fn run_on_save(executable: &'static str, file: &'static str) -> Option<Scenario> {
        tex::with_distro(&[Texlive, Miktex], |distro| {
            async move {
                let scenario = create_scenario(distro, executable, true, file).await;
                let text_document = TextDocumentIdentifier::new(scenario.uri(file).into());
                let params = DidSaveTextDocumentParams { text_document };
                scenario.server.execute(|svr| svr.did_save(params)).await;
                scenario
            }
        })
        .await
    }
}

pub mod completion {
    use super::*;

    pub async fn run_list(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
    ) -> (Scenario, Vec<CompletionItem>) {
        let scenario_name = format!("completion/{}", scenario_short_name);
        let scenario = Scenario::new(&scenario_name, Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open(file).await;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
                position: Position::new(line, character),
            },
            context: None,
        };

        let items = scenario
            .server
            .execute_async(|svr| svr.completion(params))
            .await
            .unwrap()
            .items;

        (scenario, items)
    }

    pub async fn run_empty(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
    ) {
        assert!(run_list(scenario_short_name, file, line, character)
            .await
            .1
            .is_empty());
    }

    pub async fn run_item(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
        item_name: &'static str,
    ) -> CompletionItem {
        let (scenario, items) = run_list(scenario_short_name, file, line, character).await;

        let item = items
            .into_iter()
            .find(|item| item.label == item_name)
            .unwrap();

        scenario
            .server
            .execute_async(|svr| svr.completion_resolve(item))
            .await
            .unwrap()
    }

    pub mod verify {
        use lsp_types::*;
        use texlab::range::RangeExt;

        pub fn text_edit(
            item: &CompletionItem,
            start_line: u64,
            start_character: u64,
            end_line: u64,
            end_character: u64,
            text: &str,
        ) {
            assert_eq!(
                item.text_edit,
                Some(TextEdit::new(
                    Range::new_simple(start_line, start_character, end_line, end_character),
                    text.into()
                ))
            );
        }

        pub fn detail(item: &CompletionItem, detail: &str) {
            assert_eq!(item.detail.as_ref().unwrap(), detail);
        }

        pub fn labels(items: &[CompletionItem], expected_labels: Vec<&'static str>) {
            let mut actual_labels: Vec<&str> =
                items.iter().map(|item| item.label.as_ref()).collect();
            actual_labels.sort();
            assert_eq!(actual_labels, expected_labels);
        }
    }
}

pub mod definition {
    use super::capabilities::*;
    use super::*;
    use texlab::protocol_types::DefinitionResponse;

    pub async fn run(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
        capabilities: &ClientCapabilities,
    ) -> (Scenario, DefinitionResponse) {
        let scenario_name = format!("definition/{}", scenario_short_name);
        let scenario = Scenario::new(&scenario_name, Arc::new(Box::new(tex::Unknown)));
        scenario.initialize(capabilities).await;
        scenario.open(file).await;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
            position: Position::new(line, character),
        };

        let response = scenario
            .server
            .execute_async(|svr| svr.definition(params))
            .await
            .unwrap();

        (scenario, response)
    }

    pub async fn run_link(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
    ) -> (Scenario, Vec<LocationLink>) {
        let (scenario, response) = run(
            scenario_short_name,
            file,
            line,
            character,
            &CLIENT_FULL_CAPABILITIES,
        )
        .await;
        match response {
            DefinitionResponse::LocationLinks(links) => (scenario, links),
            DefinitionResponse::Locations(_) => unreachable!(),
        }
    }

    pub async fn run_location(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
    ) -> (Scenario, Vec<Location>) {
        let (scenario, response) = run(
            scenario_short_name,
            file,
            line,
            character,
            &CLIENT_NO_LINK_CAPABILITIES,
        )
        .await;
        match response {
            DefinitionResponse::LocationLinks(_) => unreachable!(),
            DefinitionResponse::Locations(locations) => (scenario, locations),
        }
    }

    pub mod verify {
        use super::*;
        use texlab::range::RangeExt;

        pub fn origin_selection_range(
            link: &LocationLink,
            start_line: u64,
            start_character: u64,
            end_line: u64,
            end_character: u64,
        ) {
            assert_eq!(
                link.origin_selection_range,
                Some(Range::new_simple(
                    start_line,
                    start_character,
                    end_line,
                    end_character
                ))
            );
        }
    }
}

pub mod folding {
    use super::*;
    use std::cmp::Reverse;

    pub async fn run(file: &'static str) -> Vec<FoldingRange> {
        let scenario = Scenario::new("folding", Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open(file).await;
        let params = FoldingRangeParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        };

        let mut foldings = scenario
            .server
            .execute_async(|svr| svr.folding_range(params))
            .await
            .unwrap();

        foldings.sort_by_key(|folding| {
            let start = Position::new(folding.start_line, folding.start_character.unwrap());
            let end = Position::new(folding.end_line, folding.end_character.unwrap());
            (start, Reverse(end))
        });
        foldings
    }
}

pub mod formatting {
    use super::*;

    pub async fn run_bibtex(
        file: &'static str,
        options: Option<BibtexFormattingOptions>,
    ) -> (Scenario, Vec<TextEdit>) {
        let scenario = Scenario::new("formatting/bibtex", Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open(file).await;
        {
            scenario.client.options.lock().await.bibtex_formatting = options;
        }

        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                properties: HashMap::new(),
            },
        };

        let edits = scenario
            .server
            .execute_async(|svr| svr.formatting(params))
            .await
            .unwrap();
        (scenario, edits)
    }
}

pub mod hover {
    use super::*;

    pub async fn run(
        scenario_short_name: &'static str,
        file: &'static str,
        line: u64,
        character: u64,
    ) -> Option<HoverContents> {
        let scenario_name = format!("hover/{}", scenario_short_name);
        let scenario = Scenario::new(&scenario_name, Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open(file).await;
        let identifier = TextDocumentIdentifier::new(scenario.uri(file).into());
        let params = TextDocumentPositionParams::new(identifier, Position::new(line, character));
        scenario
            .server
            .execute_async(|svr| svr.hover(params))
            .await
            .unwrap()
            .map(|hover| hover.contents)
    }
}

pub mod symbol {
    use super::*;
    use lsp_types::DocumentSymbolResponse;

    pub async fn run_hierarchical(file: &'static str) -> Vec<DocumentSymbol> {
        let scenario = Scenario::new("symbol/hierarchical", Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open(file).await;
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        };

        let response = scenario
            .server
            .execute_async(|svr| svr.document_symbol(params))
            .await
            .unwrap();

        match response {
            DocumentSymbolResponse::Nested(symbols) => symbols,
            DocumentSymbolResponse::Flat(_) => unreachable!(),
        }
    }

    pub async fn run_workspace(query: &'static str) -> (Scenario, Vec<SymbolInformation>) {
        let scenario = Scenario::new("symbol/workspace", Arc::new(Box::new(tex::Unknown)));
        scenario
            .initialize(&capabilities::CLIENT_FULL_CAPABILITIES)
            .await;
        scenario.open("foo.tex").await;
        scenario.open("bar.bib").await;
        let params = WorkspaceSymbolParams {
            query: query.into(),
        };
        let symbols = scenario
            .server
            .execute_async(|svr| svr.workspace_symbol(params))
            .await
            .unwrap();

        (scenario, symbols)
    }

    pub mod verify {
        use super::*;
        use texlab::range::RangeExt;

        pub fn symbol(
            symbol: &DocumentSymbol,
            name: &str,
            detail: Option<&str>,
            selection_range: Range,
            range: Range,
        ) {
            assert_eq!(symbol.name, name);
            assert_eq!(symbol.detail.as_ref().map(AsRef::as_ref), detail);
            assert_eq!(symbol.selection_range, selection_range);
            assert_eq!(symbol.range, range);
        }

        pub fn symbol_info(
            symbol: &SymbolInformation,
            scenario: &Scenario,
            file: &str,
            name: &str,
            start_line: u64,
            start_character: u64,
            end_line: u64,
            end_character: u64,
        ) {
            assert_eq!(symbol.name, name);
            let range = Range::new_simple(start_line, start_character, end_line, end_character);
            assert_eq!(
                symbol.location,
                Location::new(scenario.uri(file).into(), range)
            );
        }
    }
}
