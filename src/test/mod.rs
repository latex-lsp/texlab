#![allow(deprecated)]

mod client;
mod server;

pub use self::{
    client::{TestLatexLspClient, TestLspClient},
    server::TestLatexLspServer,
};

use crate::server::LatexLspServer;
use futures::{
    channel::mpsc,
    future::{join, AbortHandle, Abortable},
    lock::Mutex,
    prelude::*,
};
use jsonrpc::MessageHandler;
use once_cell::sync::Lazy;
use std::{path::PathBuf, sync::Arc};
use tempfile::{tempdir, TempDir};
use texlab_protocol::*;
use texlab_tex::DynamicDistribution;
use tokio::fs;

struct GlobalDistribution {
    distro: Mutex<Option<DynamicDistribution>>,
}

impl GlobalDistribution {
    fn new() -> Self {
        Self {
            distro: Mutex::new(None),
        }
    }

    async fn get(&self, use_distro: bool) -> DynamicDistribution {
        if use_distro {
            let mut distro_lock = self.distro.lock().await;
            match &*distro_lock {
                Some(distro) => distro.clone(),
                None => {
                    let distro = DynamicDistribution::detect().await;
                    *distro_lock = Some(distro.clone());
                    distro
                }
            }
        } else {
            DynamicDistribution::default()
        }
    }
}

static DISTRO: Lazy<GlobalDistribution> = Lazy::new(GlobalDistribution::new);

#[derive(Debug)]
struct Endpoint<S, C> {
    receiver: mpsc::Receiver<String>,
    client: Arc<C>,
    server: Arc<S>,
    handler: MessageHandler<S, C>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TestBedBuilder {
    files: Vec<(PathBuf, String)>,
    use_distro: bool,
    root_dir: Option<PathBuf>,
    latex_build: Option<LatexBuildOptions>,
    latex_lint: Option<LatexLintOptions>,
    latex_forward_search: Option<LatexForwardSearchOptions>,
    bibtex_formatting: Option<BibtexFormattingOptions>,
}

impl TestBedBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file<P: Into<PathBuf>>(&mut self, path: P, text: &str) -> &mut Self {
        self.files.push((path.into(), text.trim().into()));
        self
    }

    pub fn use_distro(&mut self) -> &mut Self {
        self.use_distro = true;
        self
    }

    pub fn root_dir<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.root_dir = Some(path.into());
        self
    }

    pub fn latex_build(&mut self, options: LatexBuildOptions) -> &mut Self {
        self.latex_build = Some(options);
        self
    }

    pub fn latex_lint(&mut self, options: LatexLintOptions) -> &mut Self {
        self.latex_lint = Some(options);
        self
    }

    pub fn latex_forward_search(&mut self, options: LatexForwardSearchOptions) -> &mut Self {
        self.latex_forward_search = Some(options);
        self
    }

    pub fn bibtex_formatting(&mut self, options: BibtexFormattingOptions) -> &mut Self {
        self.bibtex_formatting = Some(options);
        self
    }

    pub async fn build(&self) -> TestBed {
        let dir = tempdir().expect("failed to create temporary directory");
        for (path, text) in &self.files {
            let full_path = dir.path().join(path);
            fs::create_dir_all(full_path.parent().unwrap())
                .await
                .unwrap();
            fs::write(&full_path, text).await.unwrap();
        }

        let (tx1, rx1) = mpsc::channel(0);
        let (tx2, rx2) = mpsc::channel(0);

        let endpoint1 = self.build_endpoint1(&dir, tx2, rx1).await;
        let endpoint2 = self.build_endpoint2(&dir, tx1, rx2).await;
        let server = Arc::clone(&endpoint2.server);
        let client = Arc::clone(&endpoint2.client);
        TestBed {
            dir,
            server,
            client,
            endpoint1: Some(endpoint1),
            endpoint2: Some(endpoint2),
            handle: None,
        }
    }

    async fn build_endpoint1(
        &self,
        dir: &TempDir,
        tx2: mpsc::Sender<String>,
        rx1: mpsc::Receiver<String>,
    ) -> Endpoint<LatexLspServer<LatexLspClient>, LatexLspClient> {
        let client = Arc::new(LatexLspClient::new(tx2.clone()));
        let server = Arc::new(LatexLspServer::new(
            DISTRO.get(self.use_distro).await,
            Arc::clone(&client),
            Arc::new(dir.path().to_path_buf()),
        ));

        Endpoint {
            receiver: rx1,
            client: Arc::clone(&client),
            server: Arc::clone(&server),
            handler: MessageHandler {
                client,
                server,
                output: tx2.clone(),
            },
        }
    }

    async fn build_endpoint2(
        &self,
        dir: &TempDir,
        tx1: mpsc::Sender<String>,
        rx2: mpsc::Receiver<String>,
    ) -> Endpoint<TestLatexLspServer, TestLatexLspClient> {
        let options = Options {
            latex: Some(LatexOptions {
                root_directory: self.root_dir.as_ref().map(|path| dir.path().join(path)),
                build: self.latex_build.clone(),
                forward_search: self.latex_forward_search.clone(),
                lint: self.latex_lint.clone(),
            }),
            bibtex: Some(BibtexOptions {
                formatting: self.bibtex_formatting.clone(),
            }),
        };

        let test_client = Arc::new(TestLatexLspClient::new(tx1.clone()));
        let test_server = Arc::new(TestLatexLspServer::new(options));

        Endpoint {
            receiver: rx2,
            client: Arc::clone(&test_client),
            server: Arc::clone(&test_server),
            handler: MessageHandler {
                client: test_client,
                server: test_server,
                output: tx1,
            },
        }
    }
}

pub struct TestBed {
    pub dir: TempDir,
    pub server: Arc<TestLatexLspServer>,
    pub client: Arc<TestLatexLspClient>,
    endpoint1: Option<Endpoint<LatexLspServer<LatexLspClient>, LatexLspClient>>,
    endpoint2: Option<Endpoint<TestLatexLspServer, TestLatexLspClient>>,
    handle: Option<AbortHandle>,
}

impl TestBed {
    pub fn spawn(&mut self) {
        let (handle, reg) = AbortHandle::new_pair();

        let endpoint1 = self.endpoint1.take().unwrap();
        let endpoint2 = self.endpoint2.take().unwrap();
        let mut rx1 = endpoint1.receiver;
        let mut rx2 = endpoint2.receiver;
        let mut handler1 = endpoint1.handler;
        let mut handler2 = endpoint2.handler;

        tokio::spawn(Abortable::new(
            async move {
                let task1 = async move {
                    while let Some(json) = rx2.next().await {
                        handler2.handle(&json).await;
                    }
                };

                let task2 = async move {
                    while let Some(json) = rx1.next().await {
                        handler1.handle(&json).await;
                    }
                };

                join(task1, task2).await;
            },
            reg,
        ));
        self.handle = Some(handle);
    }

    pub async fn initialize(&self, capabilities: ClientCapabilities) {
        self.client
            .initialize(InitializeParams {
                capabilities,
                initialization_options: None,
                process_id: None,
                root_path: None,
                root_uri: None,
                trace: None,
                workspace_folders: None,
                client_info: None,
            })
            .await
            .unwrap();
        self.client.initialized(InitializedParams {}).await;
    }

    pub fn path(&self, relative_path: &str) -> PathBuf {
        self.dir.path().join(relative_path)
    }

    pub fn uri(&self, relative_path: &str) -> Uri {
        Uri::from_file_path(self.path(relative_path)).unwrap()
    }

    pub fn identifier(&self, relative_path: &str) -> TextDocumentIdentifier {
        TextDocumentIdentifier::new(self.uri(relative_path).into())
    }

    pub async fn open(&self, relative_path: &str) {
        let full_path = self.path(relative_path);
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                language_id: if relative_path.ends_with("bib") {
                    "bibtex".into()
                } else {
                    "latex".into()
                },
                uri: Uri::from_file_path(&full_path).unwrap().into(),
                version: 0,
                text: fs::read_to_string(&full_path).await.unwrap(),
            },
        };
        self.client.did_open(params).await;
    }

    pub async fn edit<S: Into<String>>(&self, relative_path: &str, text: S) {
        let uri = self.uri(relative_path).into();
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(uri, 0),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.into(),
            }],
        };
        self.client.did_change(params).await;
    }

    pub async fn push_options(&self) {
        let options = self.server.options.lock().await.clone();
        let params = DidChangeConfigurationParams {
            settings: serde_json::to_value::<Options>(options).unwrap(),
        };
        self.client.did_change_configuration(params).await
    }

    pub async fn definition_link(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Vec<LocationLink>> {
        let params = TextDocumentPositionParams {
            text_document: self.identifier(relative_path),
            position: Position::new(line, character),
        };
        let response = self.client.definition(params).await.ok()?;
        match response {
            DefinitionResponse::LocationLinks(links) => Some(links),
            DefinitionResponse::Locations(_) => unreachable!(),
        }
    }

    pub async fn definition_location(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Vec<Location>> {
        let params = TextDocumentPositionParams {
            text_document: self.identifier(relative_path),
            position: Position::new(line, character),
        };
        let response = self.client.definition(params).await.ok()?;
        match response {
            DefinitionResponse::LocationLinks(_) => unreachable!(),
            DefinitionResponse::Locations(locations) => Some(locations),
        }
    }

    pub async fn completion(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Vec<CompletionItem>> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: self.identifier(relative_path),
                position: Position::new(line, character),
            },
            context: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        self.client
            .completion(params)
            .await
            .ok()
            .map(|list| list.items)
    }

    pub async fn folding_range(&self, relative_path: &str) -> Option<Vec<FoldingRange>> {
        let params = FoldingRangeParams {
            text_document: self.identifier(relative_path),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        self.client.folding_range(params).await.ok()
    }

    pub async fn document_highlight(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Vec<DocumentHighlight>> {
        let params = TextDocumentPositionParams {
            text_document: self.identifier(relative_path),
            position: Position::new(line, character),
        };
        self.client.document_highlight(params).await.ok()
    }

    pub async fn document_link(&self, relative_path: &str) -> Option<Vec<DocumentLink>> {
        let params = DocumentLinkParams {
            text_document: self.identifier(relative_path),
        };
        self.client.document_link(params).await.ok()
    }

    pub async fn references(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams::new(
                self.identifier(relative_path),
                Position::new(line, character),
            ),
            context: ReferenceContext {
                include_declaration,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        self.client.references(params).await.ok()
    }

    pub async fn prepare_rename(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Option<Range>> {
        let pos = Position::new(line, character);
        let params = TextDocumentPositionParams::new(self.identifier(relative_path), pos);
        self.client.prepare_rename(params).await.ok()
    }

    pub async fn rename<S: Into<String>>(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
        new_name: S,
    ) -> Option<Option<WorkspaceEdit>> {
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams::new(
                self.identifier(relative_path),
                Position::new(line, character),
            ),
            new_name: new_name.into(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        self.client.rename(params).await.ok()
    }

    pub async fn document_symbol_flat(
        &self,
        relative_path: &str,
    ) -> Option<Vec<SymbolInformation>> {
        let params = DocumentSymbolParams {
            text_document: self.identifier(relative_path),
        };
        match self.client.document_symbol(params).await.ok()? {
            DocumentSymbolResponse::Flat(symbols) => Some(symbols),
            DocumentSymbolResponse::Nested(_) => unreachable!(),
        }
    }

    pub async fn document_symbol_nested(&self, relative_path: &str) -> Option<Vec<DocumentSymbol>> {
        let params = DocumentSymbolParams {
            text_document: self.identifier(relative_path),
        };
        match self.client.document_symbol(params).await.ok()? {
            DocumentSymbolResponse::Flat(_) => unreachable!(),
            DocumentSymbolResponse::Nested(symbols) => Some(symbols),
        }
    }

    pub async fn hover(
        &self,
        relative_path: &str,
        line: u64,
        character: u64,
    ) -> Option<Option<Hover>> {
        let params = TextDocumentPositionParams {
            text_document: self.identifier(relative_path),
            position: Position::new(line, character),
        };
        self.client.hover(params).await.ok()
    }

    pub async fn detect_root(&self, relative_path: &str) {
        self.client
            .detect_root(self.identifier(relative_path))
            .await
            .unwrap();
    }

    pub async fn shutdown(&self) {
        self.client.shutdown(()).await.unwrap();
        self.client.exit(()).await;
    }
}

impl Drop for TestBed {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.as_ref() {
            handle.abort();
        }
    }
}

pub static PULL_CAPABILITIES: ClientCapabilities = {
    ClientCapabilities {
        experimental: None,
        text_document: None,
        window: None,
        workspace: Some(WorkspaceClientCapabilities {
            apply_edit: None,
            configuration: Some(true),
            did_change_configuration: None,
            did_change_watched_files: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            workspace_folders: None,
        }),
    }
};

pub static PUSH_CAPABILITIES: ClientCapabilities = {
    ClientCapabilities {
        experimental: None,
        text_document: None,
        window: None,
        workspace: Some(WorkspaceClientCapabilities {
            apply_edit: None,
            configuration: None,
            did_change_configuration: Some(GenericCapability {
                dynamic_registration: Some(true),
            }),
            did_change_watched_files: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            workspace_folders: None,
        }),
    }
};

pub static NESTED_SYMBOL_CAPABILITIES: ClientCapabilities = {
    ClientCapabilities {
        experimental: None,
        text_document: Some(TextDocumentClientCapabilities {
            code_action: None,
            code_lens: None,
            color_provider: None,
            completion: None,
            declaration: None,
            definition: None,
            document_highlight: None,
            document_link: None,
            document_symbol: Some(DocumentSymbolCapability {
                hierarchical_document_symbol_support: Some(true),
                dynamic_registration: None,
                symbol_kind: None,
            }),
            folding_range: None,
            formatting: None,
            hover: None,
            implementation: None,
            on_type_formatting: None,
            publish_diagnostics: None,
            range_formatting: None,
            references: None,
            rename: None,
            signature_help: None,
            synchronization: None,
            type_definition: None,
        }),
        window: None,
        workspace: Some(WorkspaceClientCapabilities {
            apply_edit: None,
            configuration: Some(true),
            did_change_configuration: None,
            did_change_watched_files: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            workspace_folders: None,
        }),
    }
};

pub static LOCATION_LINK_CAPABILITIES: ClientCapabilities = {
    ClientCapabilities {
        experimental: None,
        text_document: Some(TextDocumentClientCapabilities {
            code_action: None,
            code_lens: None,
            color_provider: None,
            completion: None,
            declaration: None,
            definition: Some(GotoCapability {
                dynamic_registration: None,
                link_support: Some(true),
            }),
            document_highlight: None,
            document_link: None,
            document_symbol: None,
            folding_range: None,
            formatting: None,
            hover: None,
            implementation: None,
            on_type_formatting: None,
            publish_diagnostics: None,
            range_formatting: None,
            references: None,
            rename: None,
            signature_help: None,
            synchronization: None,
            type_definition: None,
        }),
        window: None,
        workspace: Some(WorkspaceClientCapabilities {
            apply_edit: None,
            configuration: Some(true),
            did_change_configuration: None,
            did_change_watched_files: None,
            execute_command: None,
            symbol: None,
            workspace_edit: None,
            workspace_folders: None,
        }),
    }
};
