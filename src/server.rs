use crate::{
    components::COMPONENT_DATABASE,
    config::ConfigManager,
    jsonrpc::{server::Result, Middleware},
    protocol::*,
    tex::Distribution,
    workspace::Workspace,
};
use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use once_cell::sync::{Lazy, OnceCell};
use std::{mem, path::PathBuf, sync::Arc};

pub struct LatexLspServer<C> {
    distro: Arc<Box<dyn Distribution + Send + Sync>>,
    client: Arc<C>,
    client_capabilities: OnceCell<Arc<ClientCapabilities>>,
    config_manager: OnceCell<ConfigManager<C>>,
    action_manager: ActionManager,
    workspace: Workspace,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync + 'static> LatexLspServer<C> {
    pub fn new(
        distro: Arc<Box<dyn Distribution + Send + Sync>>,
        client: Arc<C>,
        current_dir: PathBuf,
    ) -> Self {
        let workspace = Workspace::new(Arc::clone(&distro), current_dir);
        Self {
            distro,
            client,
            client_capabilities: OnceCell::new(),
            config_manager: OnceCell::new(),
            action_manager: ActionManager::default(),
            workspace,
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
    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {}

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {}

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub async fn did_save(&self, params: DidSaveTextDocumentParams) {}

    #[jsonrpc_method("textDocument/didClose", kind = "notification")]
    pub async fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("workspace/didChangeConfiguration", kind = "notification")]
    pub async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.config_manager().push(params.settings).await;
    }

    #[jsonrpc_method("window/workDoneProgress/cancel", kind = "notification")]
    pub async fn work_done_progress_cancel(&self, params: WorkDoneProgressCancelParams) {}

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    pub async fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
        Ok(CompletionList {
            is_incomplete: true,
            items: Vec::new(),
        })
    }

    #[jsonrpc_method("completionItem/resolve", kind = "request")]
    pub async fn completion_resolve(&self, mut item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<DefinitionResponse> {
        Ok(DefinitionResponse::Locations(Vec::new()))
    }

    #[jsonrpc_method("textDocument/references", kind = "request")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("workspace/symbol", kind = "request")]
    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<DocumentSymbolResponse> {
        Ok(DocumentSymbolResponse::Flat(Vec::new()))
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/formatting", kind = "request")]
    pub async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/prepareRename", kind = "request")]
    pub async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<Range>> {
        Ok(None)
    }

    #[jsonrpc_method("textDocument/rename", kind = "request")]
    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(None)
    }

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/build", kind = "request")]
    pub async fn build(&self, params: BuildParams) -> Result<BuildResult> {
        Ok(BuildResult {
            status: BuildStatus::Failure,
        })
    }

    #[jsonrpc_method("textDocument/forwardSearch", kind = "request")]
    pub async fn forward_search(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<ForwardSearchResult> {
        Ok(ForwardSearchResult {
            status: ForwardSearchStatus::Failure,
        })
    }
}

impl<C: LspClient + Send + Sync + 'static> Middleware for LatexLspServer<C> {
    #[boxed]
    async fn before_message(&self) {}

    #[boxed]
    async fn after_message(&self) {
        for action in self.action_manager.take().await {
            match action {
                Action::PullConfiguration => {
                    let config_manager = self.config_manager();
                    if config_manager.pull().await {
                        let options = config_manager.get().await;
                        self.workspace.reparse(&options).await;
                    }
                }
                Action::RegisterCapabilities => self.config_manager().register().await,
            };
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Action {
    PullConfiguration,
    RegisterCapabilities,
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
