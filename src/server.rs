use crate::{
    build::BuildProvider, config::ConfigManager, diagnostics::DiagnosticsManager, forward_search,
    highlight::HighlightProvider, link::LinkProvider,
};
use async_trait::async_trait;
use futures::lock::Mutex;
use jsonrpc::{server::Result, Middleware};
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::{debug, error, info, warn};
use once_cell::sync::{Lazy, OnceCell};
use std::{mem, path::PathBuf, sync::Arc};
use texlab_citeproc::render_citation;
use texlab_completion::{CompletionItemData, CompletionProvider};
use texlab_components::COMPONENT_DATABASE;
use texlab_definition::DefinitionProvider;
use texlab_feature::{DocumentContent, DocumentView, FeatureProvider, FeatureRequest, Workspace};
use texlab_folding::FoldingProvider;
use texlab_hover::HoverProvider;
use texlab_protocol::*;
use texlab_reference::ReferenceProvider;
use texlab_rename::{PrepareRenameProvider, RenameProvider};
use texlab_symbol::{document_symbols, workspace_symbols, SymbolProvider};
use texlab_syntax::{bibtex, latexindent, CharStream, SyntaxNode};
use texlab_tex::{DistributionKind, DynamicDistribution, KpsewhichError};

pub struct LatexLspServer<C> {
    distro: DynamicDistribution,
    client: Arc<C>,
    client_capabilities: OnceCell<Arc<ClientCapabilities>>,
    current_dir: Arc<PathBuf>,
    config_manager: OnceCell<ConfigManager<C>>,
    action_manager: ActionManager,
    workspace: Workspace,
    build_provider: BuildProvider<C>,
    completion_provider: CompletionProvider,
    definition_provider: DefinitionProvider,
    folding_provider: FoldingProvider,
    highlight_provider: HighlightProvider,
    link_provider: LinkProvider,
    reference_provider: ReferenceProvider,
    prepare_rename_provider: PrepareRenameProvider,
    rename_provider: RenameProvider,
    symbol_provider: SymbolProvider,
    hover_provider: HoverProvider,
    diagnostics_manager: DiagnosticsManager,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync + 'static> LatexLspServer<C> {
    pub fn new(distro: DynamicDistribution, client: Arc<C>, current_dir: Arc<PathBuf>) -> Self {
        let workspace = Workspace::new(distro.clone(), Arc::clone(&current_dir));
        Self {
            distro,
            client: Arc::clone(&client),
            client_capabilities: OnceCell::new(),
            current_dir,
            config_manager: OnceCell::new(),
            action_manager: ActionManager::default(),
            workspace,
            build_provider: BuildProvider::new(client),
            completion_provider: CompletionProvider::new(),
            definition_provider: DefinitionProvider::new(),
            folding_provider: FoldingProvider::new(),
            highlight_provider: HighlightProvider::new(),
            link_provider: LinkProvider::new(),
            reference_provider: ReferenceProvider::new(),
            prepare_rename_provider: PrepareRenameProvider::new(),
            rename_provider: RenameProvider::new(),
            symbol_provider: SymbolProvider::new(),
            hover_provider: HoverProvider::new(),
            diagnostics_manager: DiagnosticsManager::default(),
        }
    }

    fn client_capabilities(&self) -> Arc<ClientCapabilities> {
        Arc::clone(
            self.client_capabilities
                .get()
                .expect("initialize has not been called"),
        )
    }

    fn config_manager(&self) -> &ConfigManager<C> {
        self.config_manager
            .get()
            .expect("initialize has not been called")
    }

    #[jsonrpc_method("initialize", kind = "request")]
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client_capabilities
            .set(Arc::new(params.capabilities))
            .expect("initialize was called two times");

        let _ = self.config_manager.set(ConfigManager::new(
            Arc::clone(&self.client),
            self.client_capabilities(),
        ));

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::Full),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(SaveOptions {
                        include_text: Some(false),
                    }),
                },
            )),
            hover_provider: Some(true),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    "\\".into(),
                    "{".into(),
                    "}".into(),
                    "@".into(),
                    "/".into(),
                    " ".into(),
                ]),
                ..CompletionOptions::default()
            }),
            definition_provider: Some(true),
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: Some(true),
            workspace_symbol_provider: Some(true),
            document_formatting_provider: Some(true),
            rename_provider: Some(RenameProviderCapability::Options(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            ..ServerCapabilities::default()
        };

        Lazy::force(&COMPONENT_DATABASE);
        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "TexLab".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        })
    }

    #[jsonrpc_method("initialized", kind = "notification")]
    pub async fn initialized(&self, _params: InitializedParams) {
        self.action_manager.push(Action::PullConfiguration).await;
        self.action_manager.push(Action::RegisterCapabilities).await;
        self.action_manager.push(Action::LoadDistribution).await;
        self.action_manager.push(Action::PublishDiagnostics).await;
    }

    #[jsonrpc_method("shutdown", kind = "request")]
    pub async fn shutdown(&self, _params: ()) -> Result<()> {
        Ok(())
    }

    #[jsonrpc_method("exit", kind = "notification")]
    pub async fn exit(&self, _params: ()) {}

    #[jsonrpc_method("$/cancelRequest", kind = "notification")]
    pub async fn cancel_request(&self, _params: CancelParams) {}

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let options = self.config_manager().get().await;
        self.workspace.add(params.text_document, &options).await;
        self.action_manager
            .push(Action::DetectRoot(uri.clone().into()))
            .await;
        self.action_manager
            .push(Action::RunLinter(uri.into(), LintReason::Save))
            .await;
        self.action_manager.push(Action::PublishDiagnostics).await;
    }

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let options = self.config_manager().get().await;
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            self.workspace
                .update(uri.into(), change.text, &options)
                .await;
        }
        self.action_manager
            .push(Action::RunLinter(
                params.text_document.uri.clone().into(),
                LintReason::Change,
            ))
            .await;
        self.action_manager.push(Action::PublishDiagnostics).await;
    }

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.action_manager
            .push(Action::Build(params.text_document.uri.clone().into()))
            .await;

        self.action_manager
            .push(Action::RunLinter(
                params.text_document.uri.into(),
                LintReason::Save,
            ))
            .await;
        self.action_manager.push(Action::PublishDiagnostics).await;
    }

    #[jsonrpc_method("textDocument/didClose", kind = "notification")]
    pub async fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("workspace/didChangeConfiguration", kind = "notification")]
    pub async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let config_manager = self.config_manager();
        config_manager.push(params.settings).await;
        let options = config_manager.get().await;
        self.workspace.reparse(&options).await;
    }

    #[jsonrpc_method("window/workDoneProgress/cancel", kind = "notification")]
    pub async fn work_done_progress_cancel(&self, params: WorkDoneProgressCancelParams) {
        self.build_provider.cancel(params.token).await;
    }

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    pub async fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
        let req = self
            .make_feature_request(params.text_document_position.text_document.as_uri(), params)
            .await?;
        Ok(CompletionList {
            is_incomplete: true,
            items: self.completion_provider.execute(&req).await,
        })
    }

    #[jsonrpc_method("completionItem/resolve", kind = "request")]
    pub async fn completion_resolve(&self, mut item: CompletionItem) -> Result<CompletionItem> {
        let data: CompletionItemData = serde_json::from_value(item.data.clone().unwrap()).unwrap();
        match data {
            CompletionItemData::Package | CompletionItemData::Class => {
                item.documentation = COMPONENT_DATABASE
                    .documentation(&item.label)
                    .map(Documentation::MarkupContent);
            }
            CompletionItemData::Citation { uri, key } => {
                let snapshot = self.workspace.get().await;
                if let Some(doc) = snapshot.find(&uri) {
                    if let DocumentContent::Bibtex(tree) = &doc.content {
                        let markup = render_citation(&tree, &key);
                        item.documentation = markup.map(Documentation::MarkupContent);
                    }
                }
            }
            _ => {}
        };
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.hover_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<DefinitionResponse> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        let results = self.definition_provider.execute(&req).await;
        let response = if req.client_capabilities.has_definition_link_support() {
            DefinitionResponse::LocationLinks(results)
        } else {
            DefinitionResponse::Locations(
                results
                    .into_iter()
                    .map(|link| Location::new(link.target_uri, link.target_selection_range))
                    .collect(),
            )
        };

        Ok(response)
    }

    #[jsonrpc_method("textDocument/references", kind = "request")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let req = self
            .make_feature_request(params.text_document_position.as_uri(), params)
            .await?;
        Ok(self.reference_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.highlight_provider.execute(&req).await)
    }

    #[jsonrpc_method("workspace/symbol", kind = "request")]
    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>> {
        let distro = self.distro.clone();
        let client_capabilities = self.client_capabilities();
        let snapshot = self.workspace.get().await;
        let options = self.config_manager().get().await;
        let symbols = workspace_symbols(
            distro,
            client_capabilities,
            snapshot,
            &options,
            Arc::clone(&self.current_dir),
            &params,
        )
        .await;
        Ok(symbols)
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<DocumentSymbolResponse> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;

        let symbols = self.symbol_provider.execute(&req).await;
        let response = document_symbols(
            &req.client_capabilities,
            req.snapshot(),
            &req.current().uri,
            &req.options,
            &req.current_dir,
            symbols.into_iter().map(Into::into).collect(),
        );
        Ok(response)
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.link_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/formatting", kind = "request")]
    pub async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        let mut edits = Vec::new();
        match &req.current().content {
            DocumentContent::Latex(_) => {
                Self::run_latexindent(&req.current().text, "tex", &mut edits).await;
            }
            DocumentContent::Bibtex(tree) => {
                let options = req
                    .options
                    .bibtex
                    .clone()
                    .and_then(|opts| opts.formatting)
                    .unwrap_or_default();

                match options.formatter.unwrap_or_default() {
                    BibtexFormatter::Texlab => {
                        let params = bibtex::FormattingParams {
                            tab_size: req.params.options.tab_size as usize,
                            insert_spaces: req.params.options.insert_spaces,
                            options: &options,
                        };

                        for node in tree.children(tree.root) {
                            let should_format = match &tree.graph[node] {
                                bibtex::Node::Preamble(_) | bibtex::Node::String(_) => true,
                                bibtex::Node::Entry(entry) => !entry.is_comment(),
                                _ => false,
                            };
                            if should_format {
                                let text = bibtex::format(&tree, node, params);
                                edits.push(TextEdit::new(tree.graph[node].range(), text));
                            }
                        }
                    }
                    BibtexFormatter::Latexindent => {
                        Self::run_latexindent(&req.current().text, "bib", &mut edits).await;
                    }
                }
            }
        }
        Ok(edits)
    }

    async fn run_latexindent(old_text: &str, extension: &str, edits: &mut Vec<TextEdit>) {
        match latexindent::format(old_text, extension).await {
            Ok(new_text) => {
                let mut stream = CharStream::new(&old_text);
                while stream.next().is_some() {}
                let range = Range::new(Position::new(0, 0), stream.current_position);
                edits.push(TextEdit::new(range, new_text));
            }
            Err(why) => {
                debug!("Failed to run latexindent.pl: {}", why);
            }
        }
    }

    #[jsonrpc_method("textDocument/prepareRename", kind = "request")]
    pub async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<Range>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.prepare_rename_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/rename", kind = "request")]
    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let req = self
            .make_feature_request(params.text_document_position.text_document.as_uri(), params)
            .await?;
        Ok(self.rename_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.folding_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/build", kind = "request")]
    pub async fn build(&self, params: BuildParams) -> Result<BuildResult> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.build_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/forwardSearch", kind = "request")]
    pub async fn forward_search(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<ForwardSearchResult> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;

        forward_search::search(
            &req.view.snapshot,
            &req.current().uri,
            req.params.position.line,
            &req.options,
            &self.current_dir,
        )
        .await
        .ok_or_else(|| "Unable to execute forward search".into())
    }

    #[jsonrpc_method("$/detectRoot", kind = "request")]
    pub async fn detect_root(&self, params: TextDocumentIdentifier) -> Result<()> {
        let options = self.config_manager().get().await;
        let _ = self.workspace.detect_root(&params.as_uri(), &options).await;
        Ok(())
    }

    async fn make_feature_request<P>(&self, uri: Uri, params: P) -> Result<FeatureRequest<P>> {
        let options = self.pull_configuration().await;
        let snapshot = self.workspace.get().await;
        let client_capabilities = self.client_capabilities();
        match snapshot.find(&uri) {
            Some(current) => Ok(FeatureRequest {
                params,
                view: DocumentView::analyze(snapshot, current, &options, &self.current_dir),
                distro: self.distro.clone(),
                client_capabilities,
                options,
                current_dir: Arc::clone(&self.current_dir),
            }),
            None => {
                let msg = format!("Unknown document: {}", uri);
                Err(msg)
            }
        }
    }

    async fn pull_configuration(&self) -> Options {
        let config_manager = self.config_manager();
        let has_changed = config_manager.pull().await;
        let options = config_manager.get().await;
        if has_changed {
            self.workspace.reparse(&options).await;
        }
        options
    }

    async fn update_build_diagnostics(&self) {
        let snapshot = self.workspace.get().await;
        let options = self.config_manager().get().await;

        for doc in snapshot.0.iter().filter(|doc| doc.uri.scheme() == "file") {
            if let DocumentContent::Latex(table) = &doc.content {
                if table.is_standalone {
                    match self
                        .diagnostics_manager
                        .build
                        .update(&snapshot, &doc.uri, &options, &self.current_dir)
                        .await
                    {
                        Ok(true) => self.action_manager.push(Action::PublishDiagnostics).await,
                        Ok(false) => (),
                        Err(why) => {
                            warn!("Unable to read log file ({}): {}", why, doc.uri.as_str())
                        }
                    }
                }
            }
        }
    }

    async fn load_distribution(&self) {
        info!("Detected TeX distribution: {}", self.distro.0.kind());
        if self.distro.0.kind() == DistributionKind::Unknown {
            let params = ShowMessageParams {
                message: "Your TeX distribution could not be detected. \
                          Please make sure that your distribution is in your PATH."
                    .into(),
                typ: MessageType::Error,
            };
            self.client.show_message(params).await;
        }

        if let Err(why) = self.distro.0.load().await {
            let message = match why {
                KpsewhichError::NotInstalled | KpsewhichError::InvalidOutput => {
                    "An error occurred while executing `kpsewhich`.\
                     Please make sure that your distribution is in your PATH \
                     environment variable and provides the `kpsewhich` tool."
                }
                KpsewhichError::CorruptDatabase | KpsewhichError::NoDatabase => {
                    "The file database of your TeX distribution seems \
                     to be corrupt. Please rebuild it and try again."
                }
                KpsewhichError::Decode(_) => {
                    "An error occurred while decoding the output of `kpsewhich`."
                }
                KpsewhichError::IO(why) => {
                    error!("An I/O error occurred while executing 'kpsewhich': {}", why);
                    "An I/O error occurred while executing 'kpsewhich'"
                }
            };
            let params = ShowMessageParams {
                message: message.into(),
                typ: MessageType::Error,
            };
            self.client.show_message(params).await;
        };
    }
}

#[async_trait]
impl<C: LspClient + Send + Sync + 'static> Middleware for LatexLspServer<C> {
    async fn before_message(&self) {
        if let Some(config_manager) = self.config_manager.get() {
            let options = config_manager.get().await;
            self.workspace.detect_children(&options).await;
            self.workspace.reparse_all_if_newer(&options).await;
        }
    }

    async fn after_message(&self) {
        self.update_build_diagnostics().await;
        for action in self.action_manager.take().await {
            match action {
                Action::LoadDistribution => {
                    self.load_distribution().await;
                }
                Action::RegisterCapabilities => {
                    let config_manager = self.config_manager();
                    config_manager.register().await;
                }
                Action::PullConfiguration => {
                    self.pull_configuration().await;
                }
                Action::DetectRoot(uri) => {
                    let options = self.config_manager().get().await;
                    let _ = self.workspace.detect_root(&uri, &options).await;
                }
                Action::PublishDiagnostics => {
                    let snapshot = self.workspace.get().await;
                    for doc in &snapshot.0 {
                        let diagnostics = self.diagnostics_manager.get(doc).await;
                        let params = PublishDiagnosticsParams {
                            uri: doc.uri.clone().into(),
                            diagnostics,
                            version: None,
                        };
                        self.client.publish_diagnostics(params).await;
                    }
                }
                Action::Build(uri) => {
                    let options = self
                        .config_manager()
                        .get()
                        .await
                        .latex
                        .and_then(|opts| opts.build)
                        .unwrap_or_default();

                    if options.on_save() {
                        let text_document = TextDocumentIdentifier::new(uri.into());
                        self.build(BuildParams { text_document }).await.unwrap();
                    }
                }
                Action::RunLinter(uri, reason) => {
                    let options = self
                        .config_manager()
                        .get()
                        .await
                        .latex
                        .and_then(|opts| opts.lint)
                        .unwrap_or_default();

                    let should_lint = match reason {
                        LintReason::Change => options.on_change(),
                        LintReason::Save => options.on_save() || options.on_change(),
                    };

                    if should_lint {
                        let snapshot = self.workspace.get().await;
                        if let Some(doc) = snapshot.find(&uri) {
                            if let DocumentContent::Latex(_) = &doc.content {
                                self.diagnostics_manager.latex.update(&uri, &doc.text).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LintReason {
    Change,
    Save,
}

#[derive(Debug, PartialEq, Clone)]
enum Action {
    LoadDistribution,
    RegisterCapabilities,
    PullConfiguration,
    DetectRoot(Uri),
    PublishDiagnostics,
    Build(Uri),
    RunLinter(Uri, LintReason),
}

#[derive(Debug, Default)]
struct ActionManager {
    actions: Mutex<Vec<Action>>,
}

impl ActionManager {
    pub async fn push(&self, action: Action) {
        let mut actions = self.actions.lock().await;
        actions.push(action);
    }

    pub async fn take(&self) -> Vec<Action> {
        let mut actions = self.actions.lock().await;
        mem::replace(&mut *actions, Vec::new())
    }
}
