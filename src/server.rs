use crate::{
    citeproc::render_citation,
    completion::{CompletionItemData, CompletionProvider},
    components::COMPONENT_DATABASE,
    config::ConfigManager,
    feature::{DocumentView, FeatureProvider, FeatureRequest},
    folding::FoldingProvider,
    highlight::HighlightProvider,
    jsonrpc::{server::Result, Middleware},
    link::LinkProvider,
    protocol::*,
    reference::ReferenceProvider,
    rename::{PrepareRenameProvider, RenameProvider},
    tex::{DistributionKind, DynamicDistribution, KpsewhichError},
    workspace::{DocumentContent, Workspace},
};
use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::{error, info};
use once_cell::sync::{Lazy, OnceCell};
use std::{mem, path::PathBuf, sync::Arc};

pub struct LatexLspServer<C> {
    distro: DynamicDistribution,
    client: Arc<C>,
    client_capabilities: OnceCell<Arc<ClientCapabilities>>,
    current_dir: Arc<PathBuf>,
    config_manager: OnceCell<ConfigManager<C>>,
    action_manager: ActionManager,
    workspace: Workspace,
    completion_provider: CompletionProvider,
    folding_provider: FoldingProvider,
    highlight_provider: HighlightProvider,
    link_provider: LinkProvider,
    reference_provider: ReferenceProvider,
    prepare_rename_provider: PrepareRenameProvider,
    rename_provider: RenameProvider,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync + 'static> LatexLspServer<C> {
    pub fn new(distro: DynamicDistribution, client: Arc<C>, current_dir: Arc<PathBuf>) -> Self {
        let workspace = Workspace::new(distro.clone(), Arc::clone(&current_dir));
        Self {
            distro,
            client,
            client_capabilities: OnceCell::new(),
            current_dir,
            config_manager: OnceCell::new(),
            action_manager: ActionManager::default(),
            workspace,
            completion_provider: CompletionProvider::new(),
            folding_provider: FoldingProvider::new(),
            highlight_provider: HighlightProvider::new(),
            link_provider: LinkProvider::new(),
            reference_provider: ReferenceProvider::new(),
            prepare_rename_provider: PrepareRenameProvider::new(),
            rename_provider: RenameProvider::new(),
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
            }),
            signature_help_provider: None,
            definition_provider: Some(true),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: Some(true),
            workspace_symbol_provider: Some(true),
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: Some(true),
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: Some(RenameProviderCapability::Options(RenameOptions {
                prepare_provider: Some(true),
            })),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
            }),
            color_provider: None,
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            execute_command_provider: None,
            workspace: None,
            selection_range_provider: None,
        };

        Lazy::force(&COMPONENT_DATABASE);
        Ok(InitializeResult { capabilities })
    }

    #[jsonrpc_method("initialized", kind = "notification")]
    pub async fn initialized(&self, _params: InitializedParams) {
        self.action_manager.push(Action::PullConfiguration).await;
        self.action_manager.push(Action::RegisterCapabilities).await;
        self.action_manager.push(Action::LoadDistribution).await;
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
    }

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub async fn did_save(&self, _params: DidSaveTextDocumentParams) {}

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
    pub async fn work_done_progress_cancel(&self, _params: WorkDoneProgressCancelParams) {}

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
    pub async fn hover(&self, _params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(
        &self,
        _params: TextDocumentPositionParams,
    ) -> Result<DefinitionResponse> {
        Ok(DefinitionResponse::Locations(Vec::new()))
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
        _params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        _params: DocumentSymbolParams,
    ) -> Result<DocumentSymbolResponse> {
        Ok(DocumentSymbolResponse::Flat(Vec::new()))
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let req = self
            .make_feature_request(params.text_document.as_uri(), params)
            .await?;
        Ok(self.link_provider.execute(&req).await)
    }

    #[jsonrpc_method("textDocument/formatting", kind = "request")]
    pub async fn formatting(&self, _params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
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
    pub async fn build(&self, _params: BuildParams) -> Result<BuildResult> {
        Ok(BuildResult {
            status: BuildStatus::Failure,
        })
    }

    #[jsonrpc_method("textDocument/forwardSearch", kind = "request")]
    pub async fn forward_search(
        &self,
        _params: TextDocumentPositionParams,
    ) -> Result<ForwardSearchResult> {
        Ok(ForwardSearchResult {
            status: ForwardSearchStatus::Failure,
        })
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

impl<C: LspClient + Send + Sync + 'static> Middleware for LatexLspServer<C> {
    #[boxed]
    async fn before_message(&self) {
        if let Some(config_manager) = self.config_manager.get() {
            let options = config_manager.get().await;
            self.workspace.detect_children(&options).await;
            self.workspace.reparse_all_if_newer(&options).await;
        }
    }

    #[boxed]
    async fn after_message(&self) {
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
            };
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Action {
    LoadDistribution,
    RegisterCapabilities,
    PullConfiguration,
    DetectRoot(Uri),
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
